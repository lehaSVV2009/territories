//! Adaptive weight blending, opponent profiling integration, and move inference for Appex.
//!
//! Milestone 3: Appex Model, Blending & Benchmark Verification.
//!
//! Implements:
//! - Convex combination around the immutable Argon-4 base anchor:
//!   W_active = (1 - alpha) * W_Argon4 + alpha * sum_{k=1..4} (p_k * W_{counter, k})
//! - Dynamic blending factor: alpha = min(0.65, (turn / 8) * (1 - H(p))), guaranteeing >= 35% Argon-4 preservation in any phase
//! - Normalized Shannon entropy modulation H(p) in [0.0, 1.0] with numerical safeguard (p_k <= 1e-12 => 0)
//! - Numeraire protection: WEIGHT_AREA strictly locked at 50.0
//! - Clamping corridor preservation for all heuristic parameters
//! - Autonomous `AppexBot` with online telemetry tracking and SpatialSlayer move evaluation
//! - Headless match simulation and mirrored CRN evaluation engine

use serde::{Deserialize, Serialize};
use crate::appex::profiler::EnemyProfiler;
use crate::appex::zoo::{
    AggressiveChaserBot, Archetype, ArchetypeBot, CenterChokerBot, ChaoticHopperBot,
    CompactTurtleBot, CrossFlankRaiderBot, FlankExpressBot, GreedyMaxAreaBot, LoopHunterBot,
    PerimeterLurkerBot, PhaseShiftBot, WallBuilderBot,
};
use crate::game::{
    generate_default_v2_matrix, select_move_spatial_slayer, Board, HeuristicWeights, MatchResult,
    Move, Mulberry32, Player, BOARD_HEIGHT, BOARD_WIDTH, TOTAL_CELLS,
};
use crate::selector::HeuristicParams;

/// Frozen Argon-4 baseline champion parameters.
pub const ARGON4_BASE_PARAMS: HeuristicParams = HeuristicParams {
    weight_area: 50.0,
    weight_capture: 70.0,
    weight_deny_capture: 101.99484311775426,
    weight_voronoi_delta: 45.0,
    weight_safe_territory: 15.0,
    weight_compactness: 17.254612428998307,
    weight_spatial_matrix: 0.1,
    weight_frontier: 12.916827590509,
    weight_opp_choke: 6.9879891811449815,
    weight_bottleneck: 12.18890761042757,
};

/// 4 Strategic counter-profile parameters (CMA-ES optimized on Zoo).
/// 0: Anti-Rush (counters FlankExpress, CrossFlankRaider, PerimeterLurker)
pub const COUNTER_PROFILE_0_ANTI_RUSH: HeuristicParams = HeuristicParams {
    weight_area: 50.0,
    weight_capture: 70.0,
    weight_deny_capture: 102.0,
    weight_voronoi_delta: 45.0,
    weight_safe_territory: 16.0,
    weight_compactness: 17.25,
    weight_spatial_matrix: 0.10,
    weight_frontier: 22.0,
    weight_opp_choke: 12.0,
    weight_bottleneck: 20.0,
};

/// 1: Anti-Wall/Loop (counters WallBuilder, CenterChoker, LoopHunter)
pub const COUNTER_PROFILE_1_ANTI_WALL_LOOP: HeuristicParams = HeuristicParams {
    weight_area: 50.0,
    weight_capture: 75.0,
    weight_deny_capture: 125.0,
    weight_voronoi_delta: 45.0,
    weight_safe_territory: 16.0,
    weight_compactness: 17.25,
    weight_spatial_matrix: 0.10,
    weight_frontier: 13.0,
    weight_opp_choke: 8.0,
    weight_bottleneck: 14.0,
};

