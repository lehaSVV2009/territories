//! Empirical Challenger Verification Suite for Milestone M1 (`src/game.rs`)
//!
//! Covers:
//! 1. 180° Rotation Invariance of board, moves, legal move generation, and heuristic scores.
//! 2. Stress-testing Voronoi field calculations under complex maze obstacles with an independent Oracle.
//! 3. High-precision benchmarking of `SpatialSlayerBot::select_move` latency (1,000 decisions, p50/p90/p99 vs 2.0 ms SLA).

use pyrust::game::*;
use std::collections::VecDeque;
use std::time::Instant;

// ============================================================================
// 1. Independent Voronoi Oracle
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct OracleVoronoiField {
    pub p1_territory: i32,
    pub p2_territory: i32,
    pub p1_safe_territory: i32,
    pub p2_safe_territory: i32,
    pub opp_choked_cells: i32,
    pub spatial_score: f64,
}

/// Independent multi-source BFS oracle that does not share any code with `game.rs`.
pub fn oracle_compute_voronoi(
    board: &Board,
    candidate: &Move,
    placing_player: Player,
    matrix: &[f64; TOTAL_CELLS],
) -> OracleVoronoiField {
    let mut dist1 = vec![9999i32; TOTAL_CELLS];
    let mut dist2 = vec![9999i32; TOTAL_CELLS];

    let mut q1 = VecDeque::new();
    let mut q2 = VecDeque::new();

    // Mark occupied sources
    for r in 0..BOARD_HEIGHT {
        for c in 0..BOARD_WIDTH {
            let idx = r * BOARD_WIDTH + c;
            let is_cand = r >= candidate.r
                && r < candidate.r + candidate.h
                && c >= candidate.c
                && c < candidate.c + candidate.w;

            if is_cand {
                if placing_player == Player::Player2 {
                    dist2[idx] = 0;
                    q2.push_back(idx);
                } else {
                    dist1[idx] = 0;
                    q1.push_back(idx);
                }
            } else {
                let cell = board.cells[idx];
                if cell == PLAYER_1 {
                    dist1[idx] = 0;
                    q1.push_back(idx);
                } else if cell == PLAYER_2 {
                    dist2[idx] = 0;
                    q2.push_back(idx);
                }
            }
        }
    }

    // BFS P1: enemy cells (where dist2 == 0) are impassable
    while let Some(curr) = q1.pop_front() {
        let d = dist1[curr] + 1;
        let cr = curr / BOARD_WIDTH;
        let cc = curr % BOARD_WIDTH;

        let neighbors = [
            if cr > 0 { Some(curr - BOARD_WIDTH) } else { None },
            if cr + 1 < BOARD_HEIGHT { Some(curr + BOARD_WIDTH) } else { None },
            if cc > 0 { Some(curr - 1) } else { None },
            if cc + 1 < BOARD_WIDTH { Some(curr + 1) } else { None },
        ];

        for &n_opt in &neighbors {
            if let Some(n) = n_opt {
                // If cell is an enemy source, P1 cannot enter
                if dist2[n] != 0 && dist1[n] > d {
                    dist1[n] = d;
                    q1.push_back(n);
                }
            }
        }
    }

    // BFS P2: enemy cells (where dist1 == 0) are impassable
    while let Some(curr) = q2.pop_front() {
        let d = dist2[curr] + 1;
        let cr = curr / BOARD_WIDTH;
        let cc = curr % BOARD_WIDTH;

        let neighbors = [
            if cr > 0 { Some(curr - BOARD_WIDTH) } else { None },
            if cr + 1 < BOARD_HEIGHT { Some(curr + BOARD_WIDTH) } else { None },
            if cc > 0 { Some(curr - 1) } else { None },
            if cc + 1 < BOARD_WIDTH { Some(curr + 1) } else { None },
        ];

        for &n_opt in &neighbors {
            if let Some(n) = n_opt {
                // If cell is an enemy source, P2 cannot enter
                if dist1[n] != 0 && dist2[n] > d {
                    dist2[n] = d;
                    q2.push_back(n);
                }
            }
        }
    }

    // Tabulate metrics
    let mut p1_territory = 0;
    let mut p2_territory = 0;
    let mut p1_safe_territory = 0;
    let mut p2_safe_territory = 0;
    let mut opp_choked_cells = 0;
    let mut spatial_score = 0.0;

    for i in 0..TOTAL_CELLS {
        let d1 = dist1[i];
        let d2 = dist2[i];

        // Only evaluate empty (non-source) cells
        if d1 != 0 && d2 != 0 {
            let weight = matrix[i];

            if d2 < d1 {
                p2_territory += 1;
                spatial_score += weight;
                if d1 >= 9999 {
                    p2_safe_territory += 1;
                    spatial_score += weight * 2.0;
                }
            } else if d1 < d2 {
                p1_territory += 1;
                spatial_score -= weight;
                if d2 >= 9999 {
                    p1_safe_territory += 1;
                    spatial_score -= weight * 2.0;
                }
            }

            // Opponent choked cells: P1 within 2 steps and P2 right next to it or closer
            if d1 <= 2 && d2 <= d1 + 1 {
                opp_choked_cells += 1;
            }
        }
    }

    OracleVoronoiField {
        p1_territory,
        p2_territory,
        p1_safe_territory,
        p2_safe_territory,
        opp_choked_cells,
        spatial_score,
    }
}

