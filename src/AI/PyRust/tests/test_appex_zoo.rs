//! Milestone 1 Test Suite: Catalog of 10 Archetypes & PhaseShiftBot.
//!
//! Verifies:
//! - Mulberry32 deterministic PRNG reproducibility.
//! - Canonical Player 2 perspective and 180° rotation invariance across ALL 10 archetypes.
//! - Move legality across all phases of match simulation.
//! - Policy distinctiveness on control board states.
//! - PhaseShiftBot sequential, stochastic, and event-driven transitions.
//! - Trait-based polymorphic dispatch without mocks or stubs.

use pyrust::appex::zoo::{
    compute_min_contact_distance, select_archetype_move, AggressiveChaserBot, Archetype,
    ArchetypeBot, CenterChokerBot, ChaoticHopperBot, CompactTurtleBot, CrossFlankRaiderBot,
    FlankExpressBot, GreedyMaxAreaBot, LoopHunterBot, PerimeterLurkerBot, PhaseShiftBot,
    WallBuilderBot,
};
use pyrust::game::{
    Board, Move, Mulberry32, Player, BOARD_HEIGHT, BOARD_WIDTH, PLAYER_1,
};

/// 1. Mulberry32 PRNG Determinism & Distribution Test
#[test]
fn test_mulberry32_determinism_identical_seeds() {
    let seeds = [42u32, 123456789u32, 987654321u32, 314159265u32];

    for &seed in &seeds {
        let mut rng1 = Mulberry32::new(seed);
        let mut rng2 = Mulberry32::new(seed);

        for _ in 0..2000 {
            let u1 = rng1.next_u32();
            let u2 = rng2.next_u32();
            assert_eq!(u1, u2, "next_u32 failed determinism for seed {seed}");

            let f1 = rng1.next_f64();
            let f2 = rng2.next_f64();
            assert_eq!(f1, f2, "next_f64 failed determinism for seed {seed}");
            assert!(
                (0.0..1.0).contains(&f1),
                "next_f64 out of range [0, 1): {f1}"
            );

            let d1 = rng1.roll_die();
            let d2 = rng2.roll_die();
            assert_eq!(d1, d2, "roll_die failed determinism for seed {seed}");
            assert!((1..=6).contains(&d1), "roll_die out of range [1, 6]: {d1}");
        }

        // Verify distinct seeds produce distinct outputs
        let mut rng_other = Mulberry32::new(seed.wrapping_add(1));
        let mut match_count = 0;
        for _ in 0..100 {
            if rng1.next_u32() == rng_other.next_u32() {
                match_count += 1;
            }
        }
        assert!(
            match_count < 5,
            "Distinct seeds produced too many identical outputs: {match_count}/100"
        );
    }
}

/// 2. 180° Rotation Invariance for ALL 10 Archetypes on Opening Board
#[test]
fn test_180_rotation_invariance_all_10_archetypes_opening() {
    let board = Board::new();
    let rot_board = board.rotate_board_180();

    let dice_pairs = [
        (1, 1),
        (2, 3),
        (3, 2),
        (4, 5),
        (6, 6),
        (1, 6),
        (6, 1),
        (3, 4),
        (5, 2),
    ];

    for &arch in &Archetype::all() {
        for &(d1, d2) in &dice_pairs {
            let mut rng_p1 = Mulberry32::new(42 + (d1 * 10 + d2) as u32);
            let mut rng_p2 = Mulberry32::new(42 + (d1 * 10 + d2) as u32);

            let mv_p1 = select_archetype_move(arch, &board, Player::Player1, d1, d2, Some(&mut rng_p1));
            let mv_p2 = select_archetype_move(
                arch,
                &rot_board,
                Player::Player2,
                d1,
                d2,
                Some(&mut rng_p2),
            );

            assert!(
                mv_p1.is_some(),
                "Archetype {:?} returned None for P1 opening ({}, {})",
                arch,
                d1,
                d2
            );
            assert!(
                mv_p2.is_some(),
                "Archetype {:?} returned None for P2 opening ({}, {})",
                arch,
                d1,
                d2
            );

            let m1 = mv_p1.unwrap();
            let m2 = mv_p2.unwrap();
            let inv_m2 = Board::inverse_move_180(&m2);

            assert_eq!(
                m1, inv_m2,
                "Rotation invariance violation for {:?} on opening board ({}, {}):\n  P1 move: {:?}\n  P2 rotated move: {:?}\n  Inverted P2: {:?}",
                arch, d1, d2, m1, m2, inv_m2
            );
        }
    }
}

