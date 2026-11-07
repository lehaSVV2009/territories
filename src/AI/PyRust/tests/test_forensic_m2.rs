//! Independent Forensic Audit Suite for Milestone M2 (`src/selector.rs` and `src/tree.rs`).
//!
//! Validates:
//! 1. ModelSelector calculates actual, non-hardcoded winrates, margins, and fitness from match simulations.
//! 2. `weights.json` serialization and deserialization performs genuine disk I/O without dummy shortcuts.
//! 3. `DecisionTree` performs genuine search tree construction, dynamic 1-ply evaluation, beam pruning (Top-4/5),
//!    chokepoint detection, and dynamic Graphviz DOT & JSON generation.
//! 4. Syntax validation rejects corrupted or malformed DOT and JSON.

use pyrust::game::{
    generate_default_v2_matrix, Board, BotType, HeuristicWeights, Move, Player,
    BOARD_WIDTH,
};
use pyrust::selector::{
    load_from_file, save_to_file, HeuristicParams, ModelSelector, SelectorError,
    SelectorMetadata,
};
use pyrust::tree::{
    compute_chokepoint_penalty, detect_narrow_chokepoints, DecisionTree,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_path(prefix: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("{}_{}_{}.{}", prefix, std::process::id(), nanos, ext));
    path
}

// ============================================================================
// 1. Selector Forensic Verification: Calculation Authenticity & Logic
// ============================================================================

#[test]
fn test_forensic_selector_real_simulation_calculations() {
    let mut selector = ModelSelector::with_default_champion();

    // Candidate 1: standard clamped default
    let p1 = HeuristicParams::default();
    // Candidate 2: mutated voronoi delta
    let mut p2 = HeuristicParams::default();
    p2.weight_voronoi_delta = 35.0;

    let id1 = selector.add_candidate(p1, generate_default_v2_matrix());
    let id2 = selector.add_candidate(p2, generate_default_v2_matrix());

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    // Evaluate both candidates against champion across real simulation seeds
    let seeds = [101u32, 202u32, 303u32];
    selector.evaluate_population_vs_champion(&seeds, 40);

    let cand1 = &selector.population[0];
    let cand2 = &selector.population[1];

    println!(
        "Cand 1 vs Champ (3 seeds, 6 matches): wins={}, losses={}, draws={}, net_margin={:.1}, winrate={:.3}, avg_margin={:.2}, fitness={:.2}",
        cand1.stats.wins, cand1.stats.losses, cand1.stats.draws, cand1.stats.net_margin, cand1.stats.winrate, cand1.stats.avg_margin, cand1.stats.fitness
    );
    println!(
        "Cand 2 vs Champ (3 seeds, 6 matches): wins={}, losses={}, draws={}, net_margin={:.1}, winrate={:.3}, avg_margin={:.2}, fitness={:.2}",
        cand2.stats.wins, cand2.stats.losses, cand2.stats.draws, cand2.stats.net_margin, cand2.stats.winrate, cand2.stats.avg_margin, cand2.stats.fitness
    );

    // Total matches must strictly equal 2 * seeds.len() = 6
    assert_eq!(cand1.stats.total_matches, 6);
    assert_eq!(cand2.stats.total_matches, 6);
    assert_eq!(
        cand1.stats.wins + cand1.stats.losses + cand1.stats.draws,
        6
    );
    assert_eq!(
        cand2.stats.wins + cand2.stats.losses + cand2.stats.draws,
        6
    );

    // Mathematical integrity of winrate and fitness formulas
    let expected_winrate1 = cand1.stats.wins as f64 / 6.0;
    assert!((cand1.stats.winrate - expected_winrate1).abs() < 1e-6);

    let expected_fitness1 = cand1.stats.winrate * 100.0 + cand1.stats.avg_margin * 2.0;
    assert!((cand1.stats.fitness - expected_fitness1).abs() < 1e-6);

    let expected_winrate2 = cand2.stats.wins as f64 / 6.0;
    assert!((cand2.stats.winrate - expected_winrate2).abs() < 1e-6);

    let expected_fitness2 = cand2.stats.winrate * 100.0 + cand2.stats.avg_margin * 2.0;
    assert!((cand2.stats.fitness - expected_fitness2).abs() < 1e-6);

    // Dynamic ranking
    let ranking = selector.rank_population();
    assert_eq!(ranking.len(), 2);
    if cand1.stats.fitness >= cand2.stats.fitness {
        assert_eq!(ranking, vec![0, 1]);
    } else {
        assert_eq!(ranking, vec![1, 0]);
    }

    // Bot evaluation against GreedyBot
    let bot_stats = ModelSelector::evaluate_candidate_vs_bot(
        &cand1.weights,
        &cand1.matrix,
        BotType::Greedy,
        &[42u32, 99u32],
        30,
    );
    assert_eq!(bot_stats.total_matches, 4);
    assert_eq!(
        bot_stats.wins + bot_stats.losses + bot_stats.draws,
        4
    );
    let expected_bot_winrate = bot_stats.wins as f64 / 4.0;
    assert!((bot_stats.winrate - expected_bot_winrate).abs() < 1e-6);
}

