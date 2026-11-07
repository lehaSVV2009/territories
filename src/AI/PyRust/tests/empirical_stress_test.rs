//! Empirical stress-testing suite for Milestone M1 (`src/game.rs`).
//!
//! Covers:
//! 1. PRNG distribution: 10,000 rolls, strictly [1..6], uniformity check, zero panics.
//! 2. Match simulation robustness: 500 matches across 5 matchups (100 matches each),
//!    termination checks, score validity, zero panics.
//! 3. Loop enclosure capture: complex zig-zag, serpentine, spiral, comb, and leak chambers.

use pyrust::game::{
    generate_default_v2_matrix, simulate_match, Board, BotType, HeuristicWeights, Mulberry32,
    Player, BOARD_WIDTH, PLAYER_1, PLAYER_2, TOTAL_CELLS,
};

#[test]
fn test_empirical_prng_distribution_10k() {
    println!("\n=== EMPIRICAL TEST 1: Mulberry32 PRNG Distribution (10,000 rolls) ===");

    // Test multiple seeds to ensure robustness across seed space
    let seeds = [123456789u32, 42u32, 0u32, 1u32, 99999999u32, 0xFFFFFFFFu32];

    for &seed in &seeds {
        let mut rng = Mulberry32::new(seed);
        let mut counts = [0usize; 7]; // 1-indexed: 1..=6
        let total_rolls = 10_000;

        for _ in 0..total_rolls {
            let roll = rng.roll_die();
            assert!(
                roll >= 1 && roll <= 6,
                "PRNG roll out of bounds [1..6]: {}",
                roll
            );
            counts[roll] += 1;
        }

        // Expected frequency per bin: 10,000 / 6 = 1666.67
        let expected = total_rolls as f64 / 6.0;
        let mut chi_square = 0.0;

        println!("Seed {}:", seed);
        for die in 1..=6 {
            let count = counts[die];
            let pct = (count as f64 / total_rolls as f64) * 100.0;
            let diff = count as f64 - expected;
            chi_square += (diff * diff) / expected;
            println!(
                "  Face {}: {:5} rolls ({:5.2}%) [diff: {:+6.1}]",
                die, count, pct, diff
            );

            // Each face must be approximately 16.67% (tolerance corridor [14.0%, 19.5%])
            assert!(
                pct >= 14.0 && pct <= 19.5,
                "Face {} frequency {:.2}% outside reasonable bounds [14.0%, 19.5%]",
                die,
                pct
            );
        }

        println!("  Chi-square statistic: {:.4} (d.f. = 5)", chi_square);
        // Critical value for Chi-Square df=5 at alpha=0.001 is 20.515
        assert!(
            chi_square < 20.515,
            "Chi-Square test rejected uniformity hypothesis: chi2 = {}",
            chi_square
        );
    }

    println!("PRNG distribution test PASSED: All rolls strictly in [1..6], uniform ~16.6%, zero panics.\n");
}

#[derive(Default)]
struct MatchupStats {
    total: usize,
    p1_wins: usize,
    p2_wins: usize,
    draws: usize,
    total_turns: usize,
    min_turns: usize,
    max_turns: usize,
    ended_by_score: usize,
    ended_by_turn_limit: usize,
    ended_by_passes: usize,
    p1_total_score: usize,
    p2_total_score: usize,
}

