//! Integration and Benchmark Verification Test Suite for Milestone 3 (Appex Model & Blending).
//!
//! Verifies:
//! 1. Blending formula invariants:
//!    - Turn 0 is 100% Argon-4
//!    - Alpha turn rampup schedule (turn/8 * (1 - H) clamped to 0.65)
//!    - Guaranteed >= 35% Argon-4 anchor preservation
//!    - Maximum entropy H(p) = 1.0 forces alpha = 0.0
//!    - Numeraire WEIGHT_AREA = 50.0 strictly invariant
//!    - Parameter clamping corridor adherence
//! 2. AppexBot online telemetry tracking, move selection, and legality
//! 3. Gatekeeper Benchmark: AppexBot vs Zoo pool & PhaseShiftBot (100 mirrored CRN pairs, >= 75% winrate)
//! 4. Gatekeeper Benchmark: AppexBot vs frozen Argon-4 (100 mirrored CRN pairs, >= 50% winrate)
//! 5. Bit-exact SHA256 integrity verification of Argon-4 benchmark files.

use pyrust::appex::model::{
    evaluate_appex_bot_crn, AppexBot, AppexWeights, ARGON4_BASE_PARAMS,
};
use pyrust::game::{
    generate_default_v2_matrix, Board, Move, Player, BOARD_HEIGHT, BOARD_WIDTH,
};
use std::fs::File;
use std::io::Read;

const EPS: f64 = 1e-6;

#[test]
fn test_blending_turn_0_is_pure_argon4() {
    let weights = AppexWeights::default();
    let probs = [1.0, 0.0, 0.0, 0.0];
    let active = weights.compute_active_weights(0, &probs, 0.0);

    assert_eq!(active.weight_area, 50.0);
    assert!((active.weight_capture - ARGON4_BASE_PARAMS.weight_capture).abs() < EPS);
    assert!((active.weight_deny_capture - ARGON4_BASE_PARAMS.weight_deny_capture).abs() < EPS);
    assert!((active.weight_voronoi_delta - ARGON4_BASE_PARAMS.weight_voronoi_delta).abs() < EPS);
    assert!((active.weight_safe_territory - ARGON4_BASE_PARAMS.weight_safe_territory).abs() < EPS);
    assert!((active.weight_compactness - ARGON4_BASE_PARAMS.weight_compactness).abs() < EPS);
    assert!((active.weight_spatial_matrix - ARGON4_BASE_PARAMS.weight_spatial_matrix).abs() < EPS);
    assert!((active.weight_frontier - ARGON4_BASE_PARAMS.weight_frontier).abs() < EPS);
    assert!((active.weight_opp_choke - ARGON4_BASE_PARAMS.weight_opp_choke).abs() < EPS);
    assert!((active.weight_bottleneck - ARGON4_BASE_PARAMS.weight_bottleneck).abs() < EPS);
}

#[test]
fn test_blending_alpha_rampup_schedule() {
    // Zero entropy: alpha = min(0.65, turn / 8.0)
    assert!((AppexWeights::compute_alpha(0, 0.0) - 0.0).abs() < EPS);
    assert!((AppexWeights::compute_alpha(2, 0.0) - 0.25).abs() < EPS);
    assert!((AppexWeights::compute_alpha(4, 0.0) - 0.50).abs() < EPS);
    assert!((AppexWeights::compute_alpha(6, 0.0) - 0.65).abs() < EPS);
    assert!((AppexWeights::compute_alpha(8, 0.0) - 0.65).abs() < EPS);
    assert!((AppexWeights::compute_alpha(20, 0.0) - 0.65).abs() < EPS);
    assert!((AppexWeights::compute_alpha(80, 0.0) - 0.65).abs() < EPS);
}

#[test]
fn test_blending_guarantees_minimum_35_percent_argon4() {
    let turns = [0, 1, 2, 4, 8, 15, 30, 80];
    let entropies = [0.0, 0.1, 0.5, 0.9, 1.0];

    for &turn in &turns {
        for &ent in &entropies {
            let alpha = AppexWeights::compute_alpha(turn, ent);
            assert!(alpha >= 0.0, "alpha must be non-negative");
            assert!(alpha <= 0.65 + EPS, "alpha must never exceed 0.65");
            let argon4_share = 1.0 - alpha;
            assert!(
                argon4_share >= 0.35 - EPS,
                "Argon-4 fraction must be at least 35%"
            );
        }
    }
}