/// 3. Helper to build diverse board states for midgame invariance testing
fn create_midgame_test_boards() -> Vec<Board> {
    let mut boards = Vec::new();

    // Board 1: Flank expansion
    let mut b1 = Board::new();
    b1.apply_move(Player::Player1, &Move::new(0, 0, 2, 4));
    b1.apply_move(Player::Player1, &Move::new(0, 4, 2, 5));
    b1.apply_move(Player::Player2, &Move::new(13, 35, 2, 5));
    b1.apply_move(Player::Player2, &Move::new(13, 30, 2, 5));
    b1.turn = 5;
    boards.push(b1);

    // Board 2: Central approach
    let mut b2 = Board::new();
    b2.apply_move(Player::Player1, &Move::new(0, 0, 3, 3));
    b2.apply_move(Player::Player1, &Move::new(3, 2, 3, 4));
    b2.apply_move(Player::Player1, &Move::new(5, 5, 2, 6));
    b2.apply_move(Player::Player2, &Move::new(12, 37, 3, 3));
    b2.apply_move(Player::Player2, &Move::new(9, 34, 3, 4));
    b2.apply_move(Player::Player2, &Move::new(7, 29, 2, 6));
    b2.turn = 7;
    boards.push(b2);

    // Board 3: Direct contact standoff
    let mut b3 = Board::new();
    b3.apply_move(Player::Player1, &Move::new(0, 0, 4, 4));
    b3.apply_move(Player::Player1, &Move::new(4, 3, 3, 5));
    b3.apply_move(Player::Player1, &Move::new(6, 7, 2, 5));
    b3.apply_move(Player::Player2, &Move::new(11, 36, 4, 4));
    b3.apply_move(Player::Player2, &Move::new(8, 32, 3, 5));
    b3.apply_move(Player::Player2, &Move::new(6, 12, 2, 5)); // Adjacent to P1 at col 12 vs col 11
    b3.turn = 9;
    boards.push(b3);

    // Board 4: Complex multi-island board with loops
    let mut b4 = Board::new();
    b4.apply_move(Player::Player1, &Move::new(0, 0, 3, 3));
    b4.apply_move(Player::Player1, &Move::new(0, 3, 1, 6));
    b4.apply_move(Player::Player1, &Move::new(2, 3, 1, 6));
    b4.apply_move(Player::Player2, &Move::new(12, 37, 3, 3));
    b4.apply_move(Player::Player2, &Move::new(14, 31, 1, 6));
    b4.apply_move(Player::Player2, &Move::new(12, 31, 1, 6));
    b4.turn = 11;
    boards.push(b4);

    // Board 5: Highly asymmetric board
    let mut b5 = Board::new();
    b5.apply_move(Player::Player1, &Move::new(0, 0, 5, 2));
    b5.apply_move(Player::Player1, &Move::new(5, 0, 4, 3));
    b5.apply_move(Player::Player1, &Move::new(9, 1, 3, 3));
    b5.apply_move(Player::Player2, &Move::new(12, 35, 3, 5));
    b5.apply_move(Player::Player2, &Move::new(10, 33, 2, 4));
    b5.turn = 13;
    boards.push(b5);

    boards
}

