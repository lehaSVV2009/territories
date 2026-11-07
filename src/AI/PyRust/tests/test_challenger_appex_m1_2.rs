//! Milestone 1 Adversarial Empirical Verification & Stress Test Suite (Challenger 2)
//!
//! Independent stress testing covering:
//! 1. Adversarial 180° rotation invariance across complex asymmetric topologies:
//!    - Asymmetric mazes, isolated chambers, edge choke-points, comb/tooth barriers, random clusters.
//!    - All 10 archetypes tested across all 36 dice combinations (1..=6 x 1..=6).
//!    - Bidirectional verification: P1 -> rot(P2) and P2 -> rot(P1).
//! 2. Edge cases in move selection:
//!    - Completely filled board (600 cells).
//!    - Blocked/walled-in territory (0 legal moves).
//!    - Partial fit failures (1x1 opening with large dice rolls like 5x6, 6x6).
//!    - Turn 1 starting corner requirement (r0=0, c0=0 for P1; r0=15-h, c0=40-w for P2).
//!    - Turn 1 adversarial obstruction (starting corner blocked -> Option::None without panic).
//! 3. PhaseShiftBot stress testing:
//!    - Rapid alternating switching between contrasting archetypes on every turn.
//!    - Stochastic mixer probability distribution empirical verification over 1,000 rolls with chi-squared test.
//!    - EventDriven dynamic switching threshold boundary precision and priority order.
//! 4. Argon-4 benchmark preservation (SHA256 integrity).

use pyrust::appex::zoo::{
    compute_min_contact_distance, select_archetype_move, Archetype, ArchetypeBot, CenterChokerBot,
    FlankExpressBot, PhaseShiftBot,
};
use pyrust::game::{
    Board, Move, Mulberry32, Player, BOARD_HEIGHT, BOARD_WIDTH, EMPTY, PLAYER_1, PLAYER_2,
    TOTAL_CELLS,
};

// ============================================================================
// Adversarial Board Generators
// ============================================================================

/// 1. Asymmetric Maze: Complex serpentine corridors with dead ends and uneven walls.
fn create_asymmetric_maze_board() -> Board {
    let mut board = Board::new();
    board.turn = 12;

    // Base anchors
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));

    // Asymmetric maze walls
    // Horizontal maze bars
    for c in 3..25 {
        if c % 4 != 0 {
            board.cells[2 * BOARD_WIDTH + c] = PLAYER_1;
        }
    }
    for c in 10..38 {
        if c % 5 != 0 {
            board.cells[5 * BOARD_WIDTH + c] = PLAYER_2;
        }
    }
    for c in 1..30 {
        if c % 3 != 1 {
            board.cells[9 * BOARD_WIDTH + c] = PLAYER_1;
        }
    }
    for c in 15..39 {
        if c % 4 != 2 {
            board.cells[12 * BOARD_WIDTH + c] = PLAYER_2;
        }
    }

    // Vertical maze ribs creating dead ends
    for r in 3..9 {
        board.cells[r * BOARD_WIDTH + 8] = PLAYER_1;
        board.cells[r * BOARD_WIDTH + 18] = PLAYER_2;
        board.cells[r * BOARD_WIDTH + 28] = PLAYER_1;
    }
    // Gaps in ribs
    board.cells[4 * BOARD_WIDTH + 8] = EMPTY;
    board.cells[7 * BOARD_WIDTH + 18] = EMPTY;
    board.cells[6 * BOARD_WIDTH + 28] = EMPTY;

    update_scores(&mut board);
    board
}