#[test]
fn test_empirical_match_simulation_robustness_500() {
    println!("\n=== EMPIRICAL TEST 2: Match Simulation Robustness (500 matches) ===");

    let matchups = [
        ("Greedy vs Greedy", BotType::Greedy, BotType::Greedy),
        ("Greedy vs Fast", BotType::Greedy, BotType::Fast),
        ("Greedy vs Hard", BotType::Greedy, BotType::LegacyHard),
        ("Hard vs Hard", BotType::LegacyHard, BotType::LegacyHard),
        ("SpatialSlayer vs Hard", BotType::SpatialSlayer, BotType::LegacyHard),
    ];

    let weights = HeuristicWeights::default();
    let matrix = generate_default_v2_matrix();

    let mut global_match_count = 0;

    for (name, bot1, bot2) in matchups {
        println!("\n--- Testing Matchup: {} (100 games) ---", name);
        let mut stats = MatchupStats {
            min_turns: usize::MAX,
            ..Default::default()
        };

        for i in 0..100 {
            // Generate diverse seeds using salt and multiplier
            let seed = (global_match_count as u32)
                .wrapping_mul(2654435761)
                .wrapping_add(i as u32 * 31 + 12345);
            global_match_count += 1;

            let res = simulate_match(
                bot1,
                bot2,
                Some(seed),
                80,
                Some(&weights),
                Some(&weights),
                Some(&matrix),
                Some(&matrix),
            );

            // 1. Invariant: Turns strictly in [1..80]
            assert!(
                res.turns >= 1 && res.turns <= 80,
                "Game turns out of bounds: turns={} for seed={}",
                res.turns,
                seed
            );

            // 2. Invariant: Total board cells cannot exceed 600
            let total_score = (res.p1_score as usize) + (res.p2_score as usize);
            assert!(
                total_score <= TOTAL_CELLS,
                "Total score exceeds board size 600: p1={}, p2={}, sum={}",
                res.p1_score,
                res.p2_score,
                total_score
            );

            // 3. Invariant: Both players scored points (unless immediate deadlock on turn 1)
            assert!(
                res.p1_score > 0,
                "P1 should have placed initial move: p1_score={}",
                res.p1_score
            );
            assert!(
                res.p2_score > 0,
                "P2 should have placed initial move: p2_score={}",
                res.p2_score
            );

            // 4. Invariant: Winner validity
            if res.p1_score > 300 {
                assert_eq!(
                    res.winner,
                    Some(Player::Player1),
                    "P1 score > 300 but winner is not Player1: seed={}",
                    seed
                );
            } else if res.p2_score > 300 {
                assert_eq!(
                    res.winner,
                    Some(Player::Player2),
                    "P2 score > 300 but winner is not Player2: seed={}",
                    seed
                );
            } else if res.p1_score > res.p2_score {
                assert_eq!(res.winner, Some(Player::Player1));
            } else if res.p2_score > res.p1_score {
                assert_eq!(res.winner, Some(Player::Player2));
            } else {
                assert_eq!(res.winner, None, "Tie game must have winner=None");
            }

            // 5. Termination categorization
            if res.p1_score > 300 || res.p2_score > 300 {
                stats.ended_by_score += 1;
            } else if res.turns == 80 {
                stats.ended_by_turn_limit += 1;
            } else {
                // Must have ended by 4 consecutive passes
                stats.ended_by_passes += 1;
            }

            // Stats accumulation
            stats.total += 1;
            match res.winner {
                Some(Player::Player1) => stats.p1_wins += 1,
                Some(Player::Player2) => stats.p2_wins += 1,
                None => stats.draws += 1,
            }
            stats.total_turns += res.turns;
            if res.turns < stats.min_turns {
                stats.min_turns = res.turns;
            }
            if res.turns > stats.max_turns {
                stats.max_turns = res.turns;
            }
            stats.p1_total_score += res.p1_score as usize;
            stats.p2_total_score += res.p2_score as usize;
        }

        println!("Results for {}:", name);
        println!(
            "  Winrate: P1: {}% ({}), P2: {}% ({}), Draws: {}% ({})",
            stats.p1_wins, stats.p1_wins, stats.p2_wins, stats.p2_wins, stats.draws, stats.draws
        );
        println!(
            "  Turns: avg={:.1}, min={}, max={}",
            stats.total_turns as f64 / stats.total as f64,
            stats.min_turns,
            stats.max_turns
        );
        println!(
            "  Average Scores: P1={:.1}, P2={:.1}, Sum={:.1}",
            stats.p1_total_score as f64 / stats.total as f64,
            stats.p2_total_score as f64 / stats.total as f64,
            (stats.p1_total_score + stats.p2_total_score) as f64 / stats.total as f64
        );
        println!(
            "  Termination Reasons: Score>300: {}, TurnLimit(80): {}, Passes(4): {}",
            stats.ended_by_score, stats.ended_by_turn_limit, stats.ended_by_passes
        );
    }

    assert_eq!(global_match_count, 500);
    println!("\nMatch simulation robustness test PASSED: 500/500 matches terminated cleanly without panic or loop.\n");
}