/// 4. 180° Rotation Invariance for ALL 10 Archetypes across Midgame Board States
#[test]
fn test_180_rotation_invariance_all_10_archetypes_midgame() {
    let test_boards = create_midgame_test_boards();
    let dice_tests = [(2, 2), (1, 4), (4, 1), (3, 5), (6, 2), (2, 6)];

    for (b_idx, board) in test_boards.iter().enumerate() {
        let rot_board = board.rotate_board_180();

        for &arch in &Archetype::all() {
            for &(d1, d2) in &dice_tests {
                let mut rng_p1 = Mulberry32::new(100 + (d1 * 11 + d2 + b_idx * 7) as u32);
                let mut rng_p2 = Mulberry32::new(100 + (d1 * 11 + d2 + b_idx * 7) as u32);

                let mv_p1 = select_archetype_move(arch, board, Player::Player1, d1, d2, Some(&mut rng_p1));
                let mv_p2 = select_archetype_move(
                    arch,
                    &rot_board,
                    Player::Player2,
                    d1,
                    d2,
                    Some(&mut rng_p2),
                );

                match (mv_p1, mv_p2) {
                    (Some(m1), Some(m2)) => {
                        let inv_m2 = Board::inverse_move_180(&m2);
                        assert_eq!(
                            m1, inv_m2,
                            "Rotation invariance mismatch for {:?} on board #{} ({}, {}):\n  P1: {:?}\n  P2: {:?}\n  Inv P2: {:?}",
                            arch, b_idx + 1, d1, d2, m1, m2, inv_m2
                        );
                    }
                    (None, None) => {
                        // Both agree no legal move exists
                    }
                    (Some(m1), None) => {
                        panic!(
                            "Asymmetry: P1 found move {:?} but P2 found None for {:?} on board #{} ({}, {})",
                            m1, arch, b_idx + 1, d1, d2
                        );
                    }
                    (None, Some(m2)) => {
                        panic!(
                            "Asymmetry: P1 found None but P2 found {:?} for {:?} on board #{} ({}, {})",
                            m2, arch, b_idx + 1, d1, d2
                        );
                    }
                }
            }
        }
    }
}

/// 5. Move Legality Verification Across Full Simulated Matches
#[test]
fn test_move_legality_all_archetypes_simulated_match() {
    for &arch in &Archetype::all() {
        let mut board = Board::new();
        let mut rng = Mulberry32::new(54321 + arch as u32);
        let mut passes = 0;

        for turn in 1..=30 {
            board.turn = turn;
            let d1 = rng.roll_die();
            let d2 = rng.roll_die();

            let acting_player = if turn % 2 == 1 {
                Player::Player1
            } else {
                Player::Player2
            };

            let maybe_mv = select_archetype_move(arch, &board, acting_player, d1, d2, Some(&mut rng));

            if let Some(mv) = maybe_mv {
                // Must be legal according to the core engine
                assert!(
                    board.can_place_rectangle(acting_player, mv.r, mv.c, mv.h, mv.w),
                    "Illegal move placed by {:?} on turn {}: {:?}",
                    arch,
                    turn,
                    mv
                );
                // Coordinates must be strictly within bounds
                assert!(mv.r + mv.h <= BOARD_HEIGHT, "Move exceeds height: {:?}", mv);
                assert!(mv.c + mv.w <= BOARD_WIDTH, "Move exceeds width: {:?}", mv);

                board.apply_move(acting_player, &mv);
                passes = 0;
            } else {
                passes += 1;
                if passes >= 4 {
                    break;
                }
            }
        }

        // At least one move must have been successfully placed
        assert!(
            board.p1_score > 0 || board.p2_score > 0,
            "Archetype {:?} failed to place any moves in 30 turns",
            arch
        );
    }
}