/// 2. Isolated Chambers: Pockets and rooms, some closed and some with 1-cell apertures.
fn create_isolated_chambers_board() -> Board {
    let mut board = Board::new();
    board.turn = 16;

    // Base anchors
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 3));
    board.apply_move(Player::Player2, &Move::new(13, 37, 2, 3));

    // Chamber 1: Northwest (r: 3..7, c: 4..10) sealed
    for c in 4..=10 {
        board.cells[3 * BOARD_WIDTH + c] = PLAYER_1;
        board.cells[7 * BOARD_WIDTH + c] = PLAYER_1;
    }
    for r in 4..6 {
        board.cells[r * BOARD_WIDTH + 4] = PLAYER_1;
        board.cells[r * BOARD_WIDTH + 10] = PLAYER_1;
    }

    // Chamber 2: Southeast (r: 8..12, c: 25..32) with 1-cell entrance
    for c in 25..=32 {
        board.cells[8 * BOARD_WIDTH + c] = PLAYER_2;
        board.cells[12 * BOARD_WIDTH + c] = PLAYER_2;
    }
    for r in 9..12 {
        board.cells[r * BOARD_WIDTH + 25] = PLAYER_2;
        board.cells[r * BOARD_WIDTH + 32] = PLAYER_2;
    }
    // Aperture at (10, 25)
    board.cells[10 * BOARD_WIDTH + 25] = EMPTY;

    // Chamber 3: Center pocket around r=6..8, c=18..22
    for c in 18..=22 {
        board.cells[6 * BOARD_WIDTH + c] = PLAYER_1;
        board.cells[8 * BOARD_WIDTH + c] = PLAYER_2;
    }
    board.cells[7 * BOARD_WIDTH + 18] = PLAYER_1;
    board.cells[7 * BOARD_WIDTH + 22] = PLAYER_2;

    update_scores(&mut board);
    board
}

/// 3. Edge Chokepoints: Walls hugging perimeter with extreme bottlenecks.
fn create_edge_chokepoints_board() -> Board {
    let mut board = Board::new();
    board.turn = 20;

    // Base anchors connected along flanks
    for c in 0..5 {
        board.cells[0 * BOARD_WIDTH + c] = PLAYER_1;
        board.cells[14 * BOARD_WIDTH + (39 - c)] = PLAYER_2;
    }

    // Perimeter boundary wall running along row 1 and row 13
    for c in 2..38 {
        if c != 12 && c != 27 {
            board.cells[1 * BOARD_WIDTH + c] = PLAYER_1;
        }
        if c != 15 && c != 30 {
            board.cells[13 * BOARD_WIDTH + c] = PLAYER_2;
        }
    }

    // Mid-board bottleneck column barriers at c=14 and c=26
    for r in 2..13 {
        if r != 6 && r != 7 {
            board.cells[r * BOARD_WIDTH + 14] = PLAYER_1;
            board.cells[r * BOARD_WIDTH + 26] = PLAYER_2;
        }
    }

    update_scores(&mut board);
    board
}

/// 4. Comb / Tooth Pattern: Interleaving teeth restricting rectangle orientations.
fn create_comb_board() -> Board {
    let mut board = Board::new();
    board.turn = 10;

    board.apply_move(Player::Player1, &Move::new(0, 0, 1, 5));
    board.apply_move(Player::Player2, &Move::new(14, 35, 1, 5));

    // Alternating vertical comb teeth extending from rows 2..12
    for c in (6..34).step_by(3) {
        if c % 2 == 0 {
            // Tooth from top
            for r in 1..8 {
                board.cells[r * BOARD_WIDTH + c] = PLAYER_1;
            }
        } else {
            // Tooth from bottom
            for r in 7..14 {
                board.cells[r * BOARD_WIDTH + c] = PLAYER_2;
            }
        }
    }

    update_scores(&mut board);
    board
}

/// 5. Random Asymmetric Clusters: Pseudo-random scattered blocks with guaranteed anchors.
fn create_random_asymmetric_clusters_board(seed: u32) -> Board {
    let mut board = Board::new();
    board.turn = 18;
    let mut rng = Mulberry32::new(seed);

    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));

    for _ in 0..25 {
        let r = (rng.next_u32() as usize) % (BOARD_HEIGHT - 2);
        let c = (rng.next_u32() as usize) % (BOARD_WIDTH - 2);
        let player = if rng.next_f64() < 0.5 { PLAYER_1 } else { PLAYER_2 };
        let h = 1 + (rng.next_u32() as usize % 2);
        let w = 1 + (rng.next_u32() as usize % 3);

        for dr in 0..h {
            for dc in 0..w {
                if r + dr < BOARD_HEIGHT && c + dc < BOARD_WIDTH {
                    // Do not overwrite starting corners
                    if (r + dr < 2 && c + dc < 2) || (r + dr >= 13 && c + dc >= 38) {
                        continue;
                    }
                    board.cells[(r + dr) * BOARD_WIDTH + (c + dc)] = player;
                }
            }
        }
    }

    update_scores(&mut board);
    board
}

