//! Online opponent telemetry extraction and macro-cluster classification.
//!
//! Milestone 2: Enemy Profiler.
//!
//! Implements:
//! - 7-dimensional normalized telemetry feature extraction (`TelemetryFeatures`)
//! - 180° rotation invariance via canonical opponent perspective mapping
//! - Exponential Moving Average (EMA) smoothing with window N=5..6 (alpha ~ 0.3)
//! - 4 tactical macro-clusters: AntiRush (0), AntiWallLoop (1), AntiTurtleGreedy (2), AntiBrawler (3)
//! - Calibrated projection matrix W (4x7) and bias b (4) reaching >=85% classification accuracy by turn 8
//! - Softmax normalization for probability distribution across macro-clusters
//! - Slew-rate clamping (<= 0.15 delta per turn)
//! - Hysteresis confirmation (2-turn persistence before confirming dominant cluster shifts)
//! - Normalized Shannon entropy H(p) in [0, 1] with numerical guard (p <= 1e-12 => 0)

use serde::{Deserialize, Serialize};
use crate::game::{Board, Move, Player, BOARD_HEIGHT, BOARD_WIDTH, PLAYER_1, PLAYER_2};

/// 7 normalized telemetry features extracted from opponent moves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TelemetryFeatures {
    /// Normalized shift of opponent center-of-mass along rows ([-1.0, 1.0])
    pub delta_r: f64,
    /// Normalized shift of opponent center-of-mass along columns ([-1.0, 1.0], typically >= 0)
    pub delta_c: f64,
    /// Placed rectangle area normalized by 36.0 ([1/36, 1.0])
    pub area_norm: f64,
    /// Normalized distance from placed rectangle center to board center (7.0, 19.5) in [0.0, 1.0]
    pub center_dist_norm: f64,
    /// Normalized distance from placed rectangle to nearest board edge in [0.0, 1.0]
    pub border_dist_norm: f64,
    /// Fraction of placed rectangle cells touching boundary rows 0 or 14 in [0.0, 1.0]
    pub perimeter_ratio: f64,
    /// Normalized minimum Manhattan distance from placed rectangle to friendly cells in [0.0, 1.0]
    pub contact_dist_norm: f64,
}

pub type MoveTelemetry = TelemetryFeatures;

impl TelemetryFeatures {
    /// Converts the 7 features into a fixed-size array.
    #[inline(always)]
    pub fn as_array(&self) -> [f64; 7] {
        [
            self.delta_r,
            self.delta_c,
            self.area_norm,
            self.center_dist_norm,
            self.border_dist_norm,
            self.perimeter_ratio,
            self.contact_dist_norm,
        ]
    }

    /// Constructs TelemetryFeatures from an array of 7 floats.
    #[inline(always)]
    pub fn from_array(arr: [f64; 7]) -> Self {
        Self {
            delta_r: arr[0],
            delta_c: arr[1],
            area_norm: arr[2],
            center_dist_norm: arr[3],
            border_dist_norm: arr[4],
            perimeter_ratio: arr[5],
            contact_dist_norm: arr[6],
        }
    }
}

/// The 4 tactical macro-clusters of opponent strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacroCluster {
    AntiRush = 0,
    AntiWallLoop = 1,
    AntiTurtleGreedy = 2,
    AntiBrawler = 3,
}

impl MacroCluster {
    #[inline(always)]
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(MacroCluster::AntiRush),
            1 => Some(MacroCluster::AntiWallLoop),
            2 => Some(MacroCluster::AntiTurtleGreedy),
            3 => Some(MacroCluster::AntiBrawler),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn name(self) -> &'static str {
        match self {
            MacroCluster::AntiRush => "AntiRush",
            MacroCluster::AntiWallLoop => "AntiWallLoop",
            MacroCluster::AntiTurtleGreedy => "AntiTurtleGreedy",
            MacroCluster::AntiBrawler => "AntiBrawler",
        }
    }
}

/// Calibrated default projection matrix W (4 macro-clusters x 7 telemetry features).
/// Features: [delta_r, delta_c, area_norm, center_dist_norm, border_dist_norm, perimeter_ratio, contact_dist_norm]
pub const DEFAULT_PROJECTION_WEIGHTS: [[f64; 7]; 4] = [
    // AntiRush (0)
    [-2.6419, 2.2591, -3.7912, -13.6581, -11.2493, 16.4809, -0.2096],
    // AntiWallLoop (1)
    [4.4524, -3.2022, 4.7247, 1.7072, 7.8826, 7.4465, 0.3584],
    // AntiTurtleGreedy (2)
    [-4.5547, -2.0751, -0.2437, 1.8510, -1.0513, -19.9653, 10.8025],
    // AntiBrawler (3)
    [2.7443, 3.0183, -0.6897, 10.1000, 4.4181, -3.9621, -25.8262],
];