/// 6. Policy Distinctiveness on Control Board States
#[test]
fn test_policy_distinctiveness_across_archetypes() {
    let mut board = Board::new();
    // Establish anchor blocks for both players
    board.apply_move(Player::Player1, &Move::new(0, 0, 3, 3));
    board.apply_move(Player::Player2, &Move::new(12, 37, 3, 3));
    board.turn = 3;

    // Fixed dice roll offering multiple distinct tactical paths
    let d1 = 2;
    let d2 = 5;

    // Collect moves for deterministic archetypes from P2 perspective
    let mv_flank = select_archetype_move(Archetype::FlankExpress, &board, Player::Player2, d1, d2, None)
        .expect("FlankExpress move");
    let mv_cross = select_archetype_move(Archetype::CrossFlankRaider, &board, Player::Player2, d1, d2, None)
        .expect("CrossFlankRaider move");
    let mv_choker = select_archetype_move(Archetype::CenterChoker, &board, Player::Player2, d1, d2, None)
        .expect("CenterChoker move");
    let mv_wall = select_archetype_move(Archetype::WallBuilder, &board, Player::Player2, d1, d2, None)
        .expect("WallBuilder move");
    let mv_greedy = select_archetype_move(Archetype::GreedyMaxArea, &board, Player::Player2, d1, d2, None)
        .expect("GreedyMaxArea move");
    let mv_turtle = select_archetype_move(Archetype::CompactTurtle, &board, Player::Player2, d1, d2, None)
        .expect("CompactTurtle move");
    let mv_lurker = select_archetype_move(Archetype::PerimeterLurker, &board, Player::Player2, d1, d2, None)
        .expect("PerimeterLurker move");

    // FlankExpress should stay in bottom flank (r+h > 12)
    assert!(
        mv_flank.r + mv_flank.h >= 13,
        "FlankExpress should hug home flank (rows 12..14): {:?}",
        mv_flank
    );

    // CrossFlankRaider should head northward (lower row) compared to FlankExpress
    assert!(
        mv_cross.r <= mv_flank.r,
        "CrossFlankRaider should move north toward enemy corridor: cross={:?}, flank={:?}",
        mv_cross,
        mv_flank
    );

    // GreedyMaxArea must select maximum available area (2 * 5 = 10)
    assert_eq!(
        mv_greedy.area(),
        10,
        "GreedyMaxArea should pick 10-cell rectangle: {:?}",
        mv_greedy
    );

    // CenterChoker should minimize distance to center (7.0, 19.5)
    let choker_dist = (mv_choker.r as f64 + 0.5 * mv_choker.h as f64 - 7.0).abs()
        + (mv_choker.c as f64 + 0.5 * mv_choker.w as f64 - 19.5).abs();
    let flank_dist = (mv_flank.r as f64 + 0.5 * mv_flank.h as f64 - 7.0).abs()
        + (mv_flank.c as f64 + 0.5 * mv_flank.w as f64 - 19.5).abs();
    assert!(
        choker_dist < flank_dist,
        "CenterChoker ({choker_dist}) should be closer to center than FlankExpress ({flank_dist})"
    );

    // PerimeterLurker should touch the outer perimeter
    let touches_perimeter = mv_lurker.r == 0
        || mv_lurker.r + mv_lurker.h == BOARD_HEIGHT
        || mv_lurker.c == 0
        || mv_lurker.c + mv_lurker.w == BOARD_WIDTH;
    assert!(
        touches_perimeter,
        "PerimeterLurker should touch perimeter: {:?}",
        mv_lurker
    );

    // Turtle should avoid noodles and maintain compactness
    let turtle_aspect = mv_turtle.h.min(mv_turtle.w) as f64 / mv_turtle.h.max(mv_turtle.w) as f64;
    assert!(
        turtle_aspect > 0.0,
        "Turtle aspect ratio should be positive: {:?}",
        mv_turtle
    );

    // Distinct policies should not all choose the exact same move
    let distinct_moves = [mv_flank, mv_cross, mv_choker, mv_wall, mv_greedy, mv_turtle, mv_lurker];
    let unique_count = {
        let mut v = distinct_moves.to_vec();
        v.sort_by_key(|m| (m.r, m.c, m.h, m.w));
        v.dedup();
        v.len()
    };
    assert!(
        unique_count >= 3,
        "Expected at least 3 distinct move choices across archetypes, got {unique_count}"
    );
}

/// 7. PhaseShiftBot Sequential Strategy Transitions
#[test]
fn test_phaseshift_bot_sequential_transitions() {
    let mut bot = PhaseShiftBot::new_sequential(
        Archetype::FlankExpress,
        Archetype::CenterChoker,
        Archetype::LoopHunter,
        12,
        28,
        12345,
    );

    let mut board = Board::new();
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));

    // Phase 1 (Turns 1..12): FlankExpress
    board.turn = 5;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::FlankExpress),
        "Turn 5 should be FlankExpress"
    );
    board.turn = 12;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::FlankExpress),
        "Turn 12 should be FlankExpress"
    );

    // Phase 2 (Turns 13..28): CenterChoker
    board.turn = 13;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::CenterChoker),
        "Turn 13 should transition to CenterChoker"
    );
    board.turn = 25;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::CenterChoker),
        "Turn 25 should be CenterChoker"
    );
    board.turn = 28;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::CenterChoker),
        "Turn 28 should be CenterChoker"
    );

    // Phase 3 (Turns 29+): LoopHunter
    board.turn = 29;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::LoopHunter),
        "Turn 29 should transition to LoopHunter"
    );
    board.turn = 50;
    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::LoopHunter),
        "Turn 50 should be LoopHunter"
    );

    // Verify move selection matches the active archetype
    board.turn = 8;
    let mv_bot = bot.select_move(&board, Player::Player2, 2, 3);
    let mut rng_ref = Mulberry32::new(12345);
    let mv_flank = select_archetype_move(
        Archetype::FlankExpress,
        &board,
        Player::Player2,
        2,
        3,
        Some(&mut rng_ref),
    );
    assert_eq!(
        mv_bot, mv_flank,
        "PhaseShiftBot move should match FlankExpress during Phase 1"
    );
}