/// 2: Anti-Turtle/Greedy (counters CompactTurtle, GreedyMaxArea)
pub const COUNTER_PROFILE_2_ANTI_TURTLE_GREEDY: HeuristicParams = HeuristicParams {
    weight_area: 50.0,
    weight_capture: 70.0,
    weight_deny_capture: 102.0,
    weight_voronoi_delta: 45.0,
    weight_safe_territory: 22.0,
    weight_compactness: 18.0,
    weight_spatial_matrix: 0.12,
    weight_frontier: 16.0,
    weight_opp_choke: 10.0,
    weight_bottleneck: 12.0,
};

/// 3: Anti-Brawler (counters AggressiveChaser, ChaoticHopper)
pub const COUNTER_PROFILE_3_ANTI_BRAWLER: HeuristicParams = HeuristicParams {
    weight_area: 50.0,
    weight_capture: 72.0,
    weight_deny_capture: 102.0,
    weight_voronoi_delta: 45.0,
    weight_safe_territory: 25.0,
    weight_compactness: 20.0,
    weight_spatial_matrix: 0.15,
    weight_frontier: 12.0,
    weight_opp_choke: 15.0,
    weight_bottleneck: 15.0,
};

/// Blended heuristic weights wrapper containing Argon-4 anchor and 4 counter profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppexWeights {
    pub argon4_weights: HeuristicWeights,
    pub counter_weights: [HeuristicWeights; 4],
}

impl Default for AppexWeights {
    fn default() -> Self {
        Self {
            argon4_weights: HeuristicWeights::from(&ARGON4_BASE_PARAMS),
            counter_weights: [
                HeuristicWeights::from(&COUNTER_PROFILE_0_ANTI_RUSH),
                HeuristicWeights::from(&COUNTER_PROFILE_1_ANTI_WALL_LOOP),
                HeuristicWeights::from(&COUNTER_PROFILE_2_ANTI_TURTLE_GREEDY),
                HeuristicWeights::from(&COUNTER_PROFILE_3_ANTI_BRAWLER),
            ],
        }
    }
}

impl AppexWeights {
    /// Constructs AppexWeights from custom parameter structs.
    pub fn new(argon4: HeuristicParams, counters: [HeuristicParams; 4]) -> Self {
        Self {
            argon4_weights: HeuristicWeights::from(&argon4),
            counter_weights: [
                HeuristicWeights::from(&counters[0]),
                HeuristicWeights::from(&counters[1]),
                HeuristicWeights::from(&counters[2]),
                HeuristicWeights::from(&counters[3]),
            ],
        }
    }

    /// Computes dynamic adaptation coefficient alpha = min(0.65, (turn / 8) * (1 - H(p))).
    /// Guarantees alpha in [0.0, 0.65] and Argon-4 fraction (1 - alpha) >= 0.35.
    #[inline(always)]
    pub fn compute_alpha(turn: usize, entropy: f64) -> f64 {
        let ent = entropy.clamp(0.0, 1.0);
        let raw_alpha = (turn as f64 / 8.0) * (1.0 - ent);
        raw_alpha.clamp(0.0, 0.65)
    }

    /// Computes the active blended heuristic weights for the current turn.
    /// W_active = (1 - alpha) * W_Argon4 + alpha * sum_{k=0..3} (p_k * W_{counter, k})
    /// Invariant: WEIGHT_AREA is strictly fixed at 50.0.
    pub fn compute_active_weights(
        &self,
        turn: usize,
        probs: &[f64; 4],
        entropy: f64,
    ) -> HeuristicWeights {
        let alpha = Self::compute_alpha(turn, entropy);

        let blend = |base: f64, idx_fn: fn(&HeuristicWeights) -> f64| -> f64 {
            let mut counter_sum = 0.0;
            for (k, prob) in probs.iter().enumerate().take(4) {
                counter_sum += prob * idx_fn(&self.counter_weights[k]);
            }
            (1.0 - alpha) * base + alpha * counter_sum
        };

        let mut active = self.argon4_weights.clone();
        active.weight_area = 50.0; // Invariant Numeraire
        active.weight_capture = blend(self.argon4_weights.weight_capture, |w| w.weight_capture);
        active.weight_deny_capture =
            blend(self.argon4_weights.weight_deny_capture, |w| w.weight_deny_capture);
        active.weight_voronoi_delta =
            blend(self.argon4_weights.weight_voronoi_delta, |w| w.weight_voronoi_delta);
        active.weight_safe_territory =
            blend(self.argon4_weights.weight_safe_territory, |w| w.weight_safe_territory);
        active.weight_compactness =
            blend(self.argon4_weights.weight_compactness, |w| w.weight_compactness);
        active.weight_spatial_matrix =
            blend(self.argon4_weights.weight_spatial_matrix, |w| w.weight_spatial_matrix);
        active.weight_frontier =
            blend(self.argon4_weights.weight_frontier, |w| w.weight_frontier);
        active.weight_opp_choke =
            blend(self.argon4_weights.weight_opp_choke, |w| w.weight_opp_choke);
        active.weight_bottleneck =
            blend(self.argon4_weights.weight_bottleneck, |w| w.weight_bottleneck);

        active
    }
}