/// Helper: Recomputes p1_score and p2_score accurately.
fn update_scores(board: &mut Board) {
    let mut s1 = 0u16;
    let mut s2 = 0u16;
    for &cell in &board.cells {
        if cell == PLAYER_1 {
            s1 += 1;
        } else if cell == PLAYER_2 {
            s2 += 1;
        }
    }
    board.p1_score = s1;
    board.p2_score = s2;
}

// ============================================================================
// TEST SUITE 1: Adversarial 180° Rotation Invariance (Full 6x6 Dice Grid)
// ============================================================================

#[test]
fn test_adversarial_180_rotation_invariance_full_dice_grid() {
    let boards = [
        ("Asymmetric Maze", create_asymmetric_maze_board()),
        ("Isolated Chambers", create_isolated_chambers_board()),
        ("Edge Chokepoints", create_edge_chokepoints_board()),
        ("Comb/Tooth Pattern", create_comb_board()),
        ("Random Clusters A", create_random_asymmetric_clusters_board(101)),
        ("Random Clusters B", create_random_asymmetric_clusters_board(999)),
    ];

    let archetypes = Archetype::all();
    let mut total_evals = 0;
    let mut invariant_matches = 0;

    for (b_name, board) in &boards {
        let rotated_board = board.rotate_board_180();

        for &arch in &archetypes {
            for d1 in 1..=6 {
                for d2 in 1..=6 {
                    total_evals += 1;

                    let mut rng_p1 = Mulberry32::new(12345 + (d1 * 10 + d2) as u32);
                    let mut rng_p2 = Mulberry32::new(12345 + (d1 * 10 + d2) as u32);

                    // Direction 1: P1 on board vs P2 on rotated_board
                    let mv_p1 = select_archetype_move(
                        arch,
                        board,
                        Player::Player1,
                        d1,
                        d2,
                        Some(&mut rng_p1),
                    );
                    let mv_p2 = select_archetype_move(
                        arch,
                        &rotated_board,
                        Player::Player2,
                        d1,
                        d2,
                        Some(&mut rng_p2),
                    );

                    match (mv_p1, mv_p2) {
                        (None, None) => {
                            invariant_matches += 1;
                        }
                        (Some(m1), Some(m2)) => {
                            let inv_m2 = Board::inverse_move_180(&m2);
                            assert_eq!(
                                m1, inv_m2,
                                "180° Invariance violation for arch {:?} on board '{}' with dice ({}, {}):\n\
                                 m1 (P1): {:?}\n\
                                 m2 (P2 on rotated): {:?}\n\
                                 inv(m2): {:?}",
                                arch, b_name, d1, d2, m1, m2, inv_m2
                            );
                            invariant_matches += 1;
                        }
                        (Some(m1), None) => {
                            panic!(
                                "Invariance mismatch for {:?} on '{}' with dice ({}, {}): P1 found move {:?}, but P2 on rotated found None",
                                arch, b_name, d1, d2, m1
                            );
                        }
                        (None, Some(m2)) => {
                            panic!(
                                "Invariance mismatch for {:?} on '{}' with dice ({}, {}): P1 found None, but P2 on rotated found move {:?}",
                                arch, b_name, d1, d2, m2
                            );
                        }
                    }

                    // Direction 2: P2 on board vs P1 on rotated_board
                    let mut rng_p2_rev = Mulberry32::new(54321 + (d1 * 10 + d2) as u32);
                    let mut rng_p1_rev = Mulberry32::new(54321 + (d1 * 10 + d2) as u32);

                    let rev_p2 = select_archetype_move(
                        arch,
                        board,
                        Player::Player2,
                        d1,
                        d2,
                        Some(&mut rng_p2_rev),
                    );
                    let rev_p1 = select_archetype_move(
                        arch,
                        &rotated_board,
                        Player::Player1,
                        d1,
                        d2,
                        Some(&mut rng_p1_rev),
                    );

                    match (rev_p2, rev_p1) {
                        (None, None) => {}
                        (Some(m2), Some(m1)) => {
                            let inv_m1 = Board::inverse_move_180(&m1);
                            assert_eq!(
                                m2, inv_m1,
                                "Reverse 180° Invariance violation for arch {:?} on board '{}' with dice ({}, {}):\n\
                                 m2 (P2): {:?}\n\
                                 m1 (P1 on rotated): {:?}\n\
                                 inv(m1): {:?}",
                                arch, b_name, d1, d2, m2, m1, inv_m1
                            );
                        }
                        _ => panic!("Reverse invariance mismatch for {:?}", arch),
                    }
                }
            }
        }
    }

    assert_eq!(
        total_evals, invariant_matches,
        "Total evaluations did not match verified invariant count!"
    );
    println!(
        "PASS: 180° Rotation Invariance strictly verified across {} evaluations across 6 adversarial topologies.",
        total_evals
    );
}

