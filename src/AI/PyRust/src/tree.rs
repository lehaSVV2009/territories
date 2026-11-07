//! Decision search tree representation and visualization export.
//!
//! Provides:
//! - Data structures: `DecisionTree`, `TreeNode`, `MoveCandidate`, `SearchPhase`, `MoveBreakdown`, `TwoPlyThreat`.
//! - 1-ply candidate evaluation with 9-component breakdown and narrow chokepoint distance penalty.
//! - Candidate beam pruning limiting 2-ply counter-attack threat search to Top-4 or Top-5 candidates (user time budget).
//! - Graphviz DOT (`to_dot`) and structured JSON (`to_json`) export functions.
//! - Robust syntax validation for DOT and JSON schemas.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::game::{
    Board, HeuristicWeights, Move, Player, TOTAL_CELLS,
    BOARD_HEIGHT, BOARD_WIDTH, EMPTY,
    can_close_loop, check_enclosed_cells, compute_endgame_gap_penalty,
    compute_voronoi_influence_v2, evaluate_counter_threat,
};

/// High-level game phase governing heuristic weights and branch evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchPhase {
    Opening,
    Midgame,
    Endgame,
}

impl SearchPhase {
    pub fn from_empty_cells(empty_cells: usize) -> Self {
        if empty_cells > 490 {
            SearchPhase::Opening
        } else if empty_cells < 180 {
            SearchPhase::Endgame
        } else {
            SearchPhase::Midgame
        }
    }
}

/// Geometric coordinates of a rectangle move on the 15x40 grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RectCoords {
    pub r0: usize,
    pub c0: usize,
    pub h: usize,
    pub w: usize,
}

impl From<&Move> for RectCoords {
    fn from(m: &Move) -> Self {
        Self {
            r0: m.r,
            c0: m.c,
            h: m.h,
            w: m.w,
        }
    }
}

impl From<Move> for RectCoords {
    fn from(m: Move) -> Self {
        RectCoords::from(&m)
    }
}

/// Comprehensive breakdown of individual evaluation score components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveBreakdown {
    pub area: f64,
    pub compactness: f64,
    #[serde(rename = "loopCapture")]
    pub loop_capture: f64,
    #[serde(rename = "loopDenial")]
    pub loop_denial: f64,
    pub voronoi: f64,
    #[serde(rename = "voronoiDelta")]
    pub voronoi_delta: f64,
    #[serde(rename = "safeTerritory")]
    pub safe_territory: f64,
    #[serde(rename = "spatialMatrix")]
    pub spatial_matrix: f64,
    pub frontier: f64,
    #[serde(rename = "frontierAdvance")]
    pub frontier_advance: f64,
    pub bottleneck: f64,
    pub guillotine: f64,
    #[serde(rename = "wallCork")]
    pub wall_cork: f64,
    #[serde(rename = "pocketTrap")]
    pub pocket_trap: f64,
    #[serde(rename = "oppChoke")]
    pub opp_choke: f64,
    #[serde(rename = "gapPenalty")]
    pub gap_penalty: f64,
    #[serde(rename = "chokepointPenalty")]
    pub chokepoint_penalty: f64,
}

/// 2-Ply counter-threat lookahead probing dangerous opponent rolls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwoPlyThreat {
    #[serde(rename = "maxOppCapture")]
    pub max_opp_capture: usize,
    #[serde(rename = "testedDice")]
    pub tested_dice: Vec<[usize; 2]>,
    pub penalty: f64,
    #[serde(rename = "finalAdjustedScore")]
    pub final_adjusted_score: f64,
}

/// Evaluated candidate move with ranking, breakdown, and 2-ply threat results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveCandidate {
    pub rank: usize,
    pub rect: RectCoords,
    pub score: f64,
    pub breakdown: MoveBreakdown,
    #[serde(rename = "twoPlyCounterThreat")]
    pub two_ply_counter_threat: TwoPlyThreat,
    pub selected: bool,
}

/// High-level board state statistics at the search root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardSummary {
    #[serde(rename = "p1Count")]
    pub p1_count: u16,
    #[serde(rename = "p2Count")]
    pub p2_count: u16,
    #[serde(rename = "emptyCount")]
    pub empty_count: usize,
}

