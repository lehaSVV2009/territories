//! Empirical Challenger Verification Suite for Milestone M2:
//! - ModelSelector (`src/selector.rs`)
//! - Decision Tree Visualizer (`src/tree.rs`)
//!
//! Executes rigorous empirical tests:
//! 1. Decision tree generation across 50 diverse board states (empty, opening, midgame, endgame, full).
//! 2. Graphviz DOT export validation (balanced braces, edge endpoint validation, node declaration syntax).
//! 3. JSON export validation (serde_json parsing, schema conformance, roundtrip deserialization).
//! 4. ModelSelector across 100 CRN pairs (200 matches) verifying bit-exact determinism and champion promotion.
//! 5. Decision tree construction throughput benchmark (>1,000 trees/sec budget).

use pyrust::game::*;
use pyrust::selector::*;
use pyrust::tree::*;
use std::collections::HashSet;
use std::time::Instant;

// ============================================================================
// Board Generators for 50 Diverse States
// ============================================================================

/// Generates 50 distinct, structurally diverse board states across all game phases.
fn generate_50_diverse_boards() -> Vec<(String, Board, usize, usize, Player)> {
    let mut states = Vec::with_capacity(50);

    // 1-5: Empty & First Turn States
    // ------------------------------------------------------------------------
    // Board 0: Completely empty board, P1 to move (d1=3, d2=4)
    {
        let b = Board::new();
        states.push(("Empty_P1_Start".to_string(), b, 3, 4, Player::Player1));
    }
    // Board 1: Completely empty board, P2 to move (d1=5, d2=2)
    {
        let b = Board::new();
        states.push(("Empty_P2_Start".to_string(), b, 5, 2, Player::Player2));
    }
    // Board 2: P1 has placed initial piece at (0, 0), P2 to move for first turn
    {
        let mut b = Board::new();
        for r in 0..3 {
            for c in 0..4 {
                b.set(r, c, PLAYER_1);
            }
        }
        b.p1_score = 12;
        b.turn = 1;
        states.push(("P1_Placed_P2_Start".to_string(), b, 2, 6, Player::Player2));
    }
    // Board 3: Both placed initial pieces at corners
    {
        let mut b = Board::new();
        for r in 0..2 {
            for c in 0..5 {
                b.set(r, c, PLAYER_1);
            }
        }
        b.p1_score = 10;
        for r in 12..15 {
            for c in 36..40 {
                b.set(r, c, PLAYER_2);
            }
        }
        b.p2_score = 12;
        b.turn = 2;
        states.push(("Both_Corners_Placed".to_string(), b, 4, 4, Player::Player1));
    }
    // Board 4: Large initial pieces at corners
    {
        let mut b = Board::new();
        for r in 0..6 {
            for c in 0..6 {
                b.set(r, c, PLAYER_1);
            }
        }
        b.p1_score = 36;
        for r in 9..15 {
            for c in 34..40 {
                b.set(r, c, PLAYER_2);
            }
        }
        b.p2_score = 36;
        b.turn = 2;
        states.push(("Large_Corners_Placed".to_string(), b, 3, 5, Player::Player2));
    }

    // 6-15: Opening Phase (> 490 empty cells)
    // ------------------------------------------------------------------------
    for i in 1..=10 {
        let mut b = Board::new();
        let p1_len = 5 + i * 2;
        let p2_len = 5 + i * 2;

        // P1 expands horizontally along top
        for r in 0..2 {
            for c in 0..p1_len.min(30) {
                b.set(r, c, PLAYER_1);
            }
        }
        b.p1_score = (2 * p1_len.min(30)) as u16;

        // P2 expands horizontally along bottom
        for r in 13..15 {
            let start_c = 40usize.saturating_sub(p2_len.min(30));
            for c in start_c..40 {
                b.set(r, c, PLAYER_2);
            }
        }
        b.p2_score = (2 * p2_len.min(30)) as u16;
        b.turn = (i * 2) as usize;

        let d1 = (i % 6) + 1;
        let d2 = ((i * 3) % 6) + 1;
        let player = if i % 2 == 0 { Player::Player1 } else { Player::Player2 };
        states.push((format!("Opening_Phase_{}", i), b, d1, d2, player));
    }

    // 16-25: Midgame Phase (180..490 empty cells)
    // ------------------------------------------------------------------------
    for i in 1..=10 {
        let mut b = Board::new();
        let fill_cols_p1 = 5 + i;
        let fill_cols_p2 = 5 + i;

        // P1 block on left side
        for r in 0..BOARD_HEIGHT {
            for c in 0..fill_cols_p1.min(18) {
                if (r + c + i) % 7 != 0 {
                    b.set(r, c, PLAYER_1);
                }
            }
        }
        // P2 block on right side
        for r in 0..BOARD_HEIGHT {
            let start_c = 40usize.saturating_sub(fill_cols_p2.min(18));
            for c in start_c..40 {
                if (r + c + i) % 6 != 0 {
                    b.set(r, c, PLAYER_2);
                }
            }
        }

        b.p1_score = b.cells.iter().filter(|&&c| c == PLAYER_1).count() as u16;
        b.p2_score = b.cells.iter().filter(|&&c| c == PLAYER_2).count() as u16;
        b.turn = (20 + i * 2) as usize;

        let d1 = ((i * 2) % 6) + 1;
        let d2 = ((i * 5) % 6) + 1;
        let player = if i % 2 == 0 { Player::Player2 } else { Player::Player1 };
        states.push((format!("Midgame_Phase_{}", i), b, d1, d2, player));
    }

    // 26-35: Endgame Phase (< 180 empty cells)
    // ------------------------------------------------------------------------
    for i in 1..=10 {
        let mut b = Board::new();
        // Fill most of the board
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                if c < 20 {
                    b.set(r, c, PLAYER_1);
                } else {
                    b.set(r, c, PLAYER_2);
                }
            }
        }

        // Carve out pockets of empty cells (between 40 and 170 empty cells)
        let num_pockets = 15 - i;
        for p in 0..num_pockets {
            let pr = (p * 3 + i) % BOARD_HEIGHT;
            let pc = (p * 5 + i * 2) % BOARD_WIDTH;
            for dr in 0..2 {
                for dc in 0..2 {
                    if pr + dr < BOARD_HEIGHT && pc + dc < BOARD_WIDTH {
                        b.set(pr + dr, pc + dc, EMPTY);
                    }
                }
            }
        }

        b.p1_score = b.cells.iter().filter(|&&c| c == PLAYER_1).count() as u16;
        b.p2_score = b.cells.iter().filter(|&&c| c == PLAYER_2).count() as u16;
        b.turn = (50 + i) as usize;

        let d1 = (i % 6) + 1;
        let d2 = ((i + 2) % 6) + 1;
        let player = if i % 2 == 0 { Player::Player2 } else { Player::Player1 };
        states.push((format!("Endgame_Phase_{}", i), b, d1, d2, player));
    }

    // 36-40: Almost Full / Saturated Boards (550+ occupied cells)
    // ------------------------------------------------------------------------
    for i in 1..=5 {
        let mut b = Board::new();
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                if c < 20 {
                    b.set(r, c, PLAYER_1);
                } else {
                    b.set(r, c, PLAYER_2);
                }
            }
        }

        // Leave only tiny gaps of 5-15 empty cells
        for g in 0..(i * 3) {
            let gr = (g * 7 + i) % BOARD_HEIGHT;
            let gc = (g * 11 + i * 3) % BOARD_WIDTH;
            b.set(gr, gc, EMPTY);
        }

        b.p1_score = b.cells.iter().filter(|&&c| c == PLAYER_1).count() as u16;
        b.p2_score = b.cells.iter().filter(|&&c| c == PLAYER_2).count() as u16;
        b.turn = (70 + i) as usize;

        states.push((format!("Saturated_Board_{}", i), b, 1, 1, Player::Player2));
    }

    // 41-45: Edge Cases: Zero Legal Moves, Narrow Chokepoints, Walls
    // ------------------------------------------------------------------------
    // Board 40: Zero Legal Moves (Board completely full)
    {
        let mut b = Board::new();
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                b.set(r, c, if c < 20 { PLAYER_1 } else { PLAYER_2 });
            }
        }
        b.p1_score = 300;
        b.p2_score = 300;
        b.turn = 78;
        states.push(("Edge_Full_Board_No_Moves".to_string(), b, 2, 3, Player::Player2));
    }
    // Board 41: Large roll with no legal fitting spot
    {
        let mut b = Board::new();
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                // Checkerboard pattern leaving 1x1 empty cells only
                if (r + c) % 2 == 0 {
                    b.set(r, c, if c < 20 { PLAYER_1 } else { PLAYER_2 });
                }
            }
        }
        b.p1_score = b.cells.iter().filter(|&&c| c == PLAYER_1).count() as u16;
        b.p2_score = b.cells.iter().filter(|&&c| c == PLAYER_2).count() as u16;
        b.turn = 60;
        // Roll 6x6 on checkerboard -> 0 legal moves possible!
        states.push(("Edge_Checkerboard_Roll6x6".to_string(), b, 6, 6, Player::Player1));
    }
    // Board 42: Narrow 1-cell corridor between enemy walls (chokepoint challenge)
    {
        let mut b = Board::new();
        // P1 forms vertical walls with a 1-cell corridor in between
        for r in 0..BOARD_HEIGHT {
            b.set(r, 19, PLAYER_1);
            b.set(r, 21, PLAYER_1);
        }
        b.p1_score = (BOARD_HEIGHT * 2) as u16;
        // P2 anchored at bottom-right corner
        for r in 10..15 {
            for c in 35..40 {
                b.set(r, c, PLAYER_2);
            }
        }
        b.p2_score = 25;
        b.turn = 15;
        states.push(("Edge_Narrow_Corridor_Pinch".to_string(), b, 1, 4, Player::Player2));
    }
    // Board 43: Horizontal bottleneck along wall
    {
        let mut b = Board::new();
        for c in 0..25 {
            b.set(0, c, PLAYER_1);
            b.set(2, c, PLAYER_1);
        }
        b.p1_score = 50;
        for r in 11..15 {
            for c in 30..40 {
                b.set(r, c, PLAYER_2);
            }
        }
        b.p2_score = 40;
        b.turn = 22;
        states.push(("Edge_Horizontal_Bottleneck".to_string(), b, 1, 3, Player::Player2));
    }
    // Board 44: Symmetric diagonal expansion
    {
        let mut b = Board::new();
        for i in 0..10 {
            b.set(i, i * 2, PLAYER_1);
            b.set(14 - i, 39 - i * 2, PLAYER_2);
        }
        b.p1_score = 10;
        b.p2_score = 10;
        b.turn = 8;
        states.push(("Edge_Symmetric_Diagonals".to_string(), b, 2, 2, Player::Player1));
    }

    // 46-50: Realistic Snapshot States from Simulated Bot Matches
    // ------------------------------------------------------------------------
    for (idx, seed) in [42u32, 1337, 99999, 54321, 77777].iter().enumerate() {
        let mut prng = Mulberry32::new(*seed);
        let mut b = Board::new();
        let target_turns = 10 + idx * 8;

        for turn in 0..target_turns {
            let active_player = if turn % 2 == 0 { Player::Player1 } else { Player::Player2 };
            let d1 = prng.roll_die();
            let d2 = prng.roll_die();

            let eval_board = if active_player == Player::Player1 {
                b.rotate_board_180()
            } else {
                b.clone()
            };

            let legal = eval_board.generate_legal_moves(Player::Player2, d1, d2);
            if !legal.is_empty() {
                let chosen = &legal[0];
                let real_mv = if active_player == Player::Player1 {
                    Board::inverse_move_180(chosen)
                } else {
                    *chosen
                };
                b.apply_move(active_player, &real_mv);
            }
        }

        states.push((
            format!("Simulated_Match_Turn_{}", target_turns),
            b,
            3,
            4,
            if idx % 2 == 0 { Player::Player2 } else { Player::Player1 },
        ));
    }

    assert_eq!(states.len(), 50, "Must generate exactly 50 diverse board states");
    states
}