/// Calibrated default bias vector b (4 macro-clusters).
pub const DEFAULT_BIAS_VECTOR: [f64; 4] = [5.6783, -7.3144, -5.9623, 2.1000];

/// Maximum allowable probability adjustment per turn (slew-rate clamp).
pub const MAX_SLEW_RATE_DELTA: f64 = 0.15;

/// Real-time opponent profiling and tactical classification engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyProfiler {
    /// Smoothed telemetry features via Exponential Moving Average (EMA).
    pub smoothed_features: [f64; 7],
    /// EMA smoothing factor alpha (default 0.3 for window N ~ 5.67 turns).
    pub ema_alpha: f64,
    /// Number of observations recorded so far.
    pub history_len: usize,
    /// Calibrated projection matrix W in R^{4x7}.
    pub weights: [[f64; 7]; 4],
    /// Calibrated bias vector b in R^4.
    pub bias: [f64; 4],
    /// Softmax probability distribution over the 4 macro-clusters.
    pub probabilities: [f64; 4],
    /// Confirmed dominant cluster with 2-turn hysteresis protection.
    pub confirmed_dominant: usize,
    /// Candidate dominant cluster currently being tracked for hysteresis.
    pub pending_dominant: usize,
    /// Number of consecutive turns the pending dominant cluster has led.
    pub pending_streak: usize,
}

impl EnemyProfiler {
    /// Constructs a new EnemyProfiler with default calibrated parameters.
    pub fn new() -> Self {
        Self {
            smoothed_features: [0.0; 7],
            ema_alpha: 0.3,
            history_len: 0,
            weights: DEFAULT_PROJECTION_WEIGHTS,
            bias: DEFAULT_BIAS_VECTOR,
            probabilities: [0.25; 4],
            confirmed_dominant: 0,
            pending_dominant: 0,
            pending_streak: 0,
        }
    }

    /// Extracts telemetry, updates EMA smoothing, and adjusts softmax probabilities with safeguards.
    pub fn update(
        &mut self,
        prev_board: &Board,
        current_board: &Board,
        enemy_move: &Move,
        player: Player,
    ) {
        let raw_features = extract_telemetry(prev_board, current_board, enemy_move, player);
        let raw_arr = raw_features.as_array();

        // 1. Exponential Moving Average (EMA) update with initial warmup
        if self.history_len == 0 {
            self.smoothed_features = raw_arr;
        } else {
            for (feat, &raw) in self.smoothed_features.iter_mut().zip(&raw_arr) {
                *feat = self.ema_alpha * raw + (1.0 - self.ema_alpha) * *feat;
            }
        }
        self.history_len += 1;

        // 2. Affine projection logits: z_k = sum_j (W_{kj} * F_j) + b_k
        let mut logits = [0.0; 4];
        for (k, logit) in logits.iter_mut().enumerate() {
            let mut dot = 0.0;
            for (j, &w) in self.weights[k].iter().enumerate() {
                dot += w * self.smoothed_features[j];
            }
            *logit = dot + self.bias[k];
        }

        // 3. Numerically stable Softmax
        let max_logit = logits.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mut exp_sum = 0.0;
        let mut raw_probs = [0.0; 4];
        for (k, prob) in raw_probs.iter_mut().enumerate() {
            let e = (logits[k] - max_logit).exp();
            *prob = e;
            exp_sum += e;
        }
        if exp_sum > 0.0 {
            for prob in &mut raw_probs {
                *prob /= exp_sum;
            }
        } else {
            raw_probs = [0.25; 4];
        }

        // 4. Hysteresis confirmation (2-turn persistence for dominant cluster switch)
        let raw_dominant = raw_probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        if self.history_len == 1 {
            self.confirmed_dominant = raw_dominant;
            self.pending_dominant = raw_dominant;
            self.pending_streak = 1;
        } else if raw_dominant == self.confirmed_dominant {
            self.pending_dominant = raw_dominant;
            self.pending_streak = 0;
        } else {
            if raw_dominant == self.pending_dominant {
                self.pending_streak += 1;
                if self.pending_streak >= 2 {
                    self.confirmed_dominant = raw_dominant;
                }
            } else {
                self.pending_dominant = raw_dominant;
                self.pending_streak = 1;
            }
        }

        // 5. Slew-rate clamping: delta p_k <= MAX_SLEW_RATE_DELTA per turn
        let mut max_diff = 0.0_f64;
        for (k, &raw_p) in raw_probs.iter().enumerate() {
            let diff = (raw_p - self.probabilities[k]).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }

        let lambda = if max_diff > MAX_SLEW_RATE_DELTA {
            MAX_SLEW_RATE_DELTA / max_diff
        } else {
            1.0
        };

        for (k, prob) in self.probabilities.iter_mut().enumerate() {
            *prob = (1.0 - lambda) * *prob + lambda * raw_probs[k];
        }
    }

