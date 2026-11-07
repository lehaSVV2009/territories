#![allow(clippy::useless_conversion)]

pub mod game;
pub mod selector;
pub mod tree;
pub mod appex;

pub use game::{
    generate_default_v2_matrix, select_move, simulate_match, Board, BotType, HeuristicWeights,
    MatchResult, Move, Mulberry32, Player, BOARD_HEIGHT, BOARD_WIDTH, EMPTY, PLAYER_1, PLAYER_2,
    TOTAL_CELLS,
};
pub use selector::{
    load_from_file, save_to_file, CandidateModel, CandidateStats, CrnPairResult, HeuristicParams,
    ModelSelector, SelectorError, SelectorMetadata, WeightsFile,
};
pub use tree::{
    compute_chokepoint_penalty, detect_narrow_chokepoints, export_to_dot, export_to_json,
    BoardSummary, DecisionTree, MoveBreakdown, MoveCandidate, RectCoords, SearchPhase, TreeNode,
    TreeNodeType, TwoPlyThreat,
};
pub use appex::*;

use std::fs::File;
use std::io::{Read, Write};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;

/// Parses a bot type string into the `BotType` enum.
fn parse_bot_type(s: &str) -> PyResult<BotType> {
    let lower = s.to_lowercase().replace(['_', '-'], "");
    if lower.contains("greedy") {
        Ok(BotType::Greedy)
    } else if lower.contains("fast") {
        Ok(BotType::Fast)
    } else if lower.contains("legacy") || lower == "hard" || lower == "hardbot" {
        Ok(BotType::LegacyHard)
    } else if lower.contains("slayer")
        || lower.contains("spatial")
        || lower == "v2"
        || lower.contains("champ")
    {
        Ok(BotType::SpatialSlayer)
    } else {
        Err(PyValueError::new_err(format!("Unknown bot type: '{s}'")))
    }
}

/// Helper to select a move and its score using the SpatialSlayer policy from Player 2 perspective.
fn select_move_spatial_slayer_with_score_p2(
    board: &Board,
    d1: usize,
    d2: usize,
    weights: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> Option<(Move, f64)> {
    let all_rects = board.generate_legal_moves(Player::Player2, d1, d2);
    if all_rects.is_empty() {
        return None;
    }

    let mut empty_cells = 0;
    let mut p1_max_col = 0;
    let mut p2_min_col = BOARD_WIDTH;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let cell = board.cells[row_offset + c];
            if cell == EMPTY {
                empty_cells += 1;
            } else if cell == PLAYER_1 {
                if c > p1_max_col {
                    p1_max_col = c;
                }
            } else if cell == PLAYER_2 && c < p2_min_col {
                p2_min_col = c;
            }
        }
    }

    let is_opening = empty_cells > 490;
    let has_contact = p2_min_col <= p1_max_col + 3;

    let mut scored_candidates: Vec<(Move, f64)> = all_rects
        .iter()
        .map(|&rect| {
            let score = game::evaluate_move_v2(board, &rect, weights, matrix);
            (rect, score)
        })
        .collect();

    scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top_count = scored_candidates.len().min(4);
    let mut best_cand = scored_candidates[0].0;
    let mut best_total_score = -f64::INFINITY;

    for cand in scored_candidates.iter().take(top_count) {
        let mut adjusted_score = cand.1;

        if has_contact || !is_opening {
            let opp_threat = game::evaluate_counter_threat(board, cand.0.r, cand.0.c, cand.0.h, cand.0.w);
            if opp_threat > 0 {
                adjusted_score -= (opp_threat as f64) * (weights.weight_deny_capture * 0.8);
            }
        }

        if adjusted_score > best_total_score {
            best_total_score = adjusted_score;
            best_cand = cand.0;
        }
    }

    Some((best_cand, best_total_score))
}