// ============================================================================
// TASK 1: Generate Decision Trees Across 50 Diverse Board States
// ============================================================================

#[test]
fn test_empirical_50_diverse_boards_decision_trees() {
    let board_states = generate_50_diverse_boards();
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    println!("\n=== TASK 1: Generating Decision Trees Across 50 Diverse Board States ===");

    let mut total_candidates_across_all = 0;
    let mut total_pruned_across_all = 0;
    let mut total_evaluated_across_all = 0;
    let mut zero_candidate_passes = 0;

    for (idx, (name, board, d1, d2, player)) in board_states.iter().enumerate() {
        let top_k = 4; // user budgeted top_k
        let tree = DecisionTree::build(board, *player, *d1, *d2, &weights, &matrix, top_k);

        // Core Invariants
        assert_eq!(tree.turn, board.turn, "Tree turn must match board turn");
        assert_eq!(tree.player, player.cell_value(), "Tree player must match requested player");
        assert_eq!(tree.dice, [*d1, *d2], "Tree dice must match input dice");

        // Beam pruning invariants
        assert_eq!(
            tree.total_legal_candidates,
            tree.evaluated_candidates.len() + tree.pruned_count,
            "Total candidates must equal evaluated + pruned"
        );
        assert!(
            tree.evaluated_candidates.len() <= 5,
            "Evaluated candidates must be bounded by Top-K (<= 5)"
        );

        if tree.total_legal_candidates == 0 {
            zero_candidate_passes += 1;
            assert_eq!(tree.evaluated_candidates.len(), 0);
            assert_eq!(tree.pruned_count, 0);
            assert!(tree.root.is_some());
            let root = tree.root.as_ref().unwrap();
            assert!(root.label.contains("Pass") || root.label.contains("No Legal Moves"));
        } else {
            // Check ranking monotonicity and uniqueness
            for (rank_idx, cand) in tree.evaluated_candidates.iter().enumerate() {
                assert_eq!(cand.rank, rank_idx + 1, "Candidate rank must be 1-indexed sequential");
                assert!(
                    cand.rect.r0 + cand.rect.h <= BOARD_HEIGHT,
                    "Move height must not exceed board bounds"
                );
                assert!(
                    cand.rect.c0 + cand.rect.w <= BOARD_WIDTH,
                    "Move width must not exceed board bounds"
                );
            }

            // Exactly one candidate should be marked selected
            let selected_count = tree.evaluated_candidates.iter().filter(|c| c.selected).count();
            assert_eq!(selected_count, 1, "Exactly one candidate must be marked selected");

            // Root node must be present and have children
            assert!(tree.root.is_some());
            let root = tree.root.as_ref().unwrap();
            assert_eq!(root.id, "root");
            let expected_children = tree.evaluated_candidates.len() + if tree.pruned_count > 0 { 1 } else { 0 };
            assert_eq!(
                root.children.len(),
                expected_children,
                "Root node children count must match evaluated candidates + pruned summary"
            );
        }

        total_candidates_across_all += tree.total_legal_candidates;
        total_pruned_across_all += tree.pruned_count;
        total_evaluated_across_all += tree.evaluated_candidates.len();

        if (idx + 1) % 10 == 0 || idx == 0 {
            println!(
                "  State #{:02} [{:<28}]: Legal={}, Evaluated={}, Pruned={}, Phase={:?}",
                idx + 1,
                name,
                tree.total_legal_candidates,
                tree.evaluated_candidates.len(),
                tree.pruned_count,
                tree.phase
            );
        }
    }

    println!("Summary Task 1:");
    println!("  Total board states tested: {}", board_states.len());
    println!("  Zero-move pass states: {}", zero_candidate_passes);
    println!("  Total legal candidates: {}", total_candidates_across_all);
    println!("  Total evaluated candidates (beam top-4): {}", total_evaluated_across_all);
    println!("  Total pruned candidates: {}", total_pruned_across_all);
    println!("  [PASS] All 50 decision trees constructed successfully with 100% invariant adherence.\n");
}