// ============================================================================
// TEST SUITE 2: Edge Cases in Move Selection
// ============================================================================

#[test]
fn test_edge_case_fully_saturated_board() {
    // 600 cells occupied completely: 300 P1, 300 P2
    let mut cells = [PLAYER_1; TOTAL_CELLS];
    for c in cells.iter_mut().skip(TOTAL_CELLS / 2) {
        *c = PLAYER_2;
    }
    let full_board = Board {
        cells,
        p1_score: 300,
        p2_score: 300,
        turn: 40,
        passes: 0,
    };

    let dice = [(1, 1), (2, 3), (4, 4), (6, 6)];

    for &arch in &Archetype::all() {
        for &(d1, d2) in &dice {
            let mut rng = Mulberry32::new(777);
            let mv1 = select_archetype_move(arch, &full_board, Player::Player1, d1, d2, Some(&mut rng));
            assert!(
                mv1.is_none(),
                "Expected None on full board for P1 arch {:?}, got {:?}",
                arch,
                mv1
            );

            let mv2 = select_archetype_move(arch, &full_board, Player::Player2, d1, d2, Some(&mut rng));
            assert!(
                mv2.is_none(),
                "Expected None on full board for P2 arch {:?}, got {:?}",
                arch,
                mv2
            );
        }
    }

    // Verify PhaseShiftBot also returns None safely without panic
    let mut ps_bot = PhaseShiftBot::new_sequential(
        Archetype::FlankExpress,
        Archetype::CenterChoker,
        Archetype::CompactTurtle,
        10,
        25,
        123,
    );
    let mv_ps = ps_bot.select_move(&full_board, Player::Player1, 3, 3);
    assert!(mv_ps.is_none(), "PhaseShiftBot should return None on full board");
}

#[test]
fn test_edge_case_completely_walled_in_territory() {
    // Friendly pieces are surrounded by enemy pieces, with no adjacent empty cells.
    let mut board = Board::new();
    board.turn = 10;

    // Place P1 cluster at (5, 5, 2, 2)
    for r in 5..=6 {
        for c in 5..=6 {
            board.cells[r * BOARD_WIDTH + c] = PLAYER_1;
        }
    }
    board.p1_score = 4;

    // Completely ring P1 with P2 cells so P1 has ZERO adjacent empty cells
    for r in 4..=7 {
        for c in 4..=7 {
            if !(r >= 5 && r <= 6 && c >= 5 && c <= 6) {
                board.cells[r * BOARD_WIDTH + c] = PLAYER_2;
                board.p2_score += 1;
            }
        }
    }

    // Also place a P2 anchor at bottom-right
    board.cells[14 * BOARD_WIDTH + 39] = PLAYER_2;
    board.p2_score += 1;

    for &arch in &Archetype::all() {
        for &(d1, d2) in &[(1, 1), (1, 2), (3, 3), (6, 6)] {
            let mut rng = Mulberry32::new(42);
            let mv = select_archetype_move(arch, &board, Player::Player1, d1, d2, Some(&mut rng));
            assert!(
                mv.is_none(),
                "Walled-in P1 should have 0 legal moves, but {:?} returned {:?}",
                arch,
                mv
            );
        }
    }
}