/// Type of node in the decision tree visualization graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeNodeType {
    Root,
    Candidate,
    OpponentThreat,
    PrunedSummary,
}

/// Generic node representation for search tree graphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub node_type: TreeNodeType,
    pub label: String,
    pub score: Option<f64>,
    pub children: Vec<TreeNode>,
    pub attributes: HashMap<String, String>,
}

/// Complete decision search tree containing candidate moves, pruning stats, and graph nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTree {
    pub turn: usize,
    pub player: u8,
    pub dice: [usize; 2],
    pub phase: SearchPhase,
    #[serde(rename = "boardState")]
    pub board_state: BoardSummary,
    #[serde(rename = "totalLegalCandidates")]
    pub total_legal_candidates: usize,
    #[serde(rename = "prunedCount")]
    pub pruned_count: usize,
    #[serde(rename = "evaluatedCandidates")]
    pub evaluated_candidates: Vec<MoveCandidate>,
    pub root: Option<TreeNode>,
}

/// Detects empty cells that lie in a narrow chokepoint (1-2 cells wide) between enemy cells or border walls.
pub fn detect_narrow_chokepoints(board: &Board, target_player: Player) -> Vec<usize> {
    let enemy_val = target_player.opponent().cell_value();
    let mut chokepoints = Vec::new();

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let idx = row_offset + c;
            if board.cells[idx] != EMPTY {
                continue;
            }

            let mut is_choked = false;

            // 1. Vertical pinch: enemy above and enemy below within distance 1..2
            let enemy_above = (r >= 1 && board.get(r - 1, c) == enemy_val)
                || (r >= 2 && board.get(r - 2, c) == enemy_val && board.get(r - 1, c) == EMPTY)
                || (r == 0);
            let enemy_below = (r + 1 < BOARD_HEIGHT && board.get(r + 1, c) == enemy_val)
                || (r + 2 < BOARD_HEIGHT && board.get(r + 2, c) == enemy_val && board.get(r + 1, c) == EMPTY)
                || (r == BOARD_HEIGHT - 1);

            if enemy_above && enemy_below && (
                (r >= 1 && board.get(r - 1, c) == enemy_val)
                    || (r + 1 < BOARD_HEIGHT && board.get(r + 1, c) == enemy_val)
            ) {
                is_choked = true;
            }

            // 2. Horizontal pinch: enemy left and enemy right within distance 1..2
            if !is_choked {
                let enemy_left = (c >= 1 && board.get(r, c - 1) == enemy_val)
                    || (c >= 2 && board.get(r, c - 2) == enemy_val && board.get(r, c - 1) == EMPTY)
                    || (c == 0);
                let enemy_right = (c + 1 < BOARD_WIDTH && board.get(r, c + 1) == enemy_val)
                    || (c + 2 < BOARD_WIDTH && board.get(r, c + 2) == enemy_val && board.get(r, c + 1) == EMPTY)
                    || (c == BOARD_WIDTH - 1);

                if enemy_left && enemy_right && (
                    (c >= 1 && board.get(r, c - 1) == enemy_val)
                        || (c + 1 < BOARD_WIDTH && board.get(r, c + 1) == enemy_val)
                ) {
                    is_choked = true;
                }
            }

            if is_choked {
                chokepoints.push(idx);
            }
        }
    }

    chokepoints
}

/// Computes the narrow chokepoint Voronoi distance penalty for a candidate move.
///
/// If a candidate enters or borders a narrow corridor between enemy cells,
/// it incurs a distance penalty reflecting severe encirclement risk.
pub fn compute_chokepoint_penalty(
    _board: &Board,
    mv: &Move,
    chokepoint_cells: &[usize],
) -> (usize, f64) {
    if chokepoint_cells.is_empty() {
        return (0, 0.0);
    }

    let mut count = 0;
    let r0 = mv.r;
    let c0 = mv.c;
    let r1 = r0 + mv.h - 1;
    let c1 = c0 + mv.w - 1;

    for &idx in chokepoint_cells {
        let cr = idx / BOARD_WIDTH;
        let cc = idx % BOARD_WIDTH;
        let in_piece = cr >= r0 && cr <= r1 && cc >= c0 && cc <= c1;
        let borders_piece = (cr + 1 >= r0 && cr <= r1 + 1) && (cc + 1 >= c0 && cc <= c1 + 1);

        if in_piece || borders_piece {
            count += 1;
        }
    }

    let penalty = (count as f64) * 8.0;
    (count, penalty)
}