/// Helper to select a move and its score for either player.
fn select_move_spatial_slayer_with_score(
    board: &Board,
    player: Player,
    d1: usize,
    d2: usize,
    weights: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> Option<(Move, f64)> {
    if player == Player::Player1 {
        let rotated = board.rotate_board_180();
        let (rot_mv, score) =
            select_move_spatial_slayer_with_score_p2(&rotated, d1, d2, weights, matrix)?;
        let real_mv = Board::inverse_move_180(&rot_mv);
        Some((real_mv, score))
    } else {
        select_move_spatial_slayer_with_score_p2(board, d1, d2, weights, matrix)
    }
}

/// Helper to reconstruct Board from 600 flat cells.
fn board_from_cells(board_cells: &[u8]) -> PyResult<Board> {
    if board_cells.len() != TOTAL_CELLS {
        return Err(PyValueError::new_err(format!(
            "board_cells must have exactly {} elements, got {}",
            TOTAL_CELLS,
            board_cells.len()
        )));
    }
    let mut cells = [0u8; TOTAL_CELLS];
    cells.copy_from_slice(board_cells);

    let mut p1_score = 0u16;
    let mut p2_score = 0u16;
    for &c in &cells {
        if c == PLAYER_1 {
            p1_score += 1;
        } else if c == PLAYER_2 {
            p2_score += 1;
        }
    }

    Ok(Board {
        cells,
        p1_score,
        p2_score,
        turn: 1,
        passes: 0,
    })
}

/// 1. Coarse-Grained Headless Match Simulation FFI.
/// Releases GIL with `py.allow_threads` during execution.
#[pyfunction]
#[pyo3(signature = (p1_type, p2_type, seed, p1_weights=None, p2_weights=None, max_turns=None))]
pub fn simulate_match_ffi(
    py: Python<'_>,
    p1_type: &str,
    p2_type: &str,
    seed: u32,
    p1_weights: Option<Vec<f64>>,
    p2_weights: Option<Vec<f64>>,
    max_turns: Option<usize>,
) -> PyResult<(Option<u8>, u16, u16, usize)> {
    let p1_bot = parse_bot_type(p1_type)?;
    let p2_bot = parse_bot_type(p2_type)?;
    let turns_limit = max_turns.unwrap_or(80);

    let w1 = p1_weights.as_ref().map(|w| HeuristicWeights::from_slice(w));
    let w2 = p2_weights.as_ref().map(|w| HeuristicWeights::from_slice(w));
    let default_matrix = generate_default_v2_matrix();

    let res = py.allow_threads(|| {
        simulate_match(
            p1_bot,
            p2_bot,
            Some(seed),
            turns_limit,
            w1.as_ref(),
            w2.as_ref(),
            Some(&default_matrix),
            Some(&default_matrix),
        )
    });

    let winner_u8 = res.winner.map(|p| p.cell_value());
    Ok((winner_u8, res.p1_score, res.p2_score, res.turns))
}

/// 2. Batch Candidate Population Evaluation FFI.
/// Evaluates candidate weights in parallel across CPU cores with Rayon global thread pool,
/// releasing GIL with `py.allow_threads`. Preserves input-to-output index order determinism.
#[pyfunction]
#[pyo3(signature = (candidates_params, candidates_matrices=None, opponent_type="GreedyBot", seeds=vec![], num_workers=4, max_turns=None))]
pub fn evaluate_population_ffi<'py>(
    py: Python<'py>,
    candidates_params: Vec<Vec<f64>>,
    candidates_matrices: Option<Vec<Vec<f64>>>,
    opponent_type: &str,
    seeds: Vec<u32>,
    num_workers: usize,
    max_turns: Option<usize>,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let _ = num_workers;
    let opponent_bot = parse_bot_type(opponent_type)?;
    let turns_limit = max_turns.unwrap_or(80);
    let default_matrix = generate_default_v2_matrix();
    let num_candidates = candidates_params.len();

    // Prepare candidate matrices with fallback to default_matrix
    let mut prepared_matrices = Vec::with_capacity(num_candidates);
    for i in 0..num_candidates {
        if let Some(ref mats) = candidates_matrices {
            if i < mats.len() && mats[i].len() == TOTAL_CELLS {
                let mut arr = [0.0; TOTAL_CELLS];
                arr.copy_from_slice(&mats[i]);
                prepared_matrices.push(arr);
                continue;
            }
        }
        prepared_matrices.push(default_matrix);
    }

    let seeds_ref = &seeds;

    // Parallel evaluation on the global Rayon thread pool with GIL released.
    // par_iter().zip().map().collect() guarantees strict index order preservation.
    let stats_vec: Vec<CandidateStats> = py.allow_threads(|| {
        candidates_params
            .par_iter()
            .zip(prepared_matrices.par_iter())
            .map(|(params, mat)| {
                let p = HeuristicParams::from_slice(params);
                ModelSelector::evaluate_candidate_vs_bot(&p, mat, opponent_bot, seeds_ref, turns_limit)
            })
            .collect()
    });

    let mut py_results = Vec::with_capacity(stats_vec.len());
    for s in stats_vec {
        let dict = PyDict::new_bound(py);
        dict.set_item("winrate", s.winrate)?;
        dict.set_item("avg_margin", s.avg_margin)?;
        dict.set_item("avg_turns", s.avg_turns)?;
        dict.set_item("fitness", s.fitness)?;
        dict.set_item("wins", s.wins)?;
        dict.set_item("losses", s.losses)?;
        dict.set_item("draws", s.draws)?;
        dict.set_item("total_matches", s.total_matches)?;
        dict.set_item("total_turns", s.total_turns)?;
        dict.set_item("net_margin", s.net_margin)?;
        py_results.push(dict);
    }

    Ok(py_results)
}