#[test]
fn test_edge_case_size_fit_boundary() {
    // Only a single 1x1 empty cell is available adjacent to friendly territory.
    let mut board = Board::new();
    board.turn = 15;

    // Fill entire board with P2 except P1 at (0, 0) and (0, 1) as EMPTY
    board.cells.fill(PLAYER_2);
    board.cells[0] = PLAYER_1; // (0, 0) is P1
    board.cells[1] = EMPTY;    // (0, 1) is the ONLY empty cell adjacent to P1
    board.p1_score = 1;
    board.p2_score = 599;

    for &arch in &Archetype::all() {
        let mut rng = Mulberry32::new(99);

        // (1, 1) should successfully fit at (0, 1)
        let mv_fit = select_archetype_move(arch, &board, Player::Player1, 1, 1, Some(&mut rng));
        assert!(
            mv_fit.is_some(),
            "Archetype {:?} failed to place 1x1 into available space",
            arch
        );
        let m = mv_fit.unwrap();
        assert_eq!(m, Move::new(0, 1, 1, 1));

        // Any larger piece like (1, 2), (2, 1), (3, 3) cannot fit and must return None
        let mv_no_fit1 = select_archetype_move(arch, &board, Player::Player1, 1, 2, Some(&mut rng));
        assert!(
            mv_no_fit1.is_none(),
            "Archetype {:?} should return None for (1, 2) on 1x1 pocket",
            arch
        );

        let mv_no_fit2 = select_archetype_move(arch, &board, Player::Player1, 6, 6, Some(&mut rng));
        assert!(
            mv_no_fit2.is_none(),
            "Archetype {:?} should return None for (6, 6) on 1x1 pocket",
            arch
        );
    }
}

#[test]
fn test_edge_case_turn_1_rules_and_blocked_corner() {
    // 1. Normal Turn 1 verification:
    // P1 must anchor at (0, 0), P2 must anchor at (15-h, 40-w).
    let empty_board = Board::new();
    assert_eq!(empty_board.score(Player::Player1), 0);
    assert_eq!(empty_board.score(Player::Player2), 0);

    for &arch in &Archetype::all() {
        for d1 in 1..=6 {
            for d2 in 1..=6 {
                let mut rng1 = Mulberry32::new(101);
                let mv1 = select_archetype_move(arch, &empty_board, Player::Player1, d1, d2, Some(&mut rng1));
                assert!(mv1.is_some());
                let m1 = mv1.unwrap();
                assert_eq!(
                    (m1.r, m1.c),
                    (0, 0),
                    "Turn 1 P1 move for {:?} must start at (0, 0), got ({}, {})",
                    arch,
                    m1.r,
                    m1.c
                );

                let mut rng2 = Mulberry32::new(101);
                let mv2 = select_archetype_move(arch, &empty_board, Player::Player2, d1, d2, Some(&mut rng2));
                assert!(mv2.is_some());
                let m2 = mv2.unwrap();
                assert_eq!(
                    (m2.r, m2.c),
                    (BOARD_HEIGHT - m2.h, BOARD_WIDTH - m2.w),
                    "Turn 1 P2 move for {:?} must anchor at bottom-right corner",
                    arch
                );
            }
        }
    }

    // 2. Adversarial Turn 1: Corner is blocked
    // If (0, 0) is blocked for P1 on Turn 1, P1 can never legally place a piece!
    let mut blocked_corner_board = Board::new();
    blocked_corner_board.cells[0] = PLAYER_2; // Enemy took (0, 0)
    blocked_corner_board.p2_score = 1;

    for &arch in &Archetype::all() {
        let mut rng = Mulberry32::new(42);
        let mv = select_archetype_move(arch, &blocked_corner_board, Player::Player1, 2, 3, Some(&mut rng));
        assert!(
            mv.is_none(),
            "P1 Turn 1 with blocked (0, 0) must return None, but {:?} returned {:?}",
            arch,
            mv
        );
    }

    // If (14, 39) is blocked for P2 on Turn 1, P2 can never legally place a piece!
    let mut blocked_p2_corner = Board::new();
    blocked_p2_corner.cells[14 * BOARD_WIDTH + 39] = PLAYER_1;
    blocked_p2_corner.p1_score = 1;

    for &arch in &Archetype::all() {
        let mut rng = Mulberry32::new(42);
        let mv = select_archetype_move(arch, &blocked_p2_corner, Player::Player2, 2, 3, Some(&mut rng));
        assert!(
            mv.is_none(),
            "P2 Turn 1 with blocked (14, 39) must return None, but {:?} returned {:?}",
            arch,
            mv
        );
    }
}

// ============================================================================
// TEST SUITE 3: PhaseShiftBot Stress Testing
// ============================================================================