// ============================================================================
// TASK 2: Validate Graphviz DOT Export (Syntax, Braces, Declarations)
// ============================================================================

/// Independent, strict Graphviz DOT validator that verifies syntax, balanced braces,
/// node declarations, and edge endpoint references.
fn independently_validate_dot(dot: &str) -> Result<(), String> {
    // 1. Basic structural checks
    let trimmed = dot.trim();
    if !trimmed.starts_with("digraph DecisionTree {") {
        return Err("DOT must start with 'digraph DecisionTree {'".to_string());
    }
    if !trimmed.ends_with('}') {
        return Err("DOT must end with '}'".to_string());
    }

    // 2. Balanced braces and brackets check
    let mut brace_balance = 0;
    let mut bracket_balance = 0;
    let mut in_quote = false;
    let mut prev_char = '\0';

    for ch in trimmed.chars() {
        if ch == '"' && prev_char != '\\' {
            in_quote = !in_quote;
        } else if !in_quote {
            match ch {
                '{' => brace_balance += 1,
                '}' => {
                    brace_balance -= 1;
                    if brace_balance < 0 {
                        return Err("Negative brace balance".to_string());
                    }
                }
                '[' => bracket_balance += 1,
                ']' => {
                    bracket_balance -= 1;
                    if bracket_balance < 0 {
                        return Err("Negative bracket balance".to_string());
                    }
                }
                _ => {}
            }
        }
        prev_char = ch;
    }

    if in_quote {
        return Err("Unclosed double quote in DOT".to_string());
    }
    if brace_balance != 0 {
        return Err(format!("Unbalanced braces in DOT: net balance = {}", brace_balance));
    }
    if bracket_balance != 0 {
        return Err(format!("Unbalanced brackets in DOT: net balance = {}", bracket_balance));
    }

    // 3. Node and edge declaration analysis
    let mut declared_nodes: HashSet<String> = HashSet::new();
    let mut referenced_edges: Vec<(String, String)> = Vec::new();

    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("digraph")
            || line == "}"
            || line.starts_with("rankdir")
            || line.starts_with("node [")
            || line.starts_with("edge [")
        {
            continue;
        }

        if line.contains("->") {
            // Edge line: e.g., "root -> c1 [label="..."];"
            let arrow_parts: Vec<&str> = line.split("->").collect();
            if arrow_parts.len() != 2 {
                return Err(format!("Invalid edge declaration: {}", line));
            }
            let from_node = arrow_parts[0].trim().to_string();
            let right_side = arrow_parts[1].trim();
            let to_node = right_side
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches(';')
                .to_string();

            referenced_edges.push((from_node, to_node));
        } else if line.contains('[') {
            // Node declaration line: e.g., "root [label=...];" or "c1 [label=...];"
            let id = line.split('[').next().unwrap_or("").trim().to_string();
            if !id.is_empty() {
                declared_nodes.insert(id);
            }
        }
    }

    // 4. Verify all edge endpoints were declared
    for (from, to) in &referenced_edges {
        if !declared_nodes.contains(from) {
            return Err(format!("Edge references undeclared source node '{}'", from));
        }
        if !declared_nodes.contains(to) {
            return Err(format!("Edge references undeclared target node '{}'", to));
        }
    }

    Ok(())
}