/// 3. Move Inference FFI.
/// Evaluates and selects the optimal move from the current board state under <2.0 ms SLA.
/// Returns `(r0, c0, h, w, score)`.
#[pyfunction]
pub fn select_move_ffi(
    py: Python<'_>,
    board_cells: Vec<u8>,
    d1: usize,
    d2: usize,
    weights: Vec<f64>,
) -> PyResult<(usize, usize, usize, usize, f64)> {
    let board = board_from_cells(&board_cells)?;
    let hw = HeuristicWeights::from_slice(&weights);
    let matrix = generate_default_v2_matrix();

    // Determine acting player: default Player 2, fallback to Player 1 if P2 has no moves but P1 does
    let player = if board.generate_legal_moves(Player::Player2, d1, d2).is_empty()
        && !board.generate_legal_moves(Player::Player1, d1, d2).is_empty()
    {
        Player::Player1
    } else {
        Player::Player2
    };

    let result = py.allow_threads(|| {
        select_move_spatial_slayer_with_score(&board, player, d1, d2, &hw, &matrix)
    });

    if let Some((mv, score)) = result {
        Ok((mv.r, mv.c, mv.h, mv.w, score))
    } else {
        Ok((0, 0, 0, 0, 0.0))
    }
}

/// 4. Export Decision Search Tree to Graphviz (.dot) format.
#[pyfunction]
pub fn export_tree_dot(
    board_cells: Vec<u8>,
    d1: usize,
    d2: usize,
    weights: Vec<f64>,
    out_path: &str,
) -> PyResult<()> {
    let board = board_from_cells(&board_cells)?;
    let hw = HeuristicWeights::from_slice(&weights);
    let matrix = generate_default_v2_matrix();

    let player = if board.generate_legal_moves(Player::Player2, d1, d2).is_empty()
        && !board.generate_legal_moves(Player::Player1, d1, d2).is_empty()
    {
        Player::Player1
    } else {
        Player::Player2
    };

    let tree = DecisionTree::build(&board, player, d1, d2, &hw, &matrix, 4);
    let dot_str = export_to_dot(&tree);

    std::fs::write(out_path, dot_str).map_err(|e| {
        PyIOError::new_err(format!("Failed to write DOT to '{out_path}': {e}"))
    })?;

    Ok(())
}

/// 5. Export Decision Search Tree to JSON format.
#[pyfunction]
pub fn export_tree_json(
    board_cells: Vec<u8>,
    d1: usize,
    d2: usize,
    weights: Vec<f64>,
    out_path: &str,
) -> PyResult<()> {
    let board = board_from_cells(&board_cells)?;
    let hw = HeuristicWeights::from_slice(&weights);
    let matrix = generate_default_v2_matrix();

    let player = if board.generate_legal_moves(Player::Player2, d1, d2).is_empty()
        && !board.generate_legal_moves(Player::Player1, d1, d2).is_empty()
    {
        Player::Player1
    } else {
        Player::Player2
    };

    let tree = DecisionTree::build(&board, player, d1, d2, &hw, &matrix, 4);
    let json_str = export_to_json(&tree);

    std::fs::write(out_path, json_str).map_err(|e| {
        PyIOError::new_err(format!("Failed to write JSON to '{out_path}': {e}"))
    })?;

    Ok(())
}