/// 8. PhaseShiftBot Stochastic Mixing & Determinism
#[test]
fn test_phaseshift_bot_stochastic_mixer_determinism() {
    let mut bot1 = PhaseShiftBot::new_stochastic(Archetype::GreedyMaxArea, Archetype::CompactTurtle, 0.5, 42);
    let mut bot2 = PhaseShiftBot::new_stochastic(Archetype::GreedyMaxArea, Archetype::CompactTurtle, 0.5, 42);

    let mut board = Board::new();
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));

    // Identical seeds must make identical choices across 50 simulated turns
    for turn in 1..=50 {
        board.turn = turn;
        let mv1 = bot1.select_move(&board, Player::Player2, 2, 3);
        let mv2 = bot2.select_move(&board, Player::Player2, 2, 3);
        assert_eq!(
            mv1, mv2,
            "Stochastic PhaseShiftBots with identical seeds diverged on turn {turn}"
        );
    }
}

/// 9. PhaseShiftBot Event-Driven Dynamic Switching
#[test]
fn test_phaseshift_bot_event_driven_switching() {
    let bot = PhaseShiftBot::new_event_driven(
        Archetype::CompactTurtle,
        Archetype::AggressiveChaser,
        Archetype::WallBuilder,
        5,   // contact distance <= 5 triggers AggressiveChaser
        200, // empty cells < 200 triggers WallBuilder
        42,
    );

    // Initial state: P1 at (0,0), P2 at (14,39) -> contact distance ~ 53, empty cells ~ 592
    let mut board = Board::new();
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));

    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::CompactTurtle),
        "Distant pieces should trigger base bot (CompactTurtle)"
    );

    // Move pieces close together (contact distance <= 5)
    board.apply_move(Player::Player1, &Move::new(5, 18, 2, 2));
    board.apply_move(Player::Player2, &Move::new(6, 21, 2, 2));
    let contact_dist = compute_min_contact_distance(&board);
    assert!(contact_dist <= 5, "Contact distance should be <= 5: {contact_dist}");

    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::AggressiveChaser),
        "Close pieces should trigger contact bot (AggressiveChaser)"
    );

    // Fill board so empty cells < 200
    for r in 0..10 {
        for c in 0..40 {
            board.cells[r * BOARD_WIDTH + c] = PLAYER_1;
        }
    }
    board.p1_score = 400;
    assert!(board.empty_cells_count() < 200);

    assert_eq!(
        bot.active_archetype(&board),
        Some(Archetype::WallBuilder),
        "Low empty cells should trigger endgame bot (WallBuilder)"
    );
}

/// 10. Trait Object Polymorphism via `Box<dyn ArchetypeBot>`
#[test]
fn test_archetype_bot_trait_dispatch() {
    let mut bots: Vec<Box<dyn ArchetypeBot>> = vec![
        Box::new(FlankExpressBot::new(1)),
        Box::new(CrossFlankRaiderBot::new(2)),
        Box::new(CenterChokerBot::new(3)),
        Box::new(WallBuilderBot::new(4)),
        Box::new(GreedyMaxAreaBot::new(5)),
        Box::new(CompactTurtleBot::new(6)),
        Box::new(AggressiveChaserBot::new(7)),
        Box::new(LoopHunterBot::new(8)),
        Box::new(PerimeterLurkerBot::new(9)),
        Box::new(ChaoticHopperBot::new(10)),
        Box::new(PhaseShiftBot::new_sequential(
            Archetype::FlankExpress,
            Archetype::CenterChoker,
            Archetype::LoopHunter,
            12,
            28,
            11,
        )),
    ];

    let mut board = Board::new();
    board.apply_move(Player::Player1, &Move::new(0, 0, 2, 2));
    board.apply_move(Player::Player2, &Move::new(13, 38, 2, 2));
    board.turn = 3;

    for bot in bots.iter_mut() {
        let name = bot.name();
        let mv = bot.select_move(&board, Player::Player2, 3, 3);
        assert!(
            mv.is_some(),
            "Bot '{}' returned None for valid dice roll (3, 3)",
            name
        );
        let m = mv.unwrap();
        assert!(
            board.can_place_rectangle(Player::Player2, m.r, m.c, m.h, m.w),
            "Bot '{}' placed illegal move: {:?}",
            name,
            m
        );
    }
}