#[test]
fn test_empirical_loop_enclosure_complex_zigzag_chambers() {
    println!("\n=== EMPIRICAL TEST 3: Loop Enclosure in Complex Zig-zag Chambers ===");

    // Case 1: Complex Serpentine / Zig-zag Chamber enclosed by P1 along top & left walls
    // We construct a zig-zag snake wall:
    // Row 0: wall
    // Row 1: empty corridor going left-to-right (c: 1..=10), capped at c=10 by P1 wall
    // Row 2: P1 wall separating corridors (c: 1..=9), opening at c=10
    // Row 3: empty corridor going right-to-left (c: 1..=10), capped at c=0 by board boundary
    // Row 4: P1 wall capping the bottom of the serpentine chamber (c: 0..=11)
    // and vertical P1 wall on col 11 (r: 0..=4).
    {
        let mut board = Board::new();
        // Outer bounding walls for the serpentine chamber
        for c in 0..=11 {
            board.set(0, c, PLAYER_1); // top wall
            board.set(4, c, PLAYER_1); // bottom wall
        }
        for r in 0..=4 {
            board.set(r, 0, PLAYER_1);  // left wall
            board.set(r, 11, PLAYER_1); // right wall
        }

        // Interior zig-zag divider at row 2, columns 1..=9 (leaving column 10 open as turnaround)
        for c in 1..=9 {
            board.set(2, c, PLAYER_1);
        }

        // P2 is elsewhere (bottom-right)
        board.set(14, 39, PLAYER_2);
        board.p2_score = 1;

        // Count expected empty cells:
        // Inside rectangle (r: 1..=3, c: 1..=10) = 3 * 10 = 30 cells.
        // Row 2 has 9 wall cells (c: 1..=9).
        // Total empty cells inside serpentine chamber = 30 - 9 = 21 cells.
        // Let's verify each row:
        // Row 1: c 1..=10 -> 10 cells
        // Row 2: c 10 -> 1 cell
        // Row 3: c 1..=10 -> 10 cells
        // Sum = 21 cells.
        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 1 (Serpentine zig-zag): P1 captured {} cells (expected 21), P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert_eq!(p1_caps.len(), 21, "P1 should capture all 21 zig-zag cells");
        assert_eq!(p2_caps.len(), 0, "P2 should capture 0 cells");

        // Verify specific empty cell indices are all included
        for c in 1..=10 {
            assert!(p1_caps.contains(&(1 * BOARD_WIDTH + c)));
            assert!(p1_caps.contains(&(3 * BOARD_WIDTH + c)));
        }
        assert!(p1_caps.contains(&(2 * BOARD_WIDTH + 10)));
    }

    // Case 2: Interleaved Comb / E-shaped Chamber
    // Comb spine along column 0 (r: 0..=8).
    // Three horizontal teeth: row 0 (c: 0..=6), row 4 (c: 0..=6), row 8 (c: 0..=6).
    // Right cap wall along column 7 (r: 0..=8).
    // Empty chambers are slot 1 (rows 1..=3, cols 1..=6 = 18 cells)
    // and slot 2 (rows 5..=7, cols 1..=6 = 18 cells).
    // Total enclosed = 36 cells.
    {
        let mut board = Board::new();
        // Comb spine and right cap
        for r in 0..=8 {
            board.set(r, 0, PLAYER_1);
            board.set(r, 7, PLAYER_1);
        }
        // Horizontal teeth
        for c in 0..=7 {
            board.set(0, c, PLAYER_1);
            board.set(4, c, PLAYER_1);
            board.set(8, c, PLAYER_1);
        }
        board.set(14, 39, PLAYER_2);
        board.p2_score = 1;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 2 (Interleaved Comb): P1 captured {} cells (expected 36), P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert_eq!(p1_caps.len(), 36);
        assert_eq!(p2_caps.len(), 0);
    }

    // Case 3: Inward Spiral Labyrinth (Centrally Located, No Boundary Walls)
    // Bounded completely by P1 floating in the middle of the board (rows 3..=11, cols 10..=22)
    {
        let mut board = Board::new();
        board.set(0, 0, PLAYER_1); // P1 corner
        board.set(14, 39, PLAYER_2); // P2 corner
        board.p1_score = 1;
        board.p2_score = 1;

        // Outer ring: rows 3 and 11, cols 10..=22
        for c in 10..=22 {
            board.set(3, c, PLAYER_1);
            board.set(11, c, PLAYER_1);
        }
        for r in 3..=11 {
            board.set(r, 10, PLAYER_1);
            board.set(r, 22, PLAYER_1);
        }
        // Inner spiral arm 1: enter at row 5, col 12..=20
        for c in 12..=20 {
            board.set(5, c, PLAYER_1);
        }
        // Inner spiral arm 2: down at col 20, row 5..=9
        for r in 5..=9 {
            board.set(r, 20, PLAYER_1);
        }
        // Inner spiral arm 3: left at row 9, col 14..=20
        for c in 14..=20 {
            board.set(9, c, PLAYER_1);
        }
        // Inner spiral arm 4: up at col 14, row 7..=9
        for r in 7..=9 {
            board.set(r, 14, PLAYER_1);
        }
        // Inner spiral arm 5: right at row 7, col 14..=18
        for c in 14..=18 {
            board.set(7, c, PLAYER_1);
        }

        // All internal empty cells inside rows 4..=10, cols 11..=21 are strictly bounded by P1
        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 3 (Inward Spiral): P1 captured {} cells, P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert!(p1_caps.len() > 0, "Spiral interior must be captured by P1");
        assert_eq!(p2_caps.len(), 0, "P2 should capture 0");

        // Verify that every captured cell is within rows 4..=10, cols 11..=21
        for &idx in &p1_caps {
            let r = idx / BOARD_WIDTH;
            let c = idx % BOARD_WIDTH;
            assert!(r >= 4 && r <= 10 && c >= 11 && c <= 21);
        }
    }

    // Case 4a: Contested Zig-zag Chamber with P2 outer presence
    // Same as Case 1, but P2 also touches inside the chamber at (1, 10),
    // and P2 has an outer presence at (14, 39).
    // Result: Chamber touches BOTH P1 and P2 -> 0 captured inside!
    // Outer board touches BOTH P1 and P2 -> 0 captured outside! Total = 0.
    {
        let mut board = Board::new();
        for c in 0..=11 {
            board.set(0, c, PLAYER_1);
            board.set(4, c, PLAYER_1);
        }
        for r in 0..=4 {
            board.set(r, 0, PLAYER_1);
            board.set(r, 11, PLAYER_1);
        }
        for c in 1..=9 {
            board.set(2, c, PLAYER_1);
        }
        // P2 touches inside chamber
        board.set(1, 10, PLAYER_2);
        // P2 also present outside at bottom-right corner
        board.set(14, 39, PLAYER_2);
        board.p1_score = 30;
        board.p2_score = 2;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 4a (Contested zig-zag + outer P2): P1 captured {}, P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert_eq!(p1_caps.len(), 0, "Contested chamber must NOT be captured by P1");
        assert_eq!(p2_caps.len(), 0, "Contested chamber must NOT be captured by P2");
    }

    // Case 4b: Trapped Enemy / Inverted Capture
    // P2 exists ONLY inside P1's chamber at (1, 10).
    // The chamber itself is contested (touches P1 and P2) -> 0 captured inside.
    // BUT the outer 540 cells touch ONLY P1 and outer walls -> P1 captures all 540!
    {
        let mut board = Board::new();
        for c in 0..=11 {
            board.set(0, c, PLAYER_1);
            board.set(4, c, PLAYER_1);
        }
        for r in 0..=4 {
            board.set(r, 0, PLAYER_1);
            board.set(r, 11, PLAYER_1);
        }
        for c in 1..=9 {
            board.set(2, c, PLAYER_1);
        }
        // P2 exists ONLY inside chamber
        board.set(1, 10, PLAYER_2);
        board.p1_score = 30;
        board.p2_score = 1;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 4b (Trapped Enemy): P1 captured {} cells (expected 540 outer), P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert_eq!(p1_caps.len(), 540, "P1 must capture outer 540 cells");
        assert_eq!(p2_caps.len(), 0);
        // Verify no chamber cells (r <= 4, c <= 11) were captured by P1
        for &idx in &p1_caps {
            let r = idx / BOARD_WIDTH;
            let c = idx % BOARD_WIDTH;
            assert!(
                !(r <= 4 && c <= 11),
                "Contested chamber cell ({}, {}) should not be captured",
                r,
                c
            );
        }
    }

    // Case 5: Diagonal Leak (Orthogonal flood-fill escapes through diagonal gap)
    // A perimeter wall has a missing corner where two wall segments meet only diagonally:
    // e.g., Wall from (0, 5) to (3, 5), and Wall from (4, 6) to (4, 10).
    // At (3, 5) and (4, 6), they touch diagonally, leaving cell (3, 6) and (4, 5) open.
    // Flood fill from inside the chamber escapes into the outer board where P2 is!
    {
        let mut board = Board::new();
        // Top wall
        for c in 0..=10 {
            board.set(0, c, PLAYER_1);
        }
        // Left wall
        for r in 0..=4 {
            board.set(r, 0, PLAYER_1);
        }
        // Right wall
        for r in 0..=3 {
            board.set(r, 10, PLAYER_1);
        }
        // Bottom wall: intentional diagonal gap at (4, 9)
        // Segment 1: (4, 0..=8)
        for c in 0..=8 {
            board.set(4, c, PLAYER_1);
        }
        // Missing (4, 9)! Segment 2: (4, 10) is set.
        // At (3, 10) and (4, 8), the gap is at (4, 9).
        // Since (4, 9) is empty, interior connects to row 5+ where P2 is located.
        board.set(14, 39, PLAYER_2);
        board.p1_score = 25;
        board.p2_score = 1;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 5 (Diagonal Leak): P1 captured {}, P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        // The empty cells inside are connected to row 5..14 which touches P2 at (14, 39)
        // So neither P1 nor P2 should capture them!
        assert_eq!(
            p1_caps.len(),
            0,
            "Chamber with diagonal leak must NOT be captured by P1"
        );
        assert_eq!(p2_caps.len(), 0);
    }

    // Case 6: Dual Simultaneous Complex Enclosures
    // P1 captures a 12-cell zig-zag chamber in top-left.
    // P2 captures an 8-cell zig-zag chamber in bottom-right.
    {
        let mut board = Board::new();
        // P1 top-left chamber (rows 0..=3, cols 0..=5)
        for c in 0..=5 {
            board.set(0, c, PLAYER_1);
            board.set(3, c, PLAYER_1);
        }
        for r in 0..=3 {
            board.set(r, 0, PLAYER_1);
            board.set(r, 5, PLAYER_1);
        }
        // P1 inner divider
        board.set(1, 2, PLAYER_1);
        board.set(2, 2, PLAYER_1);
        // Interior was 2 * 4 = 8, minus 2 divider cells = 6 empty cells.

        // P2 bottom-right chamber (rows 11..=14, cols 34..=39)
        for c in 34..=39 {
            board.set(11, c, PLAYER_2);
            board.set(14, c, PLAYER_2);
        }
        for r in 11..=14 {
            board.set(r, 34, PLAYER_2);
            board.set(r, 39, PLAYER_2);
        }
        board.set(12, 36, PLAYER_2);
        board.set(13, 36, PLAYER_2);

        board.p1_score = 20;
        board.p2_score = 20;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 6 (Dual Simultaneous): P1 captured {} cells (expected 6), P2 captured {} cells (expected 6)",
            p1_caps.len(),
            p2_caps.len()
        );
        assert_eq!(p1_caps.len(), 6);
        assert_eq!(p2_caps.len(), 6);

        // Apply closed loops and verify board score update
        let (c1, c2) = board.apply_closed_loops();
        assert_eq!(c1, 6);
        assert_eq!(c2, 6);
        assert_eq!(board.p1_score, 26);
        assert_eq!(board.p2_score, 26);
    }

    // Case 7: Jagged / Stepped Boundary Zig-Zag
    // Wall steps like stairs: (0, 0), (1, 1), (2, 2), etc.
    // Orthogonal wall fills stairs to form an enclosed triangle.
    {
        let mut board = Board::new();
        // Stairs along diagonal with orthogonal steps:
        // (0,0), (0,1), (1,1), (1,2), (2,2), (2,3), (3,3), (3,4), (4,4), (4,0)
        // Enclosing rows 0..=4, cols 0..=4
        for r in 0..=4 {
            board.set(r, 0, PLAYER_1); // left wall
        }
        for c in 0..=4 {
            board.set(4, c, PLAYER_1); // bottom wall
        }
        // Stepped hypotenuse:
        board.set(0, 1, PLAYER_1);
        board.set(1, 1, PLAYER_1);
        board.set(1, 2, PLAYER_1);
        board.set(2, 2, PLAYER_1);
        board.set(2, 3, PLAYER_1);
        board.set(3, 3, PLAYER_1);
        board.set(3, 4, PLAYER_1);

        board.set(14, 39, PLAYER_2);
        board.p1_score = 15;
        board.p2_score = 1;

        let (p1_caps, p2_caps) = board.find_closed_loops();
        println!(
            "Case 7 (Stepped triangle): P1 captured {} cells, P2 captured {}",
            p1_caps.len(),
            p2_caps.len()
        );
        assert!(p1_caps.len() > 0);
        assert_eq!(p2_caps.len(), 0);

        // Verify all captured cells are within the enclosed triangle
        for &idx in &p1_caps {
            let r = idx / BOARD_WIDTH;
            let c = idx % BOARD_WIDTH;
            assert!(r <= 4 && c <= 4);
        }
    }

    println!("\nLoop enclosure in complex zig-zag chambers test PASSED: All cases verified.\n");
}