/// Autonomous Appex bot combining profiling and adaptive weight blending.
#[derive(Debug, Clone)]
pub struct AppexBot {
    pub profiler: EnemyProfiler,
    pub weights: AppexWeights,
    pub spatial_matrix: [f64; TOTAL_CELLS],
    pub prev_board: Option<Board>,
}

impl AppexBot {
    /// Constructs a new AppexBot with custom weights and spatial matrix.
    pub fn new(weights: AppexWeights, spatial_matrix: [f64; TOTAL_CELLS]) -> Self {
        Self {
            profiler: EnemyProfiler::new(),
            weights,
            spatial_matrix,
            prev_board: None,
        }
    }

    /// Constructs the default AppexBot using calibrated weights and default matrix.
    pub fn default_model() -> Self {
        Self::new(AppexWeights::default(), generate_default_v2_matrix())
    }

    /// Computes the active weights vector for the given turn.
    pub fn get_active_weights(&self, turn: usize) -> HeuristicWeights {
        let probs = self.profiler.get_probabilities();
        let entropy = self.profiler.normalized_entropy();
        self.weights.compute_active_weights(turn, &probs, entropy)
    }

    /// Manually records and profiles an opponent move.
    pub fn observe_opponent_move(
        &mut self,
        prev_board: &Board,
        current_board: &Board,
        enemy_move: &Move,
        player: Player,
    ) {
        self.profiler.update(prev_board, current_board, enemy_move, player);
        self.prev_board = Some(current_board.clone());
    }

    /// Selects optimal move with 180° rotation invariance and dynamic active weight blending.
    pub fn select_move(
        &mut self,
        board: &Board,
        player: Player,
        d1: usize,
        d2: usize,
    ) -> Option<Move> {
        // Automatic opponent move detection if previous board was recorded
        if let Some(ref prev) = self.prev_board {
            let opp = player.opponent();
            if prev.turn != board.turn
                && (board.score(opp) > prev.score(opp) || board.cells != prev.cells)
            {
                let opp_val = opp.cell_value();
                let mut min_r = BOARD_HEIGHT;
                let mut max_r = 0;
                let mut min_c = BOARD_WIDTH;
                let mut max_c = 0;
                let mut found = false;
                for r in 0..BOARD_HEIGHT {
                    let row_off = r * BOARD_WIDTH;
                    for c in 0..BOARD_WIDTH {
                        if board.cells[row_off + c] == opp_val && prev.cells[row_off + c] != opp_val {
                            min_r = min_r.min(r);
                            max_r = max_r.max(r);
                            min_c = min_c.min(c);
                            max_c = max_c.max(c);
                            found = true;
                        }
                    }
                }
                if found && max_r >= min_r && max_c >= min_c {
                    let h = max_r - min_r + 1;
                    let w = max_c - min_c + 1;
                    let opp_mv = Move::new(min_r, min_c, h, w);
                    self.profiler.update(prev, board, &opp_mv, opp);
                }
            }
        }

        let active_weights = self.get_active_weights(board.turn);
        let chosen_move = select_move_spatial_slayer(
            board,
            player,
            d1,
            d2,
            &active_weights,
            &self.spatial_matrix,
        );

        self.prev_board = Some(board.clone());
        chosen_move
    }
}