#[test]
fn test_forensic_selector_clamping_and_numeraire_security() {
    let mut params = HeuristicParams {
        weight_area: 999.0, // Illegal Numeraire mutation
        weight_capture: -50.0,
        weight_deny_capture: 500.0,
        weight_voronoi_delta: 0.0,
        weight_safe_territory: 100.0,
        weight_compactness: 0.0,
        weight_spatial_matrix: 5.0,
        weight_frontier: 0.0,
        weight_opp_choke: 100.0,
        weight_bottleneck: -10.0,
    };

    assert!(!params.is_clamped());

    // Clamp must strictly restore numeraire and boundary corridors
    params.clamp();

    assert_eq!(params.weight_area, 50.0, "Numeraire must be strictly 50.0");
    assert_eq!(params.weight_capture, 70.0);
    assert_eq!(params.weight_deny_capture, 150.0);
    assert_eq!(params.weight_voronoi_delta, 15.0);
    assert_eq!(params.weight_safe_territory, 35.0);
    assert_eq!(params.weight_compactness, 5.0);
    assert_eq!(params.weight_spatial_matrix, 0.50);
    assert_eq!(params.weight_frontier, 10.0);
    assert_eq!(params.weight_opp_choke, 25.0);
    assert_eq!(params.weight_bottleneck, 5.0);
    assert!(params.is_clamped());
}

// ============================================================================
// 2. File I/O Forensic Verification: Genuine Persistence vs Shortcuts
// ============================================================================

#[test]
fn test_forensic_weights_file_io_authenticity() {
    let path_buf = unique_temp_path("forensic_weights", "json");
    let path = path_buf.to_str().unwrap();

    let mut weights = HeuristicParams::default();
    weights.weight_voronoi_delta = 31.415;
    weights.weight_safe_territory = 27.182;

    let matrix = generate_default_v2_matrix();
    let meta = SelectorMetadata {
        epoch: 42,
        winrate_vs_hard: 0.975,
        winrate_vs_greedy: 0.992,
        winrate_vs_champ: 0.650,
        avg_margin: 18.5,
        fitness: 102.0,
        extra: HashMap::new(),
    };

    // Save to disk
    save_to_file(path, &weights, &matrix, &meta).expect("Failed to write weights JSON");

    // Verify raw file content on disk
    let raw_content = fs::read_to_string(path).expect("Failed to read raw file from disk");
    assert!(
        raw_content.contains("\"WEIGHT_VORONOI_DELTA\": 31.415"),
        "Raw file does not contain formatted JSON field WEIGHT_VORONOI_DELTA"
    );
    assert!(
        raw_content.contains("\"epoch\": 42"),
        "Raw file does not contain epoch: 42"
    );

    // Modify file directly on disk to prove load_from_file reads genuine disk bytes
    let modified_content = raw_content.replace("31.415", "44.444");
    fs::write(path, modified_content).expect("Failed to write modified file");

    let (loaded_w, loaded_m, loaded_meta) =
        load_from_file(path).expect("Failed to load modified weights");

    assert_eq!(
        loaded_w.weight_voronoi_delta, 44.444,
        "load_from_file must read the actual modified value from disk"
    );
    assert_eq!(loaded_m, matrix);
    assert_eq!(loaded_meta.epoch, 42);

    // Test error handling: invalid matrix size
    let corrupted_content = raw_content.replace(
        "\"spatial_matrix\": [",
        "\"spatial_matrix\": [1.0, 2.0, ",
    );
    // This creates spatial_matrix with 602 elements
    let corrupt_path_buf = unique_temp_path("corrupt_weights", "json");
    let corrupt_path = corrupt_path_buf.to_str().unwrap();
    fs::write(corrupt_path, corrupted_content).unwrap();

    let err_res = load_from_file(corrupt_path);
    assert!(
        matches!(err_res, Err(SelectorError::InvalidFormat(_))),
        "Must return InvalidFormat on incorrect spatial_matrix length"
    );

    // Clean up
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(corrupt_path);
}