/// Evaluates all components of the 9-component V2 formula plus chokepoint penalty for a candidate move.
pub fn evaluate_candidate_breakdown(
    board: &Board,
    mv: &Move,
    params: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
    chokepoint_cells: &[usize],
) -> (f64, MoveBreakdown) {
    let r0 = mv.r;
    let c0 = mv.c;
    let h = mv.h;
    let w = mv.w;
    let area = h * w;
    let min_dim = if h < w { h } else { w };

    // Board scan
    let mut empty_cells = 0;
    let mut p1_max_col = 0;
    let mut p1_min_col = BOARD_WIDTH - 1;
    let mut p1_max_row = 0;
    let mut p1_col_at_max_row = 0;
    let mut p1_min_row = BOARD_HEIGHT - 1;
    let mut p1_col_at_min_row = 0;
    let mut p2_min_col = BOARD_WIDTH - 1;

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let cell = board.cells[row_offset + c];
            if cell == EMPTY {
                empty_cells += 1;
            } else if cell == crate::game::PLAYER_1 {
                if c > p1_max_col {
                    p1_max_col = c;
                }
                if c < p1_min_col {
                    p1_min_col = c;
                }
                if r > p1_max_row {
                    p1_max_row = r;
                    p1_col_at_max_row = c;
                }
                if r < p1_min_row {
                    p1_min_row = r;
                    p1_col_at_min_row = c;
                }
            } else if cell == crate::game::PLAYER_2 && c < p2_min_col {
                p2_min_col = c;
            }
        }
    }

    let is_opening = empty_cells > 490;
    let is_endgame = empty_cells < 180;
    let has_contact = p2_min_col <= p1_max_col + 3;

    let p1_touches_bottom_wall = p1_max_row == BOARD_HEIGHT - 1;
    let p1_touches_top_wall = p1_min_row == 0;
    let p1_threatens_bottom_wall = !p1_touches_bottom_wall && p1_max_row >= 8 && p1_col_at_max_row >= 4;
    let p1_threatens_top_wall = !p1_touches_top_wall && p1_min_row <= 6 && p1_col_at_min_row >= 4;

    // 1. Base Area
    let score_area = (area as f64) * params.weight_area;

    // 2. Compactness
    let mut score_compactness = 0.0;
    if is_opening && !has_contact {
        if w > h {
            score_compactness = ((w - h) as f64) * 4.0;
        }
    } else if min_dim >= 2 {
        score_compactness = params.weight_compactness;
    } else if min_dim == 1 && area > 2 {
        score_compactness = -params.weight_compactness * 1.5;
    }

    // 3. Loop Enclosure Check
    let mut score_loop_capture = 0.0;
    if can_close_loop(board, r0, c0, h, w, Player::Player2) {
        let captured = check_enclosed_cells(board, r0, c0, h, w, Player::Player2);
        if captured > 0 {
            score_loop_capture = (captured as f64) * params.weight_capture;
        }
    }

    // 4. Opponent Denial
    let mut score_loop_denial = 0.0;
    if can_close_loop(board, r0, c0, h, w, Player::Player1) {
        let opp_would_capture = check_enclosed_cells(board, r0, c0, h, w, Player::Player1);
        if opp_would_capture > 0 {
            score_loop_denial = (opp_would_capture as f64) * params.weight_deny_capture;
        }
    }

    // 5. Voronoi Field Evaluation
    let field = compute_voronoi_influence_v2(board, r0, c0, h, w, Player::Player2, matrix);
    let score_voronoi_delta = ((field.p2_territory - field.p1_territory) as f64) * params.weight_voronoi_delta;
    let score_safe_territory = ((field.p2_safe_territory - field.p1_safe_territory) as f64) * params.weight_safe_territory;
    let score_spatial_matrix = field.spatial_score * params.weight_spatial_matrix;

    // 6. Frontier Advancement
    let center_row_dist = ((r0 as f64 + (h as f64) * 0.5) - 7.0).abs() / 7.0;
    let vertical_center_bonus = 1.0 - center_row_dist;
    let score_frontier = if is_opening {
        ((((39.0 - (c0 as f64)) / 39.0) * 1.4) + vertical_center_bonus) * params.weight_frontier
    } else if !is_endgame {
        (((39.0 - (c0 as f64)) / 39.0) + vertical_center_bonus * 0.5) * params.weight_frontier
    } else {
        ((39.0 - (c0 as f64)) / 39.0) * (params.weight_frontier * 0.2)
    };

    // 7. Opponent Choke
    let score_opp_choke = if field.opp_choked_cells > 0 {
        (field.opp_choked_cells as f64) * params.weight_opp_choke
    } else {
        0.0
    };

    // 8. Vertical Guillotine
    let score_guillotine = if has_contact && c0 >= p1_max_col.saturating_sub(2) && c0 <= p1_max_col + 3 && h > w {
        (h as f64) * params.weight_guillotine
    } else {
        0.0
    };

    // 9. Bottleneck anchor at border walls
    let touches_wall = r0 == 0 || r0 + h == BOARD_HEIGHT;
    let score_bottleneck = if touches_wall && c0 < 25 && h >= 2 {
        params.weight_bottleneck
    } else {
        0.0
    };

    // 10. Wall Cork
    let mut score_wall_cork = 0.0;
    if p1_threatens_bottom_wall && r0 + h == BOARD_HEIGHT {
        let min_c = p1_col_at_max_row.saturating_sub(1);
        let max_c = p1_col_at_max_row + 5;
        if c0 >= min_c && c0 <= max_c {
            score_wall_cork += params.weight_wall_cork;
        }
    }
    if p1_threatens_top_wall && r0 == 0 {
        let min_c = p1_col_at_min_row.saturating_sub(1);
        let max_c = p1_col_at_min_row + 5;
        if c0 >= min_c && c0 <= max_c {
            score_wall_cork += params.weight_wall_cork;
        }
    }

    // 11. Anti-Pocket Trap
    let mut score_pocket_trap = 0.0;
    if p1_threatens_bottom_wall && c0 + w <= p1_col_at_max_row && r0 >= 8 {
        score_pocket_trap -= params.weight_pocket_trap;
    }
    if p1_threatens_top_wall && c0 + w <= p1_col_at_min_row && r0 + h <= 6 {
        score_pocket_trap -= params.weight_pocket_trap;
    }

    // 12. Anti-Fragmentation Gap Penalty
    let mut score_gap_penalty = 0.0;
    if is_endgame {
        let dead_gaps = compute_endgame_gap_penalty(board, r0, c0, h, w);
        if dead_gaps > 0 {
            score_gap_penalty -= (dead_gaps as f64) * params.weight_gap_penalty;
        }
    }

    // 13. Narrow Chokepoint Voronoi Distance Penalty
    let (_choke_count, choke_penalty) = compute_chokepoint_penalty(board, mv, chokepoint_cells);

    let total_score = score_area
        + score_compactness
        + score_loop_capture
        + score_loop_denial
        + score_voronoi_delta
        + score_safe_territory
        + score_spatial_matrix
        + score_frontier
        + score_opp_choke
        + score_guillotine
        + score_bottleneck
        + score_wall_cork
        + score_pocket_trap
        + score_gap_penalty
        - choke_penalty;

    let breakdown = MoveBreakdown {
        area: score_area,
        compactness: score_compactness,
        loop_capture: score_loop_capture,
        loop_denial: score_loop_denial,
        voronoi: score_voronoi_delta,
        voronoi_delta: score_voronoi_delta,
        safe_territory: score_safe_territory,
        spatial_matrix: score_spatial_matrix,
        frontier: score_frontier,
        frontier_advance: score_frontier,
        bottleneck: score_bottleneck,
        guillotine: score_guillotine,
        wall_cork: score_wall_cork,
        pocket_trap: score_pocket_trap,
        opp_choke: score_opp_choke,
        gap_penalty: score_gap_penalty,
        chokepoint_penalty: choke_penalty,
    };

    (total_score, breakdown)
}