#[test]
fn test_graphviz_dot_export_rigorous_validation() {
    let board_states = generate_50_diverse_boards();
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    println!("\n=== TASK 2: Validating Graphviz DOT Export Across 50 States ===");

    for (idx, (name, board, d1, d2, player)) in board_states.iter().enumerate() {
        let tree = DecisionTree::build(board, *player, *d1, *d2, &weights, &matrix, 4);

        // Export via struct method and standalone helper
        let dot_method = tree.to_dot();
        let dot_helper = export_to_dot(&tree);
        assert_eq!(dot_method, dot_helper, "Standalone export_to_dot must match tree.to_dot()");

        // 1. In-tree syntax validator
        let val_internal = DecisionTree::validate_dot_syntax(&dot_method);
        assert!(
            val_internal.is_ok(),
            "State #{} [{}]: Internal DOT validation failed: {:?}",
            idx + 1,
            name,
            val_internal.err()
        );

        // 2. Independent strict validator
        let val_external = independently_validate_dot(&dot_method);
        assert!(
            val_external.is_ok(),
            "State #{} [{}]: Independent DOT validation failed: {:?}",
            idx + 1,
            name,
            val_external.err()
        );

        // 3. Highlight assertion: selected candidate node has green highlight
        if !tree.evaluated_candidates.is_empty() {
            assert!(
                dot_method.contains("#c6f6d5"),
                "Selected candidate must have green highlight color '#c6f6d5'"
            );
        }
    }

    println!("  [PASS] All 50 Graphviz DOT files verified for balanced braces, node declarations, and edge referential integrity.\n");
}