impl ArchetypeBot for AppexBot {
    fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
        self.select_move(board, player, d1, d2)
    }

    fn name(&self) -> &'static str {
        "AppexBot"
    }
}

/// Frozen Argon-4 champion bot for benchmark sparring.
pub struct Argon4OpponentBot {
    pub weights: HeuristicWeights,
    pub spatial_matrix: [f64; TOTAL_CELLS],
    pub rng: Mulberry32,
}

impl Argon4OpponentBot {
    pub fn new(seed: u32) -> Self {
        Self {
            weights: HeuristicWeights::from(&ARGON4_BASE_PARAMS),
            spatial_matrix: generate_default_v2_matrix(),
            rng: Mulberry32::new(seed),
        }
    }
}

impl ArchetypeBot for Argon4OpponentBot {
    fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
        select_move_spatial_slayer(board, player, d1, d2, &self.weights, &self.spatial_matrix)
    }

    fn name(&self) -> &'static str {
        "Argon4"
    }
}

/// Factory to construct opponent bots by archetype name.
pub fn make_opponent_bot(opponent_name: &str, seed: u32) -> Box<dyn ArchetypeBot> {
    let lower = opponent_name.to_lowercase().replace(['_', '-'], "");
    match lower.as_str() {
        "flankexpress" => Box::new(FlankExpressBot::new(seed)),
        "crossflankraider" => Box::new(CrossFlankRaiderBot::new(seed)),
        "centerchoker" => Box::new(CenterChokerBot::new(seed)),
        "wallbuilder" => Box::new(WallBuilderBot::new(seed)),
        "greedymaxarea" | "greedy" | "greedybot" => Box::new(GreedyMaxAreaBot::new(seed)),
        "compactturtle" => Box::new(CompactTurtleBot::new(seed)),
        "aggressivechaser" => Box::new(AggressiveChaserBot::new(seed)),
        "loophunter" => Box::new(LoopHunterBot::new(seed)),
        "perimeterlurker" => Box::new(PerimeterLurkerBot::new(seed)),
        "chaotichopper" => Box::new(ChaoticHopperBot::new(seed)),
        "phaseshift" | "phaseshiftbot" => Box::new(PhaseShiftBot::new_sequential(
            Archetype::FlankExpress,
            Archetype::CenterChoker,
            Archetype::LoopHunter,
            8,
            18,
            seed,
        )),
        "g40" | "G40" | "argon4" | "argon" | "champ" => Box::new(Argon4OpponentBot::new(seed)),
        _ => Box::new(GreedyMaxAreaBot::new(seed)),
    }
}

pub type G40OpponentBot = Argon4OpponentBot;
pub const G40_BASE_PARAMS: HeuristicParams = ARGON4_BASE_PARAMS;