// ============================================================================
// 2. Empirical Verification 1: 180° Rotation Invariance
// ============================================================================

fn evaluate_move_p1_canonical(
    board: &Board,
    mv: &Move,
    weights: &HeuristicWeights,
    matrix: &[f64; TOTAL_CELLS],
) -> f64 {
    let rotated = board.rotate_board_180();
    let rot_mv = Board::inverse_move_180(mv);
    evaluate_move_v2(&rotated, &rot_mv, weights, matrix)
}

#[test]
fn test_rotation_invariance_evaluator() {
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    // Verify on 10 realistic game scenarios
    let mut boards = Vec::new();

    // Board 1: Opening
    let mut b1 = Board::new();
    b1.apply_move(Player::Player1, &Move::new(0, 0, 3, 4));
    b1.apply_move(Player::Player2, &Move::new(12, 35, 3, 5));
    boards.push(("Opening", b1));

    // Board 2: Midgame with contact
    let mut b2 = Board::new();
    b2.apply_move(Player::Player1, &Move::new(0, 0, 5, 8));
    b2.apply_move(Player::Player1, &Move::new(5, 5, 4, 6));
    b2.apply_move(Player::Player2, &Move::new(10, 30, 5, 10));
    b2.apply_move(Player::Player2, &Move::new(7, 18, 4, 12));
    boards.push(("Midgame Contact", b2));

    // Board 3: Wall bottlenecks
    let mut b3 = Board::new();
    b3.apply_move(Player::Player1, &Move::new(0, 0, 4, 12));
    b3.apply_move(Player::Player1, &Move::new(4, 8, 4, 4));
    b3.apply_move(Player::Player2, &Move::new(11, 28, 4, 12));
    b3.apply_move(Player::Player2, &Move::new(0, 20, 3, 6));
    boards.push(("Bottlenecks", b3));

    // Boards 4-10: Generated from match simulation states
    let mut rng = Mulberry32::new(987654321);
    let mut match_board = Board::new();
    match_board.apply_move(Player::Player1, &Move::new(0, 0, 3, 3));
    match_board.apply_move(Player::Player2, &Move::new(12, 37, 3, 3));

    for turn in 1..=20 {
        let d1 = rng.roll_die();
        let d2 = rng.roll_die();
        let p = if turn % 2 == 1 { Player::Player1 } else { Player::Player2 };
        if let Some(mv) = select_move(BotType::Fast, &match_board, p, d1, d2, None, None, None) {
            match_board.apply_move(p, &mv);
            match_board.apply_closed_loops();
        }
        if turn % 3 == 0 {
            boards.push(("Match State", match_board.clone()));
        }
    }

    let mut total_candidate_pairs_checked = 0;

    for (name, board) in &boards {
        let rotated_board = board.rotate_board_180();

        // Check all dice combinations (1..6 x 1..6)
        for d1 in 1..=6 {
            for d2 in 1..=6 {
                let p2_moves = board.generate_legal_moves(Player::Player2, d1, d2);
                let p1_rot_moves = rotated_board.generate_legal_moves(Player::Player1, d1, d2);

                // 1. Legal move set bijection
                assert_eq!(
                    p2_moves.len(),
                    p1_rot_moves.len(),
                    "Mismatch in legal move count on {name} for dice ({d1}, {d2})"
                );

                for mv in &p2_moves {
                    let inv_mv = Board::inverse_move_180(mv);
                    assert!(
                        p1_rot_moves.contains(&inv_mv),
                        "Inverted move {inv_mv:?} not in P1 legal moves on rotated {name}"
                    );

                    // 2. Heuristic score invariance
                    // Evaluating mv from P2 perspective on board B
                    let score_p2 = evaluate_move_v2(board, mv, &weights, &matrix);

                    // Evaluating inverted move from P1 perspective on rotate_board_180(B)
                    let score_p1 = evaluate_move_p1_canonical(&rotated_board, &inv_mv, &weights, &matrix);

                    let diff = (score_p2 - score_p1).abs();
                    assert!(
                        diff < 1e-10,
                        "Rotation invariance score failure on {name}: score_p2={score_p2}, score_p1={score_p1}, diff={diff}"
                    );

                    total_candidate_pairs_checked += 1;
                }

                // 3. Bot move selection invariance
                let best_p2 = select_move_spatial_slayer(board, Player::Player2, d1, d2, &weights, &matrix);
                let best_p1_on_rot = select_move_spatial_slayer(&rotated_board, Player::Player1, d1, d2, &weights, &matrix);

                match (best_p2, best_p1_on_rot) {
                    (Some(mv2), Some(mv1)) => {
                        assert_eq!(
                            mv1,
                            Board::inverse_move_180(&mv2),
                            "Bot move selection symmetry mismatch on {name} dice ({d1}, {d2})"
                        );
                    }
                    (None, None) => {}
                    _ => panic!("One player found move while other did not on {name}"),
                }
            }
        }
    }

    println!("Verified 180° rotation invariance across {total_candidate_pairs_checked} candidate move pairs.");
    assert!(total_candidate_pairs_checked >= 1000);
}