// ============================================================================
// TASK 3: Validate JSON Export (Parsing, Schema, Required Fields)
// ============================================================================

#[test]
fn test_json_export_schema_and_field_conformance() {
    let board_states = generate_50_diverse_boards();
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    println!("\n=== TASK 3: Validating JSON Export Across 50 Diverse States ===");

    for (idx, (name, board, d1, d2, player)) in board_states.iter().enumerate() {
        let tree = DecisionTree::build(board, *player, *d1, *d2, &weights, &matrix, 4);

        let json_method = tree.to_json();
        let json_helper = export_to_json(&tree);
        assert_eq!(json_method, json_helper, "Standalone export_to_json must match tree.to_json()");

        // 1. In-tree JSON validator
        let val_internal = DecisionTree::validate_json_syntax(&json_method);
        assert!(
            val_internal.is_ok(),
            "State #{} [{}]: Internal JSON validation failed: {:?}",
            idx + 1,
            name,
            val_internal.err()
        );

        // 2. Parse cleanly with serde_json into Generic Value
        let parsed_value: serde_json::Value = serde_json::from_str(&json_method)
            .unwrap_or_else(|e| panic!("State #{}: Failed to parse JSON: {}", idx + 1, e));

        assert!(parsed_value.is_object(), "JSON root must be an object");

        // 3. Check required top-level fields
        assert!(parsed_value.get("root").is_some(), "JSON must contain 'root'");
        assert!(parsed_value.get("turn").is_some(), "JSON must contain 'turn'");
        assert!(parsed_value.get("player").is_some(), "JSON must contain 'player'");
        assert!(parsed_value.get("dice").is_some(), "JSON must contain 'dice'");
        assert!(parsed_value.get("boardState").is_some(), "JSON must contain 'boardState'");

        // Field naming analysis:
        // Struct defines:
        // - `evaluated_candidates` (serialized as "evaluatedCandidates")
        // - `phase` (serialized as "phase")
        // - `pruned_count` (serialized as "prunedCount")
        // Check presence of these fields
        assert!(
            parsed_value.get("evaluatedCandidates").is_some(),
            "JSON must contain candidates under 'evaluatedCandidates'"
        );
        assert!(
            parsed_value.get("phase").is_some(),
            "JSON must contain search phase under 'phase'"
        );
        assert!(
            parsed_value.get("prunedCount").is_some(),
            "JSON must contain pruned candidates count under 'prunedCount'"
        );

        // 4. Verify candidate entries structure
        let candidates_arr = parsed_value
            .get("evaluatedCandidates")
            .unwrap()
            .as_array()
            .expect("evaluatedCandidates must be an array");

        for (c_idx, cand) in candidates_arr.iter().enumerate() {
            assert!(cand.get("rank").is_some(), "Candidate #{}: missing rank", c_idx);
            assert!(cand.get("rect").is_some(), "Candidate #{}: missing rect", c_idx);
            assert!(cand.get("score").is_some(), "Candidate #{}: missing score", c_idx);
            assert!(cand.get("breakdown").is_some(), "Candidate #{}: missing breakdown", c_idx);
            assert!(
                cand.get("twoPlyCounterThreat").is_some(),
                "Candidate #{}: missing twoPlyCounterThreat",
                c_idx
            );

            // Verify breakdown has 9+ components
            let bd = cand.get("breakdown").unwrap();
            assert!(bd.get("area").is_some(), "breakdown missing area");
            assert!(bd.get("compactness").is_some(), "breakdown missing compactness");
            assert!(bd.get("loopCapture").is_some(), "breakdown missing loopCapture");
            assert!(bd.get("loopDenial").is_some(), "breakdown missing loopDenial");
            assert!(bd.get("voronoiDelta").is_some(), "breakdown missing voronoiDelta");
            assert!(bd.get("safeTerritory").is_some(), "breakdown missing safeTerritory");
            assert!(bd.get("spatialMatrix").is_some(), "breakdown missing spatialMatrix");
            assert!(bd.get("frontier").is_some(), "breakdown missing frontier");
            assert!(bd.get("chokepointPenalty").is_some(), "breakdown missing chokepointPenalty");
        }

        // 5. Roundtrip Deserialization: Verify JSON deserializes cleanly back into DecisionTree
        let roundtrip_tree: DecisionTree = serde_json::from_str(&json_method)
            .unwrap_or_else(|e| panic!("State #{}: Failed to deserialize into DecisionTree: {}", idx + 1, e));

        assert_eq!(roundtrip_tree.turn, tree.turn);
        assert_eq!(roundtrip_tree.player, tree.player);
        assert_eq!(roundtrip_tree.dice, tree.dice);
        assert_eq!(roundtrip_tree.phase, tree.phase);
        assert_eq!(roundtrip_tree.total_legal_candidates, tree.total_legal_candidates);
        assert_eq!(roundtrip_tree.pruned_count, tree.pruned_count);
        assert_eq!(roundtrip_tree.evaluated_candidates.len(), tree.evaluated_candidates.len());
    }

    println!("  [PASS] All 50 decision tree JSON files parse cleanly and support 100% lossless roundtrip deserialization.\n");
}