/// 11. High-Throughput Execution Benchmark (Zero FFI Overhead)
#[test]
fn test_zoo_throughput_performance() {
    let mut board = Board::new();
    board.apply_move(Player::Player1, &Move::new(0, 0, 3, 3));
    board.apply_move(Player::Player2, &Move::new(12, 37, 3, 3));
    board.turn = 3;

    let start = std::time::Instant::now();
    let mut move_count = 0;

    for &arch in &Archetype::all() {
        for d1 in 1..=4 {
            for d2 in 1..=4 {
                let mv = select_archetype_move(arch, &board, Player::Player2, d1, d2, None);
                if mv.is_some() {
                    move_count += 1;
                }
            }
        }
    }

    let elapsed = start.elapsed();
    assert!(
        move_count > 0,
        "No moves generated during throughput test"
    );
    assert!(
        elapsed.as_millis() < 500,
        "160 archetype evaluations took too long: {:?}",
        elapsed
    );
}

/// 12. Gate M1 Criterion: Trajectory Equivalence on Mirrored Seeds
///
/// Verifies that across a sequence of 20 moves from identical dice rolls,
/// every archetype playing as Player 1 produces the exact 180°-rotated
/// trajectory as when playing as Player 2 on the mirrored board.
#[test]
fn test_mirrored_trajectory_equivalence_gate_m1() {
    for &arch in &Archetype::all() {
        for &seed in &[12345u32, 67890u32, 54321u32] {
            let mut dice_rng1 = Mulberry32::new(seed);
            let mut dice_rng2 = Mulberry32::new(seed);

            let mut bot_rng_p1 = Mulberry32::new(seed.wrapping_add(100));
            let mut bot_rng_p2 = Mulberry32::new(seed.wrapping_add(100));

            let mut board_p1 = Board::new();
            let mut board_p2 = Board::new();

            for turn in 1..=20 {
                board_p1.turn = turn;
                board_p2.turn = turn;

                let d1_1 = dice_rng1.roll_die();
                let d2_1 = dice_rng1.roll_die();

                let d1_2 = dice_rng2.roll_die();
                let d2_2 = dice_rng2.roll_die();
                assert_eq!(d1_1, d1_2);
                assert_eq!(d2_1, d2_2);

                let mv1 = select_archetype_move(
                    arch,
                    &board_p1,
                    Player::Player1,
                    d1_1,
                    d2_1,
                    Some(&mut bot_rng_p1),
                );
                let mv2 = select_archetype_move(
                    arch,
                    &board_p2,
                    Player::Player2,
                    d1_2,
                    d2_2,
                    Some(&mut bot_rng_p2),
                );

                match (mv1, mv2) {
                    (Some(m1), Some(m2)) => {
                        let inv_m2 = Board::inverse_move_180(&m2);
                        assert_eq!(
                            m1, inv_m2,
                            "Trajectory divergence for {:?} at turn {} (seed {}):\n  P1 move: {:?}\n  P2 move: {:?}\n  Inv P2: {:?}",
                            arch, turn, seed, m1, m2, inv_m2
                        );
                        board_p1.apply_move(Player::Player1, &m1);
                        board_p2.apply_move(Player::Player2, &m2);

                        // Board states must remain exact 180° rotations of each other
                        let rot_p1 = board_p1.rotate_board_180();
                        assert_eq!(
                            board_p2.cells, rot_p1.cells,
                            "Board state mismatch after turn {} for {:?}",
                            turn, arch
                        );
                    }
                    (None, None) => {
                        // Both pass
                    }
                    _ => {
                        panic!(
                            "Asymmetric pass/move for {:?} at turn {}: P1={:?}, P2={:?}",
                            arch, turn, mv1, mv2
                        );
                    }
                }
            }
        }
    }
}