impl DecisionTree {
    /// Constructs a decision search tree evaluating 1-ply candidates and probing 2-ply counter-threats
    /// for Top-K candidates (budgeted at Top-4 or Top-5 for maximum throughput).
    pub fn build(
        board: &Board,
        player: Player,
        d1: usize,
        d2: usize,
        weights: &HeuristicWeights,
        matrix: &[f64; TOTAL_CELLS],
        top_k: usize,
    ) -> Self {
        let empty_count = board.empty_cells_count();
        let phase = SearchPhase::from_empty_cells(empty_count);
        let player_val = player.cell_value();

        let board_summary = BoardSummary {
            p1_count: board.p1_score,
            p2_count: board.p2_score,
            empty_count,
        };

        // Determine orientation and perspective
        let is_p1 = player == Player::Player1;
        let eval_board = if is_p1 {
            board.rotate_board_180()
        } else {
            board.clone()
        };

        // Legal candidates in Player 2 perspective
        let legal_moves = eval_board.generate_legal_moves(Player::Player2, d1, d2);
        let total_candidates = legal_moves.len();

        if total_candidates == 0 {
            let root = TreeNode {
                id: "root".to_string(),
                node_type: TreeNodeType::Root,
                label: format!("Turn {} | Pass (No Legal Moves)", board.turn),
                score: None,
                children: Vec::new(),
                attributes: HashMap::new(),
            };
            return Self {
                turn: board.turn,
                player: player_val,
                dice: [d1, d2],
                phase,
                board_state: board_summary,
                total_legal_candidates: 0,
                pruned_count: 0,
                evaluated_candidates: Vec::new(),
                root: Some(root),
            };
        }

        // Narrow chokepoints
        let chokepoint_cells = detect_narrow_chokepoints(&eval_board, Player::Player2);

        // Score all candidates
        let mut scored_cands: Vec<(Move, f64, MoveBreakdown)> = legal_moves
            .into_iter()
            .map(|mv| {
                let (score, breakdown) =
                    evaluate_candidate_breakdown(&eval_board, &mv, weights, matrix, &chokepoint_cells);
                (mv, score, breakdown)
            })
            .collect();

        // Stable sort descending by score
        scored_cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Beam pruning: restrict 2-ply counter-attack threat to Top-K (4 or 5)
        let beam_size = top_k.clamp(4, 5).min(scored_cands.len());
        let pruned_count = scored_cands.len().saturating_sub(beam_size);

        // Check contact condition for 2-ply evaluation
        let mut p1_max_col = 0;
        let mut p2_min_col = BOARD_WIDTH;
        for r in 0..BOARD_HEIGHT {
            let row_offset = r * BOARD_WIDTH;
            for c in 0..BOARD_WIDTH {
                let cell = eval_board.cells[row_offset + c];
                if cell == crate::game::PLAYER_1 && c > p1_max_col {
                    p1_max_col = c;
                } else if cell == crate::game::PLAYER_2 && c < p2_min_col {
                    p2_min_col = c;
                }
            }
        }
        let has_contact = p2_min_col <= p1_max_col + 3;
        let is_opening = empty_count > 490;

        let mut evaluated_candidates: Vec<MoveCandidate> = Vec::with_capacity(beam_size);
        let mut best_adjusted_score = -f64::INFINITY;
        let mut best_cand_idx = 0;

        for (rank_idx, (eval_mv, score, breakdown)) in scored_cands.into_iter().take(beam_size).enumerate() {
            let rank = rank_idx + 1;

            // 2-Ply counter-threat lookahead probing dangerous opponent rolls
            let opp_threat = if has_contact || !is_opening {
                evaluate_counter_threat(&eval_board, eval_mv.r, eval_mv.c, eval_mv.h, eval_mv.w)
            } else {
                0
            };

            let penalty = if opp_threat > 0 {
                (opp_threat as f64) * (weights.weight_deny_capture * 0.8)
            } else {
                0.0
            };

            let final_adjusted_score = score - penalty;

            if final_adjusted_score > best_adjusted_score {
                best_adjusted_score = final_adjusted_score;
                best_cand_idx = rank_idx;
            }

            // Map coordinates back if P1
            let real_move = if is_p1 {
                Board::inverse_move_180(&eval_mv)
            } else {
                eval_mv
            };

            let threat_info = TwoPlyThreat {
                max_opp_capture: opp_threat,
                tested_dice: vec![[1, 5], [2, 4], [3, 3], [4, 2]],
                penalty,
                final_adjusted_score,
            };

            evaluated_candidates.push(MoveCandidate {
                rank,
                rect: RectCoords::from(real_move),
                score,
                breakdown,
                two_ply_counter_threat: threat_info,
                selected: false,
            });
        }

        if !evaluated_candidates.is_empty() {
            evaluated_candidates[best_cand_idx].selected = true;
        }

        // Construct graph node hierarchy
        let mut root_node = TreeNode {
            id: "root".to_string(),
            node_type: TreeNodeType::Root,
            label: format!(
                "Turn {} | P{} to Move\\nDice: [{}, {}]\\nEmpty: {}",
                board.turn, player_val, d1, d2, empty_count
            ),
            score: None,
            children: Vec::new(),
            attributes: HashMap::new(),
        };

        for cand in &evaluated_candidates {
            let mut cand_node = TreeNode {
                id: format!("c{}", cand.rank),
                node_type: TreeNodeType::Candidate,
                label: format!(
                    "Rank #{}{}\\n({}, {}, {}x{})\\nScore: {:.1}",
                    cand.rank,
                    if cand.selected { " (BEST)" } else { "" },
                    cand.rect.r0,
                    cand.rect.c0,
                    cand.rect.h,
                    cand.rect.w,
                    cand.score
                ),
                score: Some(cand.score),
                children: Vec::new(),
                attributes: HashMap::new(),
            };

            // Child threat node
            let threat_node = TreeNode {
                id: format!("c{}_opp", cand.rank),
                node_type: TreeNodeType::OpponentThreat,
                label: format!(
                    "Max Threat: {} cells\\nPenalty: -{:.1}\\nAdj Score: {:.1}",
                    cand.two_ply_counter_threat.max_opp_capture,
                    cand.two_ply_counter_threat.penalty,
                    cand.two_ply_counter_threat.final_adjusted_score
                ),
                score: Some(cand.two_ply_counter_threat.final_adjusted_score),
                children: Vec::new(),
                attributes: HashMap::new(),
            };
            cand_node.children.push(threat_node);
            root_node.children.push(cand_node);
        }

        if pruned_count > 0 {
            let pruned_node = TreeNode {
                id: "pruned".to_string(),
                node_type: TreeNodeType::PrunedSummary,
                label: format!("Pruned: {} Candidates\\n(Time Budget: Top {})", pruned_count, beam_size),
                score: None,
                children: Vec::new(),
                attributes: HashMap::new(),
            };
            root_node.children.push(pruned_node);
        }

        Self {
            turn: board.turn,
            player: player_val,
            dice: [d1, d2],
            phase,
            board_state: board_summary,
            total_legal_candidates: total_candidates,
            pruned_count,
            evaluated_candidates,
            root: Some(root_node),
        }
    }