/// Simulates a complete headless match between AppexBot and any opponent.
pub fn simulate_appex_match(
    appex_player: Player,
    appex: &mut AppexBot,
    opponent_name: &str,
    seed: u32,
    max_turns: usize,
) -> MatchResult {
    let mut rng = Mulberry32::new(seed);
    let mut board = Board::new();
    let mut passes = 0;
    let mut turn = 1;

    let mut opp_bot = make_opponent_bot(opponent_name, seed ^ 0x9e3779b9);

    while passes < 4 && turn <= max_turns {
        board.turn = turn;
        let current_player = if turn % 2 == 1 {
            Player::Player1
        } else {
            Player::Player2
        };

        let mut chosen_move = None;
        for _ in 0..3 {
            let d1 = rng.roll_die();
            let d2 = rng.roll_die();
            let mv = if current_player == appex_player {
                appex.select_move(&board, current_player, d1, d2)
            } else {
                opp_bot.select_move(&board, current_player, d1, d2)
            };
            if mv.is_some() {
                chosen_move = mv;
                break;
            }
        }

        if let Some(mv) = chosen_move {
            passes = 0;
            let prev_board = board.clone();
            board.apply_move(current_player, &mv);

            if board.p1_score > 0 && board.p2_score > 0 {
                board.apply_closed_loops();
            }

            if current_player != appex_player {
                appex.observe_opponent_move(&prev_board, &board, &mv, current_player);
            }
        } else {
            passes += 1;
        }

        if board.p1_score > 300 || board.p2_score > 300 {
            break;
        }

        turn += 1;
    }

    let winner = if board.p1_score > board.p2_score {
        Some(Player::Player1)
    } else if board.p2_score > board.p1_score {
        Some(Player::Player2)
    } else {
        None
    };

    MatchResult {
        winner,
        p1_score: board.p1_score,
        p2_score: board.p2_score,
        turns: turn.min(max_turns),
    }
}

/// Statistical output of a batch CRN evaluation series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppexEvaluationStats {
    pub winrate: f64,
    pub avg_margin: f64,
    pub avg_turns: f64,
    pub fitness: f64,
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub total_matches: usize,
    pub net_margin: f64,
}

/// Evaluates a candidate heuristic weight vector directly against a target archetype across mirrored CRN seeds.
pub fn evaluate_counter_candidate_crn(
    candidate_weights: &HeuristicWeights,
    spatial_matrix: &[f64; TOTAL_CELLS],
    target_opponent: &str,
    seeds: &[u32],
    max_turns: usize,
) -> AppexEvaluationStats {
    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;
    let mut total_margin = 0.0;
    let mut total_turns = 0;
    let total_matches = seeds.len() * 2;

    for &seed in seeds {
        // Match 1: Candidate as Player 1 vs Target as Player 2
        let res1 = simulate_single_match_custom(
            Player::Player1,
            candidate_weights,
            spatial_matrix,
            target_opponent,
            seed,
            max_turns,
        );
        let margin1 = (res1.p1_score as f64) - (res1.p2_score as f64);
        total_margin += margin1;
        total_turns += res1.turns;
        if res1.p1_score > res1.p2_score {
            wins += 1;
        } else if res1.p2_score > res1.p1_score {
            losses += 1;
        } else {
            draws += 1;
        }

        // Match 2: Target as Player 1 vs Candidate as Player 2
        let res2 = simulate_single_match_custom(
            Player::Player2,
            candidate_weights,
            spatial_matrix,
            target_opponent,
            seed,
            max_turns,
        );
        let margin2 = (res2.p2_score as f64) - (res2.p1_score as f64);
        total_margin += margin2;
        total_turns += res2.turns;
        if res2.p2_score > res2.p1_score {
            wins += 1;
        } else if res2.p1_score > res2.p2_score {
            losses += 1;
        } else {
            draws += 1;
        }
    }

    let winrate = if total_matches > 0 {
        (wins as f64 / total_matches as f64) * 100.0
    } else {
        0.0
    };
    let avg_margin = if total_matches > 0 {
        total_margin / total_matches as f64
    } else {
        0.0
    };
    let avg_turns = if total_matches > 0 {
        total_turns as f64 / total_matches as f64
    } else {
        0.0
    };
    let fitness = winrate + 0.02 * avg_margin;

    AppexEvaluationStats {
        winrate,
        avg_margin,
        avg_turns,
        fitness,
        wins,
        losses,
        draws,
        total_matches,
        net_margin: total_margin,
    }
}