/// 6. Save model parameters, spatial matrix, and metadata to JSON file conforming to `weights.json`.
/// 100% compatible with browser JS build: exactly 600 float matrix and all 10 keys in params object.
#[pyfunction]
#[pyo3(signature = (path, weights, matrix, metadata=None))]
pub fn save_weights_file(
    path: &str,
    weights: Vec<f64>,
    matrix: Vec<f64>,
    metadata: Option<Bound<'_, PyDict>>,
) -> PyResult<()> {
    if matrix.len() != TOTAL_CELLS {
        return Err(PyValueError::new_err(format!(
            "matrix must have exactly {} elements, got {}",
            TOTAL_CELLS,
            matrix.len()
        )));
    }

    let params = HeuristicParams::from_slice(&weights);
    let mut meta = SelectorMetadata::default();

    if let Some(dict) = metadata {
        if let Ok(Some(item)) = dict.get_item("epoch") {
            if let Ok(val) = item.extract::<usize>() {
                meta.epoch = val;
            }
        }
        if let Ok(Some(item)) = dict.get_item("winrate_vs_hard") {
            if let Ok(val) = item.extract::<f64>() {
                meta.winrate_vs_hard = val;
            }
        }
        if let Ok(Some(item)) = dict.get_item("winrate_vs_greedy") {
            if let Ok(val) = item.extract::<f64>() {
                meta.winrate_vs_greedy = val;
            }
        }
        if let Ok(Some(item)) = dict.get_item("winrate_vs_champ") {
            if let Ok(val) = item.extract::<f64>() {
                meta.winrate_vs_champ = val;
            }
        }
        if let Ok(Some(item)) = dict.get_item("avg_margin") {
            if let Ok(val) = item.extract::<f64>() {
                meta.avg_margin = val;
            }
        }
        if let Ok(Some(item)) = dict.get_item("fitness") {
            if let Ok(val) = item.extract::<f64>() {
                meta.fitness = val;
            }
        }
    }

    let file_struct = serde_json::json!({
        "name": "SpatialSlayer-V2",
        "version": "2.0.0",
        "architecture": "Argon-4 / Phase-Adaptive + Loop Denial",
        "weights": params,
        "params": params,
        "matrix": matrix,
        "spatial_matrix": matrix,
        "metadata": meta,
    });

    let json_str = serde_json::to_string_pretty(&file_struct).map_err(|e| {
        PyValueError::new_err(format!("Failed to format JSON: {e}"))
    })?;

    let mut file = File::create(path).map_err(|e| {
        PyIOError::new_err(format!("Failed to create '{path}': {e}"))
    })?;

    file.write_all(json_str.as_bytes()).map_err(|e| {
        PyIOError::new_err(format!("Failed to write to '{path}': {e}"))
    })?;

    Ok(())
}

/// 7. Load model parameters, spatial matrix, and metadata from JSON file.
#[pyfunction]
pub fn load_weights_file<'py>(
    py: Python<'py>,
    path: &str,
) -> PyResult<(Vec<f64>, Vec<f64>, Bound<'py, PyDict>)> {
    let mut file = File::open(path).map_err(|e| {
        PyIOError::new_err(format!("Failed to open '{path}': {e}"))
    })?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| {
        PyIOError::new_err(format!("Failed to read '{path}': {e}"))
    })?;

    let parsed: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
        PyValueError::new_err(format!("Failed to parse JSON from '{path}': {e}"))
    })?;

    // Extract weights / params
    let params: HeuristicParams = if let Some(p) = parsed.get("params") {
        serde_json::from_value(p.clone()).unwrap_or_default()
    } else if let Some(w) = parsed.get("weights") {
        serde_json::from_value(w.clone()).unwrap_or_default()
    } else {
        HeuristicParams::default()
    };

    // Extract matrix / spatial_matrix
    let matrix_vec: Vec<f64> = if let Some(m) = parsed.get("matrix").and_then(|v| v.as_array()) {
        m.iter().filter_map(|v| v.as_f64()).collect()
    } else if let Some(sm) = parsed.get("spatial_matrix").and_then(|v| v.as_array()) {
        sm.iter().filter_map(|v| v.as_f64()).collect()
    } else {
        generate_default_v2_matrix().to_vec()
    };

    if matrix_vec.len() != TOTAL_CELLS {
        return Err(PyValueError::new_err(format!(
            "Expected {} elements in matrix, found {}",
            TOTAL_CELLS,
            matrix_vec.len()
        )));
    }

    // Extract metadata
    let dict = PyDict::new_bound(py);
    if let Some(meta) = parsed.get("metadata").and_then(|v| v.as_object()) {
        for (k, v) in meta {
            match v {
                serde_json::Value::Number(num) => {
                    if let Some(i) = num.as_i64() {
                        dict.set_item(k, i)?;
                    } else if let Some(f) = num.as_f64() {
                        dict.set_item(k, f)?;
                    }
                }
                serde_json::Value::String(s) => {
                    dict.set_item(k, s.as_str())?;
                }
                serde_json::Value::Bool(b) => {
                    dict.set_item(k, *b)?;
                }
                _ => {}
            }
        }
    }

    Ok((params.to_vec(), matrix_vec, dict))
}