    /// Exports the search tree to Graphviz .dot syntax.
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph DecisionTree {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=\"filled,rounded\", fontname=\"Helvetica\", fontsize=10];\n");
        dot.push_str("  edge [fontname=\"Helvetica\", fontsize=9];\n\n");

        // Root
        let root_label = if self.total_legal_candidates == 0 {
            format!(
                "Turn {} | Pass (No Legal Moves)\\nDice: [{}, {}]\\nEmpty: {}",
                self.turn, self.dice[0], self.dice[1], self.board_state.empty_count
            )
        } else {
            format!(
                "Turn {} | P{} to Move\\nDice: [{}, {}]\\nEmpty: {}",
                self.turn, self.player, self.dice[0], self.dice[1], self.board_state.empty_count
            )
        };
        dot.push_str(&format!(
            "  root [label=\"{}\", fillcolor=\"#edf2f7\", color=\"#4a5568\"];\n",
            root_label
        ));

        // Candidates and threat probes
        for c in &self.evaluated_candidates {
            let cid = format!("c{}", c.rank);
            let fill = if c.selected {
                "#c6f6d5" // Green highlight for best candidate
            } else if c.rank <= 3 {
                "#feebc8" // Warm yellow for top runner-ups
            } else {
                "#fed7d7" // Subtle reddish for lower candidates
            };
            let color = if c.selected { "#22543d" } else { "#744210" };

            dot.push_str(&format!(
                "  {} [label=\"Rank #{}{}\\n({}, {}, {}x{})\\nScore: {:.1}\", fillcolor=\"{}\", color=\"{}\"];\n",
                cid,
                c.rank,
                if c.selected { " (BEST)" } else { "" },
                c.rect.r0,
                c.rect.c0,
                c.rect.h,
                c.rect.w,
                c.score,
                fill,
                color
            ));
            dot.push_str(&format!("  root -> {} [label=\"Rank {}\"];\n", cid, c.rank));

            // Opponent Threat Node
            let opp_id = format!("{}_opp", cid);
            let opp_threat = &c.two_ply_counter_threat;
            dot.push_str(&format!(
                "  {} [label=\"Opp Threat: {} cells\\nPenalty: -{:.1}\\nAdj: {:.1}\", fillcolor=\"#ebf8ff\", color=\"#2b6cb0\"];\n",
                opp_id, opp_threat.max_opp_capture, opp_threat.penalty, opp_threat.final_adjusted_score
            ));
            dot.push_str(&format!("  {} -> {} [label=\"Probe 2-Ply\"];\n", cid, opp_id));
        }