#[test]
fn test_blending_maximum_entropy_forces_alpha_zero() {
    let weights = AppexWeights::default();
    let uniform_probs = [0.25, 0.25, 0.25, 0.25];
    let entropy_max = 1.0;

    for turn in [1, 4, 8, 20, 80] {
        let alpha = AppexWeights::compute_alpha(turn, entropy_max);
        assert_eq!(alpha, 0.0);
        let active = weights.compute_active_weights(turn, &uniform_probs, entropy_max);
        assert!((active.weight_capture - ARGON4_BASE_PARAMS.weight_capture).abs() < EPS);
        assert!((active.weight_deny_capture - ARGON4_BASE_PARAMS.weight_deny_capture).abs() < EPS);
    }
}

#[test]
fn test_blending_numeraire_area_fixed_at_50() {
    let weights = AppexWeights::default();
    let test_probs = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
        [0.7, 0.1, 0.1, 0.1],
        [0.25, 0.25, 0.25, 0.25],
    ];

    for turn in [0, 1, 4, 8, 16, 40, 80] {
        for &probs in &test_probs {
            for ent in [0.0, 0.2, 0.5, 0.8, 1.0] {
                let active = weights.compute_active_weights(turn, &probs, ent);
                assert_eq!(
                    active.weight_area, 50.0,
                    "WEIGHT_AREA must be invariant at 50.0"
                );
            }
        }
    }
}

#[test]
fn test_blending_clamping_corridors_preserved() {
    let weights = AppexWeights::default();
    let test_probs = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
        [0.4, 0.3, 0.2, 0.1],
    ];

    for turn in [0, 4, 8, 30] {
        for &probs in &test_probs {
            let active = weights.compute_active_weights(turn, &probs, 0.1);
            assert!((70.0..=150.0).contains(&active.weight_capture));
            assert!((60.0..=150.0).contains(&active.weight_deny_capture));
            assert!((15.0..=55.0).contains(&active.weight_voronoi_delta));
            assert!((15.0..=35.0).contains(&active.weight_safe_territory));
            assert!((5.0..=25.0).contains(&active.weight_compactness));
            assert!((0.10..=0.50).contains(&active.weight_spatial_matrix));
            assert!((10.0..=30.0).contains(&active.weight_frontier));
            assert!((5.0..=25.0).contains(&active.weight_opp_choke));
            assert!((5.0..=25.0).contains(&active.weight_bottleneck));
        }
    }
}

#[test]
fn test_appex_bot_move_selection_validity() {
    let mut bot = AppexBot::default_model();
    let board = Board::new();

    // Opening move as P1: must cover corner (0, 0)
    let mv1 = bot.select_move(&board, Player::Player1, 2, 3);
    assert!(mv1.is_some());
    let m1 = mv1.unwrap();
    assert_eq!(m1.r, 0);
    assert_eq!(m1.c, 0);

    // Opening move as P2: must cover corner (14, 39)
    let mv2 = bot.select_move(&board, Player::Player2, 3, 2);
    assert!(mv2.is_some());
    let m2 = mv2.unwrap();
    assert_eq!(m2.r + m2.h, BOARD_HEIGHT);
    assert_eq!(m2.c + m2.w, BOARD_WIDTH);
}

#[test]
fn test_appex_bot_telemetry_observation_and_adaptation() {
    let mut bot = AppexBot::default_model();
    let prev_board = Board::new();
    let mut curr_board = prev_board.clone();

    // Simulate FlankExpress opening move by Player 2 along flank r in [12..14]
    let opp_mv = Move::new(12, 35, 3, 5);
    curr_board.apply_move(Player::Player2, &opp_mv);

    bot.observe_opponent_move(&prev_board, &curr_board, &opp_mv, Player::Player2);

    let probs = bot.profiler.get_probabilities();
    assert_eq!(probs.len(), 4);
    assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-4);

    let active_w = bot.get_active_weights(6);
    assert_eq!(active_w.weight_area, 50.0);
    assert!(active_w.weight_frontier > 12.0);
}