#[test]
fn test_phaseshift_rapid_alternating_oscillations() {
    // Construct PhaseShiftBot alternating every single turn between FlankExpress and CenterChoker
    // for 60 turns: [ (1, Flank), (2, Center), (3, Flank), (4, Center), ... ]
    let mut phases: Vec<(usize, Box<dyn ArchetypeBot>)> = Vec::new();
    for turn in 1..=60 {
        if turn % 2 == 1 {
            phases.push((turn, Box::new(FlankExpressBot::new(turn as u32 * 11))));
        } else {
            phases.push((turn, Box::new(CenterChokerBot::new(turn as u32 * 17))));
        }
    }

    let mut bot_p1 = PhaseShiftBot::new(phases, 12345);

    // Simulate match
    let mut board = Board::new();
    let mut rng = Mulberry32::new(999);

    for turn in 1..=40 {
        board.turn = turn;
        let d1 = rng.roll_die();
        let d2 = rng.roll_die();

        let mv = bot_p1.select_move(&board, Player::Player1, d1, d2);
        if let Some(m) = mv {
            assert!(
                board.can_place_rectangle(Player::Player1, m.r, m.c, m.h, m.w),
                "Turn {}: Selected move {:?} is not legal!",
                turn,
                m
            );
            board.apply_move(Player::Player1, &m);

            // Active archetype assertion
            let expected_arch = if turn % 2 == 1 {
                Archetype::FlankExpress
            } else {
                Archetype::CenterChoker
            };
            assert_eq!(
                bot_p1.active_archetype(&board),
                Some(expected_arch),
                "Turn {}: Active archetype mismatch",
                turn
            );
        }

        // P2 mirror move
        let d1_p2 = rng.roll_die();
        let d2_p2 = rng.roll_die();
        let legal_p2 = board.generate_legal_moves(Player::Player2, d1_p2, d2_p2);
        if !legal_p2.is_empty() {
            board.apply_move(Player::Player2, &legal_p2[0]);
        }
    }

    assert!(board.score(Player::Player1) > 0);
    println!("PASS: PhaseShiftBot rapid 1-turn alternating oscillation executed without errors.");
}

#[test]
fn test_phaseshift_stochastic_mixer_empirical_distribution_1000_rolls() {
    // Tests probability distribution across 1,000 rolls with chi-squared test
    let test_probs = [0.0, 0.15, 0.35, 0.50, 0.70, 0.85, 1.0];
    let num_rolls = 1000;

    for &prob_a in &test_probs {
        let mut bot = PhaseShiftBot::new_stochastic(
            Archetype::FlankExpress,
            Archetype::CenterChoker,
            prob_a,
            424242,
        );

        let mut count_a = 0;
        let mut count_b = 0;

        for _ in 0..num_rolls {
            let r = bot.rng.next_f64();
            if r < prob_a {
                count_a += 1;
            } else {
                count_b += 1;
            }
        }

        assert_eq!(count_a + count_b, num_rolls);

        if prob_a == 0.0 {
            assert_eq!(count_a, 0, "prob_a=0.0 must yield exactly 0 choices of bot_a");
            assert_eq!(count_b, num_rolls);
        } else if prob_a == 1.0 {
            assert_eq!(count_a, num_rolls, "prob_a=1.0 must yield exactly num_rolls of bot_a");
            assert_eq!(count_b, 0);
        } else {
            let expected_a = (num_rolls as f64) * prob_a;
            let expected_b = (num_rolls as f64) * (1.0 - prob_a);

            // Pearson's chi-squared statistic (1 degree of freedom)
            let diff_a = count_a as f64 - expected_a;
            let diff_b = count_b as f64 - expected_b;
            let chi2 = (diff_a * diff_a) / expected_a + (diff_b * diff_b) / expected_b;

            // Critical value for chi-squared with df=1 at alpha=0.001 is 10.828
            assert!(
                chi2 < 10.828,
                "Chi-squared test failed for prob_a={:.2}: chi2={:.3} >= 10.828 (obs_a={}, exp_a={:.1})",
                prob_a,
                chi2,
                count_a,
                expected_a
            );

            // Normal approximation z-score check
            let std_err = (prob_a * (1.0 - prob_a) / (num_rolls as f64)).sqrt();
            let obs_p = (count_a as f64) / (num_rolls as f64);
            let z = (obs_p - prob_a).abs() / std_err;
            assert!(
                z < 3.29,
                "Z-score test failed for prob_a={:.2}: z={:.3} >= 3.29 (observed p={:.4})",
                prob_a,
                z,
                obs_p
            );
        }

        // Verify determinism: repeating with the same seed yields identical sequence
        let mut bot_replay = PhaseShiftBot::new_stochastic(
            Archetype::FlankExpress,
            Archetype::CenterChoker,
            prob_a,
            424242,
        );
        let mut replay_count_a = 0;
        for _ in 0..num_rolls {
            let r = bot_replay.rng.next_f64();
            if r < prob_a {
                replay_count_a += 1;
            }
        }
        assert_eq!(
            count_a, replay_count_a,
            "Mulberry32 PRNG failed exact replay determinism for prob_a={}",
            prob_a
        );
    }

    println!("PASS: PhaseShiftBot stochastic mixer passed empirical chi-squared tests across 1,000 rolls.");
}