// ============================================================================
// 3. Tree Visualizer Forensic Verification: Algorithm & Dynamic Construction
// ============================================================================

#[test]
fn test_forensic_decision_tree_dynamic_construction_and_pruning() {
    let mut b = Board::new();
    // Setup midgame board with territory
    for r in 0..5 {
        for c in 0..10 {
            b.set(r, c, Player::Player1.cell_value());
        }
    }
    b.p1_score = 50;
    for r in 10..15 {
        for c in 30..40 {
            b.set(r, c, Player::Player2.cell_value());
        }
    }
    b.p2_score = 50;
    b.turn = 10;

    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    // 1. Build tree for Roll A: [2, 3]
    let tree_a = DecisionTree::build(&b, Player::Player2, 2, 3, &weights, &matrix, 4);

    // 2. Build tree for Roll B: [5, 6]
    let tree_b = DecisionTree::build(&b, Player::Player2, 5, 6, &weights, &matrix, 4);

    // Trees must be dynamically distinct based on dice
    assert_eq!(tree_a.dice, [2, 3]);
    assert_eq!(tree_b.dice, [5, 6]);
    assert_ne!(
        tree_a.evaluated_candidates[0].rect,
        tree_b.evaluated_candidates[0].rect,
        "Best candidate move must be different for different dice rolls"
    );
    assert_eq!(
        tree_a.evaluated_candidates[0].rect.h * tree_a.evaluated_candidates[0].rect.w,
        6,
        "Candidate for [2, 3] must have area 6"
    );
    assert_eq!(
        tree_b.evaluated_candidates[0].rect.h * tree_b.evaluated_candidates[0].rect.w,
        30,
        "Candidate for [5, 6] must have area 30"
    );

    // Verify beam pruning respects Top-4 budget
    assert!(tree_a.evaluated_candidates.len() <= 4);
    assert_eq!(
        tree_a.pruned_count + tree_a.evaluated_candidates.len(),
        tree_a.total_legal_candidates
    );

    // Verify exactly one candidate is selected as best
    let selected_count_a = tree_a
        .evaluated_candidates
        .iter()
        .filter(|c| c.selected)
        .count();
    assert_eq!(selected_count_a, 1, "Exactly one candidate must be marked selected");

    // Verify ranking order (1, 2, 3, 4)
    for (idx, cand) in tree_a.evaluated_candidates.iter().enumerate() {
        assert_eq!(cand.rank, idx + 1);
        assert!(cand.score.is_finite());
        assert!(cand.two_ply_counter_threat.final_adjusted_score.is_finite());
    }

    // Verify dynamic DOT export content
    let dot_a = tree_a.to_dot();
    let dot_b = tree_b.to_dot();

    assert_ne!(dot_a, dot_b, "DOT exports must be dynamically different for different rolls");
    assert!(dot_a.contains("Dice: [2, 3]"));
    assert!(dot_b.contains("Dice: [5, 6]"));

    // Verify DOT contains rank nodes, coordinates, and 2-ply probe edges
    let best_c = tree_a.evaluated_candidates.iter().find(|c| c.selected).unwrap();
    let best_coord_str = format!(
        "({}, {}, {}x{})",
        best_c.rect.r0, best_c.rect.c0, best_c.rect.h, best_c.rect.w
    );
    assert!(
        dot_a.contains(&best_coord_str),
        "DOT must contain candidate coordinates {}",
        best_coord_str
    );
    assert!(dot_a.contains("(BEST)"));
    assert!(dot_a.contains("Probe 2-Ply"));

    // Verify JSON export content
    let json_a = tree_a.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json_a).unwrap();
    assert_eq!(parsed["dice"][0], 2);
    assert_eq!(parsed["dice"][1], 3);
    assert_eq!(parsed["totalLegalCandidates"], tree_a.total_legal_candidates);
    assert_eq!(parsed["prunedCount"], tree_a.pruned_count);

    // Syntax validators
    assert!(DecisionTree::validate_dot_syntax(&dot_a).is_ok());
    assert!(DecisionTree::validate_json_syntax(&json_a).is_ok());
}