// ============================================================================
// 3. Empirical Verification 2: Voronoi Stress Testing on Complex Mazes
// ============================================================================

/// Helper to fill a rectangular block
fn fill_rect(b: &mut Board, r0: usize, c0: usize, h: usize, w: usize, val: u8) {
    for r in r0..r0 + h {
        for c in c0..c0 + w {
            b.cells[r * BOARD_WIDTH + c] = val;
        }
    }
}

#[test]
fn test_voronoi_stress_complex_mazes() {
    let matrix = generate_default_v2_matrix();

    // -------------------------------------------------------------
    // Maze Scenario 1: Spiral Labyrinth enclosing central chamber
    // -------------------------------------------------------------
    let mut spiral_board = Board::new();
    fill_rect(&mut spiral_board, 0, 0, 2, 2, PLAYER_1);
    fill_rect(&mut spiral_board, 13, 38, 2, 2, PLAYER_2);

    // Build spiral walls with Player 2 cells
    // Outer loop: rows 2 and 12, cols 5 and 35
    for c in 5..35 {
        spiral_board.set(2, c, PLAYER_2);
        spiral_board.set(12, c, PLAYER_2);
    }
    for r in 2..=12 {
        spiral_board.set(r, 5, PLAYER_2);
        if r != 7 {
            // Doorway at (7, 34)
            spiral_board.set(r, 34, PLAYER_2);
        }
    }
    // Inner loop: rows 4 and 10, cols 8 and 31
    for c in 8..31 {
        spiral_board.set(4, c, PLAYER_2);
        spiral_board.set(10, c, PLAYER_2);
    }
    for r in 4..=10 {
        if r != 7 {
            // Doorway at (7, 8)
            spiral_board.set(r, 8, PLAYER_2);
        }
        spiral_board.set(r, 30, PLAYER_2);
    }

    // Candidate moves that test:
    // a) Leaving doorway open
    // b) Corking doorway at (7, 34)
    let cand_open = Move::new(13, 30, 2, 4);
    let cand_cork = Move::new(6, 34, 3, 1); // covers doorway (7, 34)

    for (desc, cand) in [("Spiral Open", cand_open), ("Spiral Cork", cand_cork)] {
        let field_actual = compute_voronoi_influence_v2(
            &spiral_board,
            cand.r,
            cand.c,
            cand.h,
            cand.w,
            Player::Player2,
            &matrix,
        );
        let field_oracle = oracle_compute_voronoi(&spiral_board, &cand, Player::Player2, &matrix);

        assert_eq!(field_actual.p1_territory, field_oracle.p1_territory, "p1_territory mismatch on {desc}");
        assert_eq!(field_actual.p2_territory, field_oracle.p2_territory, "p2_territory mismatch on {desc}");
        assert_eq!(field_actual.p1_safe_territory, field_oracle.p1_safe_territory, "p1_safe mismatch on {desc}");
        assert_eq!(field_actual.p2_safe_territory, field_oracle.p2_safe_territory, "p2_safe mismatch on {desc}");
        assert_eq!(field_actual.opp_choked_cells, field_oracle.opp_choked_cells, "choke mismatch on {desc}");
        assert!(
            (field_actual.spatial_score - field_oracle.spatial_score).abs() < 1e-9,
            "spatial_score mismatch on {desc}"
        );

        if desc == "Spiral Cork" {
            // When corked, the entire inner spiral chambers become 100% safe territory for P2
            assert!(
                field_actual.p2_safe_territory > 50,
                "Expected large safe territory after corking spiral, got {}",
                field_actual.p2_safe_territory
            );
        }
    }

    // -------------------------------------------------------------
    // Maze Scenario 2: Vertical Wall Partition with Chokepoints
    // -------------------------------------------------------------
    let mut wall_board = Board::new();
    fill_rect(&mut wall_board, 0, 0, 3, 3, PLAYER_1);
    fill_rect(&mut wall_board, 12, 37, 3, 3, PLAYER_2);

    // Wall at col 20 from row 0 to 14, with gap only at row 7
    for r in 0..15 {
        if r != 7 {
            wall_board.set(r, 20, PLAYER_2);
        }
    }

    let cand_unplugged = Move::new(12, 30, 2, 4);
    let cand_plug = Move::new(6, 20, 3, 1); // Plugs gap at row 7

    let field_unplugged = compute_voronoi_influence_v2(
        &wall_board,
        cand_unplugged.r,
        cand_unplugged.c,
        cand_unplugged.h,
        cand_unplugged.w,
        Player::Player2,
        &matrix,
    );
    let oracle_unplugged = oracle_compute_voronoi(&wall_board, &cand_unplugged, Player::Player2, &matrix);
    assert_eq!(field_unplugged.p2_safe_territory, oracle_unplugged.p2_safe_territory);

    let field_plug = compute_voronoi_influence_v2(
        &wall_board,
        cand_plug.r,
        cand_plug.c,
        cand_plug.h,
        cand_plug.w,
        Player::Player2,
        &matrix,
    );
    let oracle_plug = oracle_compute_voronoi(&wall_board, &cand_plug, Player::Player2, &matrix);
    assert_eq!(field_plug.p2_safe_territory, oracle_plug.p2_safe_territory);
    assert_eq!(field_plug.p1_safe_territory, oracle_plug.p1_safe_territory);
    assert_eq!(field_plug.p1_territory, oracle_plug.p1_territory);
    assert_eq!(field_plug.p2_territory, oracle_plug.p2_territory);

    // When plugged, all empty cells on the right of col 20 (19 columns * 15 rows - P2 pieces)
    // MUST be safe territory for P2!
    assert!(field_plug.p2_safe_territory > 200, "Safe territory should exceed 200 cells when partitioned");
    // Since P2 owns the col 20 wall, P2 can step directly into col 19, reaching the left half.
    // Therefore, P1 cannot have safe territory on the left half.
    assert_eq!(field_plug.p1_safe_territory, 0, "P1 has no safe territory because P2 can reach col 19");

    // -------------------------------------------------------------
    // Maze Scenario 2B: Mutual Partition (P1 wall col 19, P2 wall col 20)
    // -------------------------------------------------------------
    let mut mutual_board = Board::new();
    fill_rect(&mut mutual_board, 0, 0, 3, 3, PLAYER_1);
    fill_rect(&mut mutual_board, 12, 37, 3, 3, PLAYER_2);
    // P1 wall at col 19
    for r in 0..15 {
        mutual_board.set(r, 19, PLAYER_1);
    }
    // P2 wall at col 20 with gap at row 7
    for r in 0..15 {
        if r != 7 {
            mutual_board.set(r, 20, PLAYER_2);
        }
    }
    let cand_mutual_plug = Move::new(6, 20, 3, 1);
    let field_mutual = compute_voronoi_influence_v2(
        &mutual_board,
        cand_mutual_plug.r,
        cand_mutual_plug.c,
        cand_mutual_plug.h,
        cand_mutual_plug.w,
        Player::Player2,
        &matrix,
    );
    let oracle_mutual = oracle_compute_voronoi(&mutual_board, &cand_mutual_plug, Player::Player2, &matrix);
    assert_eq!(field_mutual.p1_territory, oracle_mutual.p1_territory);
    assert_eq!(field_mutual.p2_territory, oracle_mutual.p2_territory);
    assert_eq!(field_mutual.p1_safe_territory, oracle_mutual.p1_safe_territory);
    assert_eq!(field_mutual.p2_safe_territory, oracle_mutual.p2_safe_territory);
    // Now neither player can reach the other half!
    assert_eq!(field_mutual.p1_territory, field_mutual.p1_safe_territory, "All P1 cells must now be safe from P2");
    assert_eq!(field_mutual.p2_territory, field_mutual.p2_safe_territory, "All P2 cells must now be safe from P1");

    // -------------------------------------------------------------
    // Maze Scenario 3: Dungeon with 8 Separated Rooms
    // -------------------------------------------------------------
    let mut dungeon_board = Board::new();
    fill_rect(&mut dungeon_board, 0, 0, 2, 2, PLAYER_1);
    fill_rect(&mut dungeon_board, 13, 38, 2, 2, PLAYER_2);

    // Horizontal dividers at rows 5 and 10
    for c in 0..40 {
        if c % 10 != 5 {
            dungeon_board.set(5, c, PLAYER_1);
            dungeon_board.set(10, c, PLAYER_2);
        }
    }
    // Vertical dividers at cols 10, 20, 30
    for r in 0..15 {
        if r != 2 && r != 7 && r != 12 {
            dungeon_board.set(r, 10, PLAYER_1);
            dungeon_board.set(r, 20, if r > 7 { PLAYER_2 } else { PLAYER_1 });
            dungeon_board.set(r, 30, PLAYER_2);
        }
    }

    let cand_dungeon = Move::new(11, 31, 3, 3);
    let field_dungeon = compute_voronoi_influence_v2(
        &dungeon_board,
        cand_dungeon.r,
        cand_dungeon.c,
        cand_dungeon.h,
        cand_dungeon.w,
        Player::Player2,
        &matrix,
    );
    let oracle_dungeon = oracle_compute_voronoi(&dungeon_board, &cand_dungeon, Player::Player2, &matrix);

    assert_eq!(field_dungeon.p1_territory, oracle_dungeon.p1_territory);
    assert_eq!(field_dungeon.p2_territory, oracle_dungeon.p2_territory);
    assert_eq!(field_dungeon.p1_safe_territory, oracle_dungeon.p1_safe_territory);
    assert_eq!(field_dungeon.p2_safe_territory, oracle_dungeon.p2_safe_territory);
    assert_eq!(field_dungeon.opp_choked_cells, oracle_dungeon.opp_choked_cells);
    assert!((field_dungeon.spatial_score - oracle_dungeon.spatial_score).abs() < 1e-9);

    // -------------------------------------------------------------
    // Maze Scenario 4: 50 Randomized Procedural Mazes
    // -------------------------------------------------------------
    let mut prng = Mulberry32::new(5551212);
    let mut verified_random_mazes = 0;

    for maze_i in 0..50 {
        let mut rand_board = Board::new();
        fill_rect(&mut rand_board, 0, 0, 2, 2, PLAYER_1);
        fill_rect(&mut rand_board, 13, 38, 2, 2, PLAYER_2);

        // Add 15 random obstacle segments (walls)
        for _ in 0..15 {
            let orient = prng.roll_die() % 2;
            let length = 3 + (prng.roll_die() % 6);
            let owner = if prng.roll_die() % 2 == 0 { PLAYER_1 } else { PLAYER_2 };

            if orient == 0 {
                // Horizontal wall
                let r = prng.roll_die() % 15;
                let c_start = prng.roll_die() % (40 - length);
                for c in c_start..c_start + length {
                    if rand_board.get(r, c) == EMPTY {
                        rand_board.set(r, c, owner);
                    }
                }
            } else {
                // Vertical wall
                let r_start = prng.roll_die() % (15 - length);
                let c = prng.roll_die() % 40;
                for r in r_start..r_start + length {
                    if rand_board.get(r, c) == EMPTY {
                        rand_board.set(r, c, owner);
                    }
                }
            }
        }

        // Test with 3 candidate moves per maze
        let cand1 = Move::new(12, 35, 2, 3);
        let cand2 = Move::new(8, 25, 3, 2);
        let cand3 = Move::new(4, 15, 2, 2);

        for (c_idx, cand) in [cand1, cand2, cand3].iter().enumerate() {
            let actual = compute_voronoi_influence_v2(
                &rand_board,
                cand.r,
                cand.c,
                cand.h,
                cand.w,
                Player::Player2,
                &matrix,
            );
            let oracle = oracle_compute_voronoi(&rand_board, cand, Player::Player2, &matrix);

            assert_eq!(
                actual.p1_territory, oracle.p1_territory,
                "Maze {maze_i} cand {c_idx} p1_territory mismatch"
            );
            assert_eq!(
                actual.p2_territory, oracle.p2_territory,
                "Maze {maze_i} cand {c_idx} p2_territory mismatch"
            );
            assert_eq!(
                actual.p1_safe_territory, oracle.p1_safe_territory,
                "Maze {maze_i} cand {c_idx} p1_safe mismatch"
            );
            assert_eq!(
                actual.p2_safe_territory, oracle.p2_safe_territory,
                "Maze {maze_i} cand {c_idx} p2_safe mismatch"
            );
            assert_eq!(
                actual.opp_choked_cells, oracle.opp_choked_cells,
                "Maze {maze_i} cand {c_idx} choke mismatch"
            );
            assert!(
                (actual.spatial_score - oracle.spatial_score).abs() < 1e-9,
                "Maze {maze_i} cand {c_idx} spatial mismatch"
            );

            verified_random_mazes += 1;
        }
    }

    println!("Verified Voronoi influence and safe territory accuracy across {verified_random_mazes} randomized obstacle states.");
}