// ============================================================================
// TASK 4: Test ModelSelector Across 100 CRN Pairs (Determinism & Promotion)
// ============================================================================

#[test]
fn test_model_selector_100_crn_pairs_determinism_and_promotion() {
    println!("\n=== TASK 4: Testing ModelSelector Across 100 CRN Pairs ===");

    // Generate 100 deterministic seeds using wrapping arithmetic
    let seeds: Vec<u32> = (1..=100)
        .map(|i| 10007u32.wrapping_add((i as u32).wrapping_mul(2654435761u32)))
        .collect();
    assert_eq!(seeds.len(), 100, "Must use exactly 100 CRN seed pairs (200 matches)");

    let matrix = generate_default_v2_matrix();
    let default_params = HeuristicParams::default();

    // Candidate A: Slightly higher Voronoi delta
    let mut cand_a_params = default_params.clone();
    cand_a_params.weight_voronoi_delta = 35.0;
    cand_a_params.clamp();

    // Candidate B: Higher capture weight
    let mut cand_b_params = default_params.clone();
    cand_b_params.weight_capture = 120.0;
    cand_b_params.clamp();

    // Candidate C: Baseline clone
    let cand_c_params = default_params.clone();

    // 1. CRN Determinism Check: Run Candidate A vs Champion over 100 seeds TWICE
    let mut selector = ModelSelector::new(default_params.clone(), matrix);
    let id_a = selector.add_candidate(cand_a_params.clone(), matrix);
    let id_b = selector.add_candidate(cand_b_params.clone(), matrix);
    let id_c = selector.add_candidate(cand_c_params.clone(), matrix);

    assert_eq!(id_a, 1);
    assert_eq!(id_b, 2);
    assert_eq!(id_c, 3);

    println!("  Evaluating Candidate A over 100 CRN pairs (Run 1)...");
    let stats_run1 = selector.evaluate_candidate_vs_champion(0, &seeds, 80);

    println!("  Evaluating Candidate A over 100 CRN pairs (Run 2 for Bit-Exact Parity)...");
    let stats_run2 = selector.evaluate_candidate_vs_champion(0, &seeds, 80);

    // Assert Bit-Exact CRN Parity
    assert_eq!(stats_run1.wins, stats_run2.wins, "Wins must be bit-exact identical");
    assert_eq!(stats_run1.losses, stats_run2.losses, "Losses must be bit-exact identical");
    assert_eq!(stats_run1.draws, stats_run2.draws, "Draws must be bit-exact identical");
    assert_eq!(stats_run1.total_matches, 200, "100 CRN seeds must equal 200 matches");
    assert_eq!(stats_run1.total_turns, stats_run2.total_turns, "Total turns must match");
    assert!(
        (stats_run1.net_margin - stats_run2.net_margin).abs() < 1e-9,
        "Net margin must be bit-exact identical"
    );
    assert!(
        (stats_run1.fitness - stats_run2.fitness).abs() < 1e-9,
        "Fitness must be bit-exact identical"
    );

    println!(
        "  CRN 100-Pair Results for Candidate A vs Champion:\n    Matches: {} | Wins: {}, Losses: {}, Draws: {}\n    Winrate: {:.2}% | AvgMargin: {:.2} | Fitness: {:.2}",
        stats_run1.total_matches,
        stats_run1.wins,
        stats_run1.losses,
        stats_run1.draws,
        stats_run1.winrate * 100.0,
        stats_run1.avg_margin,
        stats_run1.fitness
    );

    // 2. Population Evaluation & Ranking
    selector.evaluate_population_vs_champion(&seeds, 80);

    let ranking = selector.rank_population();
    assert_eq!(ranking.len(), 3, "Ranking must contain all 3 population members");

    // Verify monotonic descending fitness order
    for w in ranking.windows(2) {
        let f_prev = selector.population[w[0]].stats.fitness;
        let f_curr = selector.population[w[1]].stats.fitness;
        assert!(
            f_prev >= f_curr,
            "Ranked population must be in strict descending order of fitness: {} >= {}",
            f_prev,
            f_curr
        );
    }

    let (expected_new_champ_id, expected_new_champ_weights) = {
        let best_cand = selector.get_best_candidate().expect("Best candidate must exist");
        println!(
            "  Population Winner: Candidate #{} with fitness {:.2} (winrate {:.2}%, avg_margin {:.2})",
            best_cand.id,
            best_cand.stats.fitness,
            best_cand.stats.winrate * 100.0,
            best_cand.stats.avg_margin
        );
        (best_cand.id, best_cand.weights.clone())
    };

    // 3. Champion Promotion Verification
    let initial_gen = selector.generation;
    let old_champ_weights = selector.champion.weights.clone();

    // Test rejection: impossible threshold (winrate 1.01) -> must NOT promote
    let promoted_false = selector.select_and_promote_champion(1.01, 100.0);
    assert!(!promoted_false, "Should not promote when thresholds are not met");
    assert_eq!(selector.generation, initial_gen, "Generation must not increment");
    assert_eq!(selector.champion.weights, old_champ_weights, "Champion must not change");

    // Test promotion: achievable threshold (winrate 0.0, margin -999.0) -> must promote best candidate
    let promoted_true = selector.select_and_promote_champion(0.0, -999.0);
    assert!(promoted_true, "Should promote best candidate when thresholds are met");
    assert_eq!(selector.generation, initial_gen + 1, "Generation must increment by 1");
    assert_eq!(
        selector.champion.id, expected_new_champ_id,
        "Champion ID must match promoted candidate"
    );
    assert_eq!(
        selector.champion.weights, expected_new_champ_weights,
        "Champion weights must match promoted candidate"
    );
    assert_eq!(selector.history.len(), 1, "Selector history must record 1 entry");

    // 4. Persistence Roundtrip Verification
    let temp_weights_path = "/tmp/test_challenger_m2_weights.json";
    let save_res = selector.save_champion(temp_weights_path);
    assert!(save_res.is_ok(), "Failed to save champion to JSON: {:?}", save_res.err());

    let (loaded_weights, loaded_mat, loaded_meta) =
        load_from_file(temp_weights_path).expect("Failed to load champion from JSON");

    assert_eq!(loaded_weights, selector.champion.weights, "Loaded weights must match saved champion");
    assert_eq!(loaded_mat, selector.champion.matrix, "Loaded matrix must match saved matrix");
    assert_eq!(loaded_meta.epoch, selector.generation, "Loaded epoch must match generation");

    // Clean up temp file
    let _ = std::fs::remove_file(temp_weights_path);

    println!("  [PASS] 100 CRN pairs evaluated deterministically; ranking, champion promotion, and weights.json persistence verified.\n");
}