    /// Returns the current probability vector over the 4 macro-clusters.
    #[inline(always)]
    pub fn get_probabilities(&self) -> [f64; 4] {
        self.probabilities
    }

    /// Alias for `get_probabilities` conforming to survey specification.
    #[inline(always)]
    pub fn get_macro_cluster_probabilities(&self) -> [f64; 4] {
        self.probabilities
    }

    /// Computes the normalized Shannon entropy H(p) in [0.0, 1.0].
    /// H(p) = - sum (p_k * ln(p_k)) / ln(4)
    /// Numerical guard: p_k <= 1e-12 => term = 0.
    pub fn normalized_entropy(&self) -> f64 {
        let ln_4 = 4.0_f64.ln(); // ~1.3862943611198906
        let mut entropy = 0.0;
        for &p in &self.probabilities {
            if p > 1e-12 {
                entropy -= p * p.ln();
            }
        }
        let norm_h = entropy / ln_4;
        norm_h.clamp(0.0, 1.0)
    }

    /// Alias for `normalized_entropy`.
    #[inline(always)]
    pub fn shannon_entropy(&self) -> f64 {
        self.normalized_entropy()
    }

    /// Returns the confirmed dominant macro-cluster index (0..3).
    #[inline(always)]
    pub fn dominant_cluster(&self) -> usize {
        self.confirmed_dominant
    }

    /// Helper alias to observe move conforming to alternative signature.
    pub fn observe_opponent_move(&mut self, prev_board: &Board, enemy_move: &Move) {
        let mut current_board = prev_board.clone();
        // Determine moving player from turn or cells
        let enemy_player = if prev_board.score(Player::Player1) <= prev_board.score(Player::Player2) {
            Player::Player1
        } else {
            Player::Player2
        };
        current_board.apply_move(enemy_player, enemy_move);
        self.update(prev_board, &current_board, enemy_move, enemy_player);
    }
}