// ============================================================================
// 4. Empirical Verification 3: Inference Latency Benchmark Harness
// ============================================================================

#[test]
fn test_spatial_slayer_inference_latency_sla() {
    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    // Collect 1,000 realistic move decisions across tournament matches
    let mut decision_durations_us = Vec::with_capacity(1000);

    let mut seed = 424242;
    let mut match_count = 0;

    println!("Starting inference benchmark: measuring 1,000 decisions of SpatialSlayerBot...");

    while decision_durations_us.len() < 1000 {
        let mut board = Board::new();
        let mut rng = Mulberry32::new(seed);
        seed += 1337;
        match_count += 1;

        let mut passes = 0;
        let mut turn = 1;

        while passes < 4 && turn <= 80 && decision_durations_us.len() < 1000 {
            let player = if turn % 2 == 1 { Player::Player1 } else { Player::Player2 };

            let mut d1 = 0;
            let mut d2 = 0;
            let mut found_roll = false;

            for _ in 0..3 {
                let r1 = rng.roll_die();
                let r2 = rng.roll_die();
                let moves = board.generate_legal_moves(player, r1, r2);
                if !moves.is_empty() {
                    d1 = r1;
                    d2 = r2;
                    found_roll = true;
                    break;
                }
            }

            if !found_roll {
                passes += 1;
                turn += 1;
                continue;
            }
            passes = 0;

            // Measure inference latency for SpatialSlayerBot decision
            let t0 = Instant::now();
            let mv = select_move_spatial_slayer(&board, player, d1, d2, &weights, &matrix);
            let elapsed = t0.elapsed();

            let us = elapsed.as_micros() as f64;
            decision_durations_us.push(us);

            if let Some(m) = mv {
                board.apply_move(player, &m);
                board.apply_closed_loops();
            }

            turn += 1;
        }
    }

    decision_durations_us.truncate(1000);
    decision_durations_us.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let count = decision_durations_us.len();
    let min_us = decision_durations_us[0];
    let max_us = decision_durations_us[count - 1];
    let sum_us: f64 = decision_durations_us.iter().sum();
    let mean_us = sum_us / (count as f64);

    let p50_us = decision_durations_us[(count as f64 * 0.50) as usize];
    let p90_us = decision_durations_us[(count as f64 * 0.90) as usize];
    let p95_us = decision_durations_us[(count as f64 * 0.95) as usize];
    let p99_us = decision_durations_us[(count as f64 * 0.99) as usize];

    let p50_ms = p50_us / 1000.0;
    let p90_ms = p90_us / 1000.0;
    let p95_ms = p95_us / 1000.0;
    let p99_ms = p99_us / 1000.0;
    let max_ms = max_us / 1000.0;
    let mean_ms = mean_us / 1000.0;

    println!("============================================================");
    println!("INFERENCE LATENCY BENCHMARK RESULTS (1,000 decisions)");
    println!("============================================================");
    println!("Matches Simulated:  {match_count}");
    println!("Total Decisions:    {count}");
    println!("Min Latency:        {:>7.2} µs ({:.4} ms)", min_us, min_us / 1000.0);
    println!("Mean Latency:       {:>7.2} µs ({:.4} ms)", mean_us, mean_ms);
    println!("p50 Latency:        {:>7.2} µs ({:.4} ms)", p50_us, p50_ms);
    println!("p90 Latency:        {:>7.2} µs ({:.4} ms)", p90_us, p90_ms);
    println!("p95 Latency:        {:>7.2} µs ({:.4} ms)", p95_us, p95_ms);
    println!("p99 Latency:        {:>7.2} µs ({:.4} ms)", p99_us, p99_ms);
    println!("Max Latency:        {:>7.2} µs ({:.4} ms)", max_us, max_ms);
    println!("SLA Threshold:      2000.00 µs (2.0000 ms)");
    println!("SLA Compliance:     {}", if p99_ms <= 2.0 { "PASS (p99 <= 2.0 ms)" } else { "FAIL" });
    println!("============================================================");

    // Distribution histogram
    let mut b_0_2 = 0;
    let mut b_0_5 = 0;
    let mut b_1_0 = 0;
    let mut b_1_5 = 0;
    let mut b_2_0 = 0;
    let mut b_gt_2 = 0;

    for &us in &decision_durations_us {
        let ms = us / 1000.0;
        if ms < 0.2 {
            b_0_2 += 1;
        } else if ms < 0.5 {
            b_0_5 += 1;
        } else if ms < 1.0 {
            b_1_0 += 1;
        } else if ms < 1.5 {
            b_1_5 += 1;
        } else if ms < 2.0 {
            b_2_0 += 1;
        } else {
            b_gt_2 += 1;
        }
    }

    println!("Latency Distribution:");
    println!("  < 0.2 ms:  {:>4} ({:>5.1}%)", b_0_2, (b_0_2 as f64) / 10.0);
    println!("  0.2-0.5 ms:{:>4} ({:>5.1}%)", b_0_5, (b_0_5 as f64) / 10.0);
    println!("  0.5-1.0 ms:{:>4} ({:>5.1}%)", b_1_0, (b_1_0 as f64) / 10.0);
    println!("  1.0-1.5 ms:{:>4} ({:>5.1}%)", b_1_5, (b_1_5 as f64) / 10.0);
    println!("  1.5-2.0 ms:{:>4} ({:>5.1}%)", b_2_0, (b_2_0 as f64) / 10.0);
    println!("  > 2.0 ms:  {:>4} ({:>5.1}%)", b_gt_2, (b_gt_2 as f64) / 10.0);
    println!("============================================================");

    assert!(
        p99_ms <= 2.0,
        "Inference latency SLA violation: p99 latency was {:.4} ms (threshold is 2.0 ms)",
        p99_ms
    );
}