#[test]
fn test_gatekeeper_benchmark_vs_zoo_and_phaseshift() {
    // Series of 100 mirrored CRN pairs (200 matches) against the Zoo pool and PhaseShiftBot
    let appex_weights = AppexWeights::default();
    let matrix = generate_default_v2_matrix();

    // 10 opponents representative of the Zoo catalog and PhaseShiftBot
    let opponents = [
        "FlankExpress",
        "CrossFlankRaider",
        "CenterChoker",
        "WallBuilder",
        "GreedyMaxArea",
        "CompactTurtle",
        "AggressiveChaser",
        "LoopHunter",
        "PerimeterLurker",
        "PhaseShiftBot",
    ];

    // 10 independent test seeds across 10 opponents = 100 CRN seed pairs = 200 matches
    let seeds: Vec<u32> = (3000..3010).collect();

    let stats = evaluate_appex_bot_crn(&appex_weights, &matrix, &opponents, &seeds, 80);

    println!(
        "\n=================================================================\n\
         GATEKEEPER BENCHMARK 1: Appex vs Zoo & PhaseShift\n\
         Matches: {} ({} wins, {} losses, {} draws)\n\
         Winrate: {:.2}% (Target >= 75.0%)\n\
         Average Margin: {:+.2} cells\n\
         Fitness: {:.2}\n\
         =================================================================",
        stats.total_matches, stats.wins, stats.losses, stats.draws,
        stats.winrate, stats.avg_margin, stats.fitness
    );

    assert_eq!(stats.total_matches, 200);
    assert!(
        stats.winrate >= 75.0,
        "Gatekeeper Benchmark failed: Appex winrate vs Zoo is {:.2}%, expected >= 75.0%",
        stats.winrate
    );
}

#[test]
fn test_gatekeeper_benchmark_vs_frozen_argon4() {
    // Sparring against frozen Argon-4 champion (100 mirrored CRN pairs = 200 matches)
    let appex_weights = AppexWeights::default();
    let matrix = generate_default_v2_matrix();

    let opponents = ["Argon4"];
    // 100 independent seeds = 100 CRN pairs = 200 matches (seeds 2000..2100)
    let seeds: Vec<u32> = (2000..2100).collect();

    let stats = evaluate_appex_bot_crn(&appex_weights, &matrix, &opponents, &seeds, 80);

    // Let's inspect a single match's telemetry
    let mut debug_bot = AppexBot::new(appex_weights.clone(), matrix);
    let debug_res = pyrust::appex::model::simulate_appex_match(Player::Player1, &mut debug_bot, "Argon4", 4000, 30);
    println!("DEBUG MATCH 4000: winner={:?}, turns={}, probs={:?}, ent={:.4}, alpha={:.4}",
        debug_res.winner, debug_res.turns, debug_bot.profiler.get_probabilities(),
        debug_bot.profiler.normalized_entropy(),
        AppexWeights::compute_alpha(debug_res.turns, debug_bot.profiler.normalized_entropy())
    );

    println!(
        "\n=================================================================\n\
         GATEKEEPER BENCHMARK 2: Appex vs Frozen Argon-4 Champion\n\
         Matches: {} ({} wins, {} losses, {} draws)\n\
         Winrate: {:.2}% (Target >= 50.0% - Parity or Victory)\n\
         Average Margin: {:+.2} cells\n\
         Fitness: {:.2}\n\
         =================================================================",
        stats.total_matches, stats.wins, stats.losses, stats.draws,
        stats.winrate, stats.avg_margin, stats.fitness
    );

    assert_eq!(stats.total_matches, 200);
    assert!(
        stats.winrate >= 50.0,
        "Gatekeeper Benchmark failed: Appex winrate vs Argon-4 is {:.2}%, expected >= 50.0%",
        stats.winrate
    );
}

#[test]
fn test_argon4_baseline_protection_sha256() {
    let expected_hash = "7dac89a0f17211f383a86435f51a77bc9bdfba55957f53b5b2b08ae8ba2da43b";

    let weights_paths = [
        "weights.json",
        "src/AI/weights.json",
        "src/AI/PyRust/weights.json",
        "/Users/kirillizotov/Developer/territories/territories/src/AI/weights.json",
        "/Users/kirillizotov/Developer/territories/territories/src/AI/PyRust/weights.json",
    ];

    let mut checked = 0;
    for path in &weights_paths {
        if let Ok(mut f) = File::open(path) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                // Compute SHA256 manually or via simple byte comparison
                use std::process::Command;
                let output = Command::new("shasum")
                    .arg("-a")
                    .arg("256")
                    .arg(path)
                    .output();
                if let Ok(out) = output {
                    let s = String::from_utf8_lossy(&out.stdout);
                    let computed = s.split_whitespace().next().unwrap_or("");
                    assert_eq!(
                        computed, expected_hash,
                        "Argon-4 integrity breach on {}",
                        path
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0, "At least one weights.json must be verified");
}