/// 8. Appex Headless Match Simulation FFI.
/// Releases GIL with `py.allow_threads` during execution.
/// Returns `(winner_u8, p1_score, p2_score, turns)`.
#[pyfunction]
#[pyo3(signature = (appex_player, opp_type, seed, counter_weights=None, max_turns=None))]
pub fn simulate_appex_match_ffi(
    py: Python<'_>,
    appex_player: u8,
    opp_type: &str,
    seed: u32,
    counter_weights: Option<Vec<Vec<f64>>>,
    max_turns: Option<usize>,
) -> PyResult<(Option<u8>, u16, u16, usize)> {
    let player = if appex_player == 1 {
        Player::Player1
    } else {
        Player::Player2
    };
    let turns_limit = max_turns.unwrap_or(80);
    let default_matrix = generate_default_v2_matrix();

    let res = py.allow_threads(|| {
        let mut appex_weights = AppexWeights::default();
        if let Some(cw) = counter_weights {
            for (i, params) in cw.iter().enumerate().take(4) {
                appex_weights.counter_weights[i] =
                    HeuristicWeights::from(&HeuristicParams::from_slice(params));
            }
        }
        let mut appex = AppexBot::new(appex_weights, default_matrix);
        simulate_appex_match(player, &mut appex, opp_type, seed, turns_limit)
    });

    let winner_u8 = res.winner.map(|p| p.cell_value());
    Ok((winner_u8, res.p1_score, res.p2_score, res.turns))
}

/// 9. Appex Counter-Population Evaluation FFI.
/// Evaluates candidate counter-weights across mirrored CRN seeds in parallel using Rayon.
/// Releases GIL with `py.allow_threads`. Preserves input-to-output index order determinism.
#[pyfunction]
#[pyo3(signature = (candidates_counter_params, target_archetype, seeds, max_turns=None))]
pub fn evaluate_appex_counter_population_ffi<'py>(
    py: Python<'py>,
    candidates_counter_params: Vec<Vec<f64>>,
    target_archetype: &str,
    seeds: Vec<u32>,
    max_turns: Option<usize>,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let turns_limit = max_turns.unwrap_or(80);
    let default_matrix = generate_default_v2_matrix();
    let seeds_ref = &seeds;
    let target_ref = target_archetype;

    let stats_vec: Vec<AppexEvaluationStats> = py.allow_threads(|| {
        candidates_counter_params
            .par_iter()
            .map(|params| {
                let p = HeuristicParams::from_slice(params);
                let hw = HeuristicWeights::from(&p);
                evaluate_counter_candidate_crn(&hw, &default_matrix, target_ref, seeds_ref, turns_limit)
            })
            .collect()
    });

    let mut py_results = Vec::with_capacity(stats_vec.len());
    for s in stats_vec {
        let dict = PyDict::new_bound(py);
        dict.set_item("winrate", s.winrate)?;
        dict.set_item("avg_margin", s.avg_margin)?;
        dict.set_item("avg_turns", s.avg_turns)?;
        dict.set_item("fitness", s.fitness)?;
        dict.set_item("wins", s.wins)?;
        dict.set_item("losses", s.losses)?;
        dict.set_item("draws", s.draws)?;
        dict.set_item("total_matches", s.total_matches)?;
        dict.set_item("net_margin", s.net_margin)?;
        py_results.push(dict);
    }

    Ok(py_results)
}

/// Alias for evaluate_appex_counter_population_ffi for compatibility.
#[pyfunction]
#[pyo3(signature = (candidates_counter_params, target_archetype, seeds, max_turns=None))]
pub fn evaluate_counter_population_ffi<'py>(
    py: Python<'py>,
    candidates_counter_params: Vec<Vec<f64>>,
    target_archetype: &str,
    seeds: Vec<u32>,
    max_turns: Option<usize>,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    evaluate_appex_counter_population_ffi(
        py,
        candidates_counter_params,
        target_archetype,
        seeds,
        max_turns,
    )
}

/// PyRust PyO3 module initialization.
#[pymodule]
fn pyrust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize Rayon global thread pool to <= 3 worker threads once
    let threads = std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let _ = rayon::ThreadPoolBuilder::new().num_threads(threads).build_global();

    // Register all PyO3 FFI functions
    m.add_function(wrap_pyfunction!(simulate_match_ffi, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_population_ffi, m)?)?;
    m.add_function(wrap_pyfunction!(select_move_ffi, m)?)?;
    m.add_function(wrap_pyfunction!(export_tree_dot, m)?)?;
    m.add_function(wrap_pyfunction!(export_tree_json, m)?)?;
    m.add_function(wrap_pyfunction!(save_weights_file, m)?)?;
    m.add_function(wrap_pyfunction!(load_weights_file, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_appex_match_ffi, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_appex_counter_population_ffi, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_counter_population_ffi, m)?)?;

    Ok(())
}