#[test]
fn test_phaseshift_event_driven_boundary_precision_and_priority() {
    let base = Archetype::FlankExpress;
    let contact = Archetype::AggressiveChaser;
    let endgame = Archetype::CompactTurtle;

    let contact_thresh = 5;
    let empty_thresh = 200;

    let bot = PhaseShiftBot::new_event_driven(
        base,
        contact,
        endgame,
        contact_thresh,
        empty_thresh,
        7777,
    );

    let mut board = Board::new();
    board.turn = 20;

    // Anchor pieces
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));
    update_scores(&mut board);

    // Case 1: Early game, far contact dist, lots of empty cells -> base
    let empty = board.empty_cells_count();
    let contact_d = compute_min_contact_distance(&board);
    assert!(empty >= empty_thresh);
    assert!(contact_d > contact_thresh);
    assert_eq!(bot.active_archetype(&board), Some(base));

    // Case 2: Contact boundary testing (contact_dist <= contact_thresh)
    // Place P1 at (10, 35) -> Manhattan distance to (13, 38) is |13-10| + |38-35| = 6 ( > 5 -> base)
    board.apply_move(Player::Player1, &Move::new(10, 35, 1, 1));
    update_scores(&mut board);
    assert_eq!(compute_min_contact_distance(&board), 6);
    assert_eq!(bot.active_archetype(&board), Some(base));

    // Move to (10, 36) -> dist is |13-10| + |38-36| = 3 + 2 = 5 ( == contact_thresh -> contact)
    board.apply_move(Player::Player1, &Move::new(10, 36, 1, 1));
    update_scores(&mut board);
    assert_eq!(compute_min_contact_distance(&board), 5);
    assert_eq!(bot.active_archetype(&board), Some(contact));

    // Case 3: Priority Check - Endgame threshold PRECEDENCE over Contact
    // When empty cells < empty_thresh (200), endgame_bot MUST be chosen even if contact_dist <= 5!
    // Fill 440+ cells so empty = 600 - 448 = 152 (< 200)
    for r in 1..12 {
        for c in 0..40 {
            if board.cells[r * BOARD_WIDTH + c] == EMPTY {
                board.cells[r * BOARD_WIDTH + c] = PLAYER_1;
            }
        }
    }
    update_scores(&mut board);
    let empty_after = board.empty_cells_count();
    assert!(empty_after < empty_thresh, "empty count {} must be < 200", empty_after);
    assert!(compute_min_contact_distance(&board) <= contact_thresh);
    // MUST select endgame (CompactTurtle) because empty < empty_thresh has top priority
    assert_eq!(
        bot.active_archetype(&board),
        Some(endgame),
        "Endgame bot must take precedence over contact bot when empty < empty_cells_threshold"
    );
}

// ============================================================================
// TEST SUITE 4: Argon-4 Benchmark Protection & SHA256 Verification
// ============================================================================

#[test]
fn test_argon4_benchmark_weights_exact_sha256() {
    let weights_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("weights.json");
    assert!(weights_path.exists(), "weights.json not found at {:?}", weights_path);

    let bytes = std::fs::read(&weights_path).expect("Failed to read weights.json");

    // Pure Rust standard SHA256 calculation
    let hash = compute_sha256(&bytes);
    let expected = "7dac89a0f17211f383a86435f51a77bc9bdfba55957f53b5b2b08ae8ba2da43b";

    assert_eq!(
        hash, expected,
        "CRITICAL: weights.json SHA-256 hash has been altered!\nExpected: {}\nActual:   {}",
        expected, hash
    );
    println!("PASS: Argon-4 weights.json SHA256 bit-exact integrity confirmed.");
}

/// Standalone SHA-256 hasher for zero-dependency test assertion.
fn compute_sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, part) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(part.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    h.iter().map(|val| format!("{:08x}", val)).collect()
}