#[test]
fn test_forensic_narrow_chokepoints_detection_and_penalty() {
    let mut b = Board::new();

    // Construct a vertical narrow chokepoint at (7, 15):
    // Enemy P1 cell at (6, 15) and (8, 15)
    b.set(6, 15, Player::Player1.cell_value());
    b.set(8, 15, Player::Player1.cell_value());
    b.p1_score = 2;

    let chokepoints = detect_narrow_chokepoints(&b, Player::Player2);
    let target_idx = 7 * BOARD_WIDTH + 15;

    assert!(
        chokepoints.contains(&target_idx),
        "Chokepoints vector must detect pinched cell (7, 15)"
    );

    // Move entering the chokepoint
    let risky_mv = Move::new(7, 15, 1, 1);
    let (count, penalty) = compute_chokepoint_penalty(&b, &risky_mv, &chokepoints);

    assert_eq!(count, 1);
    assert_eq!(penalty, 8.0, "Penalty must be count * 8.0 = 8.0");

    // Move far away at (0, 0)
    let safe_mv = Move::new(0, 0, 2, 2);
    let (safe_count, safe_penalty) = compute_chokepoint_penalty(&b, &safe_mv, &chokepoints);

    assert_eq!(safe_count, 0);
    assert_eq!(safe_penalty, 0.0);
}

#[test]
fn test_forensic_syntax_validators_reject_malformed() {
    // Malformed DOT tests
    assert!(DecisionTree::validate_dot_syntax("graph G {}").is_err(), "Must start with digraph");
    assert!(DecisionTree::validate_dot_syntax("digraph G { a -> b;").is_err(), "Must balance braces");
    assert!(DecisionTree::validate_dot_syntax("digraph G { a [label=\"unclosed]; }").is_err(), "Must check unclosed quotes");
    assert!(DecisionTree::validate_dot_syntax("digraph G { a [label=\"ok\"; }").is_err(), "Must check unclosed brackets");

    // Valid DOT
    assert!(DecisionTree::validate_dot_syntax("digraph DecisionTree {\n  root -> c1 [label=\"Rank 1\"];\n}\n").is_ok());

    // Malformed JSON tests
    assert!(DecisionTree::validate_json_syntax("{ not valid json").is_err());
    assert!(DecisionTree::validate_json_syntax("[]").is_err(), "Must be root object");
    assert!(DecisionTree::validate_json_syntax(r#"{"turn": 1}"#).is_err(), "Must have required fields");

    // Valid JSON
    let valid_json = r#"{
        "turn": 1,
        "player": 2,
        "dice": [3, 4],
        "phase": "Midgame",
        "boardState": {"p1Count": 10, "p2Count": 10, "emptyCount": 580},
        "totalLegalCandidates": 1,
        "prunedCount": 0,
        "evaluatedCandidates": [
            {"rank": 1, "rect": {"r0": 0, "c0": 0, "h": 1, "w": 1}, "score": 10.0}
        ]
    }"#;
    assert!(DecisionTree::validate_json_syntax(valid_json).is_ok());
}