        // Pruned summary node if applicable
        if self.pruned_count > 0 {
            dot.push_str(&format!(
                "  pruned [label=\"Pruned: {} Candidates\\n(Budget Top {})\", fillcolor=\"#edf2f7\", color=\"#a0aec0\", style=\"dashed,rounded\"];\n",
                self.pruned_count, self.evaluated_candidates.len()
            ));
            dot.push_str("  root -> pruned [label=\"Beam Cut\", style=\"dashed\"];\n");
        }

        dot.push_str("}\n");
        dot
    }

    /// Exports the search tree to structured JSON format.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Validates that a Graphviz DOT string has balanced braces, valid identifiers, and valid attributes.
    pub fn validate_dot_syntax(dot: &str) -> Result<(), String> {
        let trimmed = dot.trim();
        if !trimmed.starts_with("digraph") {
            return Err("DOT string must start with 'digraph'".to_string());
        }
        if !trimmed.ends_with('}') {
            return Err("DOT string must terminate with '}'".to_string());
        }

        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_quotes = false;
        let mut prev_char = '\0';

        for ch in trimmed.chars() {
            if ch == '"' && prev_char != '\\' {
                in_quotes = !in_quotes;
            } else if !in_quotes {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count < 0 {
                            return Err("Unbalanced closing brace '}' in DOT".to_string());
                        }
                    }
                    '[' => bracket_count += 1,
                    ']' => {
                        bracket_count -= 1;
                        if bracket_count < 0 {
                            return Err("Unbalanced closing bracket ']' in DOT".to_string());
                        }
                    }
                    _ => {}
                }
            }
            prev_char = ch;
        }

        if in_quotes {
            return Err("Unclosed quotation mark in DOT string".to_string());
        }
        if brace_count != 0 {
            return Err("Unbalanced braces in DOT string".to_string());
        }
        if bracket_count != 0 {
            return Err("Unbalanced attribute brackets in DOT string".to_string());
        }

        Ok(())
    }

    /// Validates that a JSON string parses successfully and conforms to the decision tree schema.
    pub fn validate_json_syntax(json_str: &str) -> Result<(), String> {
        let parsed: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.is_object() {
            return Err("Expected JSON root to be an object".to_string());
        }

        for required in &["turn", "player", "dice", "boardState", "evaluatedCandidates"] {
            if parsed.get(required).is_none() {
                return Err(format!("Missing required JSON field: '{}'", required));
            }
        }

        if let Some(cands) = parsed.get("evaluatedCandidates").and_then(|v| v.as_array()) {
            for cand in cands {
                if cand.get("rank").is_none() || cand.get("rect").is_none() || cand.get("score").is_none() {
                    return Err("Candidate entry missing rank, rect, or score".to_string());
                }
            }
        }

        Ok(())
    }
}