/// Helper to simulate a match where one side uses custom weights and the other is an Archetype.
fn simulate_single_match_custom(
    cand_player: Player,
    cand_weights: &HeuristicWeights,
    cand_matrix: &[f64; TOTAL_CELLS],
    target_opponent: &str,
    seed: u32,
    max_turns: usize,
) -> MatchResult {
    let mut rng = Mulberry32::new(seed);
    let mut board = Board::new();
    let mut passes = 0;
    let mut turn = 1;

    let mut opp_bot = make_opponent_bot(target_opponent, seed ^ 0x9e3779b9);

    while passes < 4 && turn <= max_turns {
        board.turn = turn;
        let current_player = if turn % 2 == 1 {
            Player::Player1
        } else {
            Player::Player2
        };

        let mut chosen_move = None;
        for _ in 0..3 {
            let d1 = rng.roll_die();
            let d2 = rng.roll_die();
            let mv = if current_player == cand_player {
                select_move_spatial_slayer(&board, current_player, d1, d2, cand_weights, cand_matrix)
            } else {
                opp_bot.select_move(&board, current_player, d1, d2)
            };
            if mv.is_some() {
                chosen_move = mv;
                break;
            }
        }

        if let Some(mv) = chosen_move {
            passes = 0;
            board.apply_move(current_player, &mv);
            if board.p1_score > 0 && board.p2_score > 0 {
                board.apply_closed_loops();
            }
        } else {
            passes += 1;
        }

        if board.p1_score > 300 || board.p2_score > 300 {
            break;
        }

        turn += 1;
    }

    let winner = if board.p1_score > board.p2_score {
        Some(Player::Player1)
    } else if board.p2_score > board.p1_score {
        Some(Player::Player2)
    } else {
        None
    };

    MatchResult {
        winner,
        p1_score: board.p1_score,
        p2_score: board.p2_score,
        turns: turn.min(max_turns),
    }
}

/// Evaluates full AppexBot (online telemetry + adaptive soft blending) against an opponent pool across mirrored CRN pairs.
pub fn evaluate_appex_bot_crn(
    appex_weights: &AppexWeights,
    spatial_matrix: &[f64; TOTAL_CELLS],
    opponents: &[&str],
    seeds: &[u32],
    max_turns: usize,
) -> AppexEvaluationStats {
    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;
    let mut total_margin = 0.0;
    let mut total_turns = 0;
    let total_matches = opponents.len() * seeds.len() * 2;

    for &opp in opponents {
        for &seed in seeds {
            // Match A: Appex as P1
            let mut bot_a = AppexBot::new(appex_weights.clone(), *spatial_matrix);
            let res_a = simulate_appex_match(Player::Player1, &mut bot_a, opp, seed, max_turns);
            let margin_a = (res_a.p1_score as f64) - (res_a.p2_score as f64);
            total_margin += margin_a;
            total_turns += res_a.turns;
            if res_a.p1_score > res_a.p2_score {
                wins += 1;
            } else if res_a.p2_score > res_a.p1_score {
                losses += 1;
            } else {
                draws += 1;
            }

            // Match B: Appex as P2
            let mut bot_b = AppexBot::new(appex_weights.clone(), *spatial_matrix);
            let res_b = simulate_appex_match(Player::Player2, &mut bot_b, opp, seed, max_turns);
            let margin_b = (res_b.p2_score as f64) - (res_b.p1_score as f64);
            total_margin += margin_b;
            total_turns += res_b.turns;
            if res_b.p2_score > res_b.p1_score {
                wins += 1;
            } else if res_b.p1_score > res_b.p2_score {
                losses += 1;
            } else {
                draws += 1;
            }
        }
    }

    let winrate = if total_matches > 0 {
        (wins as f64 / total_matches as f64) * 100.0
    } else {
        0.0
    };
    let avg_margin = if total_matches > 0 {
        total_margin / total_matches as f64
    } else {
        0.0
    };
    let avg_turns = if total_matches > 0 {
        total_turns as f64 / total_matches as f64
    } else {
        0.0
    };
    let fitness = winrate + 0.02 * avg_margin;

    AppexEvaluationStats {
        winrate,
        avg_margin,
        avg_turns,
        fitness,
        wins,
        losses,
        draws,
        total_matches,
        net_margin: total_margin,
    }
}