// ============================================================================
// TASK 5: Benchmark Tree Construction Throughput (>1000 Trees/Sec)
// ============================================================================

#[test]
fn test_decision_tree_construction_throughput_benchmark() {
    println!("\n=== TASK 5: Benchmarking Decision Tree Construction Throughput ===");

    let board_states = generate_50_diverse_boards();
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    // Warm-up run across 50 boards
    for (_, board, d1, d2, player) in &board_states {
        let _ = DecisionTree::build(board, *player, *d1, *d2, &weights, &matrix, 4);
    }

    // Benchmark loop: 2,000 tree constructions
    let iterations = 2000;
    let mut latencies_us = Vec::with_capacity(iterations);

    let start_total = Instant::now();

    for i in 0..iterations {
        let (_, ref board, d1, d2, player) = board_states[i % board_states.len()];
        let t0 = Instant::now();
        let tree = DecisionTree::build(board, player, d1, d2, &weights, &matrix, 4);
        let elapsed_us = t0.elapsed().as_micros() as u64;
        latencies_us.push(elapsed_us);

        // Black-box optimization barrier
        std::hint::black_box(tree);
    }

    let total_elapsed = start_total.elapsed();
    let trees_per_sec = (iterations as f64) / total_elapsed.as_secs_f64();

    latencies_us.sort_unstable();
    let p50_us = latencies_us[iterations / 2];
    let p90_us = latencies_us[(iterations * 90) / 100];
    let p99_us = latencies_us[(iterations * 99) / 100];
    let max_us = latencies_us[iterations - 1];

    println!("Throughput Benchmark Results across {} trees:", iterations);
    println!("  Total elapsed time: {:.3} ms", total_elapsed.as_secs_f64() * 1000.0);
    println!("  Throughput: {:.1} trees/sec (Budget target: >1,000 trees/sec)", trees_per_sec);
    println!("  Latency p50: {} µs ({:.3} ms)", p50_us, p50_us as f64 / 1000.0);
    println!("  Latency p90: {} µs ({:.3} ms)", p90_us, p90_us as f64 / 1000.0);
    println!("  Latency p99: {} µs ({:.3} ms)", p99_us, p99_us as f64 / 1000.0);
    println!("  Latency max: {} µs ({:.3} ms)", max_us, max_us as f64 / 1000.0);

    assert!(
        trees_per_sec > 1000.0,
        "Decision tree throughput ({:.1} trees/sec) must exceed 1000 trees/sec SLA budget",
        trees_per_sec
    );

    println!("  [PASS] Throughput budget achieved: {:.1} trees/sec > 1,000 trees/sec.\n", trees_per_sec);
}