/// Standalone export helper: converts decision tree to DOT string.
pub fn export_to_dot(tree: &DecisionTree) -> String {
    tree.to_dot()
}

/// Standalone export helper: converts decision tree to JSON string.
pub fn export_to_json(tree: &DecisionTree) -> String {
    tree.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::generate_default_v2_matrix;

    fn make_test_board() -> Board {
        let mut b = Board::new();
        // Give P1 starting territory
        for r in 0..4 {
            for c in 0..5 {
                b.set(r, c, crate::game::PLAYER_1);
            }
        }
        b.p1_score = 20;

        // Give P2 starting territory
        for r in 11..15 {
            for c in 35..40 {
                b.set(r, c, crate::game::PLAYER_2);
            }
        }
        b.p2_score = 20;
        b.turn = 12;
        b
    }

    #[test]
    fn test_decision_tree_construction_and_pruning() {
        let b = make_test_board();
        let weights = HeuristicWeights::default();
        let matrix = generate_default_v2_matrix();

        let tree = DecisionTree::build(&b, Player::Player2, 3, 4, &weights, &matrix, 4);

        assert_eq!(tree.turn, 12);
        assert_eq!(tree.player, 2);
        assert_eq!(tree.dice, [3, 4]);
        assert!(tree.total_legal_candidates > 0);
        assert!(tree.evaluated_candidates.len() <= 4);
        assert!(tree.pruned_count + tree.evaluated_candidates.len() == tree.total_legal_candidates);

        // Check ranking
        for (i, c) in tree.evaluated_candidates.iter().enumerate() {
            assert_eq!(c.rank, i + 1);
        }

        // At least one candidate must be selected
        assert!(tree.evaluated_candidates.iter().any(|c| c.selected));
    }

    #[test]
    fn test_narrow_chokepoint_detection_and_penalty() {
        let mut b = Board::new();
        // Place P1 cells creating a 1-cell corridor at row 5, col 20
        b.set(4, 20, crate::game::PLAYER_1);
        b.set(6, 20, crate::game::PLAYER_1);
        b.p1_score = 2;

        let chokepoints = detect_narrow_chokepoints(&b, Player::Player2);
        let chokepoint_idx = 5 * BOARD_WIDTH + 20;

        assert!(chokepoints.contains(&chokepoint_idx));

        // Move entering this chokepoint
        let mv = Move::new(5, 19, 1, 3);
        let (count, penalty) = compute_chokepoint_penalty(&b, &mv, &chokepoints);

        assert!(count >= 1);
        assert!(penalty > 0.0);
    }

    #[test]
    fn test_to_dot_valid_graphviz_syntax() {
        let b = make_test_board();
        let weights = HeuristicWeights::default();
        let matrix = generate_default_v2_matrix();

        let tree = DecisionTree::build(&b, Player::Player2, 2, 3, &weights, &matrix, 4);
        let dot = tree.to_dot();

        assert!(dot.starts_with("digraph DecisionTree {"));
        assert!(dot.ends_with("}\n"));
        assert!(dot.contains("root -> c1"));

        // Validation test
        let val_res = DecisionTree::validate_dot_syntax(&dot);
        assert!(val_res.is_ok(), "DOT syntax invalid: {:?}", val_res.err());
    }

    #[test]
    fn test_to_json_valid_schema_and_serde_parse() {
        let b = make_test_board();
        let weights = HeuristicWeights::default();
        let matrix = generate_default_v2_matrix();

        let tree = DecisionTree::build(&b, Player::Player2, 3, 3, &weights, &matrix, 4);
        let json_str = tree.to_json();

        // Validation test
        let val_res = DecisionTree::validate_json_syntax(&json_str);
        assert!(val_res.is_ok(), "JSON syntax invalid: {:?}", val_res.err());

        // Parse via serde_json to ensure full deserialize capability
        let parsed: DecisionTree = serde_json::from_str(&json_str).expect("Failed to deserialize tree JSON");
        assert_eq!(parsed.turn, 12);
        assert_eq!(parsed.player, 2);
        assert!(!parsed.evaluated_candidates.is_empty());
    }

    #[test]
    fn test_decision_tree_no_legal_moves_pass() {
        let b = Board::new(); // empty board, but looking for moves not covering corner
        // Turn 2 for P2 requires covering (14, 39)
        // If we request a die roll of (16, 16) - impossible - no moves
        let weights = HeuristicWeights::default();
        let matrix = generate_default_v2_matrix();

        let tree = DecisionTree::build(&b, Player::Player2, 16, 16, &weights, &matrix, 4);
        assert_eq!(tree.total_legal_candidates, 0);
        assert_eq!(tree.evaluated_candidates.len(), 0);

        let dot = tree.to_dot();
        assert!(dot.contains("No Legal Moves"));
        assert!(DecisionTree::validate_dot_syntax(&dot).is_ok());

        let json_str = tree.to_json();
        assert!(DecisionTree::validate_json_syntax(&json_str).is_ok());
    }
}