impl Default for EnemyProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts the 7 normalized telemetry features by comparing board states before and after opponent's move.
/// Enforces canonical 180° rotation invariance so features are mathematically identical regardless of
/// whether the opponent plays as Player 1 or Player 2.
pub fn extract_telemetry(
    prev_board: &Board,
    current_board: &Board,
    enemy_move: &Move,
    player: Player,
) -> TelemetryFeatures {
    // Determine which player actually placed the move by inspecting board cells
    let enemy_player = if enemy_move.r < BOARD_HEIGHT && enemy_move.c < BOARD_WIDTH {
        let cell_val = current_board.get(enemy_move.r, enemy_move.c);
        if cell_val == player.cell_value() {
            player
        } else if cell_val == player.opponent().cell_value() {
            player.opponent()
        } else {
            player
        }
    } else {
        player
    };

    // Canonical representation: Map to frame where Opponent starts at (0, 0)
    // If Opponent is Player 1: already starts at (0, 0), advance toward (14, 39).
    // If Opponent is Player 2: starts at (14, 39); 180° rotation maps them to (0, 0).
    let (can_prev, can_curr, can_mv, can_enemy_val, can_own_val) = if enemy_player == Player::Player1 {
        (
            prev_board.clone(),
            current_board.clone(),
            *enemy_move,
            PLAYER_1,
            PLAYER_2,
        )
    } else {
        let rot_prev = prev_board.rotate_board_180();
        let rot_curr = current_board.rotate_board_180();
        let rot_mv = Move::new(
            BOARD_HEIGHT - (enemy_move.r + enemy_move.h),
            BOARD_WIDTH - (enemy_move.c + enemy_move.w),
            enemy_move.h,
            enemy_move.w,
        );
        (rot_prev, rot_curr, rot_mv, PLAYER_1, PLAYER_2)
    };

    let r0 = can_mv.r;
    let r1 = can_mv.r + can_mv.h - 1;
    let c0 = can_mv.c;
    let c1 = can_mv.c + can_mv.w - 1;

    // Feature 1 & 2: Shift of opponent center-of-mass (delta_r, delta_c)
    let mut prev_sum_r = 0.0;
    let mut prev_sum_c = 0.0;
    let mut prev_count = 0;
    for r in 0..BOARD_HEIGHT {
        let row_off = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            if can_prev.cells[row_off + c] == can_enemy_val {
                prev_sum_r += r as f64;
                prev_sum_c += c as f64;
                prev_count += 1;
            }
        }
    }

    let prev_mean_r = if prev_count > 0 {
        prev_sum_r / prev_count as f64
    } else {
        0.0 // Canonical start corner (0, 0)
    };
    let prev_mean_c = if prev_count > 0 {
        prev_sum_c / prev_count as f64
    } else {
        0.0
    };

    let mut curr_sum_r = 0.0;
    let mut curr_sum_c = 0.0;
    let mut curr_count = 0;
    for r in 0..BOARD_HEIGHT {
        let row_off = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            if can_curr.cells[row_off + c] == can_enemy_val {
                curr_sum_r += r as f64;
                curr_sum_c += c as f64;
                curr_count += 1;
            }
        }
    }

    let curr_mean_r = if curr_count > 0 {
        curr_sum_r / curr_count as f64
    } else {
        r0 as f64 + 0.5 * (can_mv.h as f64 - 1.0)
    };
    let curr_mean_c = if curr_count > 0 {
        curr_sum_c / curr_count as f64
    } else {
        c0 as f64 + 0.5 * (can_mv.w as f64 - 1.0)
    };

    let delta_r = ((curr_mean_r - prev_mean_r) / 14.0).clamp(-1.0, 1.0);
    let delta_c = ((curr_mean_c - prev_mean_c) / 39.0).clamp(-1.0, 1.0);

    // Feature 3: Placed rectangle area normalized by 36.0
    let area = (can_mv.h * can_mv.w) as f64;
    let area_norm = (area / 36.0).clamp(1.0 / 36.0, 1.0);

    // Feature 4: Normalized distance to board center (7.0, 19.5)
    let r_mid = r0 as f64 + 0.5 * (can_mv.h as f64 - 1.0);
    let c_mid = c0 as f64 + 0.5 * (can_mv.w as f64 - 1.0);
    let center_dist_norm = (
        ((r_mid - 7.0).abs() / 7.0) * 0.5 + ((c_mid - 19.5).abs() / 19.5) * 0.5
    ).clamp(0.0, 1.0);

    // Feature 5: Distance to nearest board edge (borders r=0, r=14, c=0, c=39)
    let dist_top = r0;
    let dist_bottom = 14usize.saturating_sub(r1);
    let dist_left = c0;
    let dist_right = 39usize.saturating_sub(c1);
    let min_border_dist = dist_top.min(dist_bottom).min(dist_left).min(dist_right);
    let border_dist_norm = (min_border_dist as f64 / 7.0).clamp(0.0, 1.0);

    // Feature 6: Perimeter ratio (fraction of placed cells touching boundary rows 0 or 14)
    let mut boundary_cells = 0;
    if r0 == 0 {
        boundary_cells += can_mv.w;
    }
    if r1 == 14 {
        boundary_cells += can_mv.w;
    }
    let perimeter_ratio = if area > 0.0 {
        (boundary_cells as f64 / area).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Feature 7: Minimum Manhattan distance to own player cells
    let mut min_manhattan = usize::MAX;
    let mut has_own_cells = false;
    for r in 0..BOARD_HEIGHT {
        let row_off = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            if can_curr.cells[row_off + c] == can_own_val {
                has_own_cells = true;
                let dr = if r < r0 {
                    r0 - r
                } else {
                    r.saturating_sub(r1)
                };
                let dc = if c < c0 {
                    c0 - c
                } else {
                    c.saturating_sub(c1)
                };
                let d = dr + dc;
                if d < min_manhattan {
                    min_manhattan = d;
                }
            }
        }
    }

    let contact_dist = if has_own_cells && min_manhattan != usize::MAX {
        min_manhattan
    } else {
        // Distance to own start corner (14, 39)
        let dr = 14usize.saturating_sub(r1);
        let dc = 39usize.saturating_sub(c1);
        dr + dc
    };

    let contact_dist_norm = (contact_dist as f64 / 53.0).clamp(0.0, 1.0);

    TelemetryFeatures {
        delta_r,
        delta_c,
        area_norm,
        center_dist_norm,
        border_dist_norm,
        perimeter_ratio,
        contact_dist_norm,
    }
}
