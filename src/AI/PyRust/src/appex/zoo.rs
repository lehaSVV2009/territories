//! Catalog of 10 Archetypes and PhaseShiftBot combinator for Territories.
//!
//! All archetypes strictly support:
//! - Canonical Player 2 perspective evaluation and 180° rotation invariance.
//! - Deterministic PRNG threading via Mulberry32.
//! - Trait-based polymorphic dispatch via `ArchetypeBot`.

use serde::{Deserialize, Serialize};
use crate::game::{
    can_close_loop, Board, Move, Mulberry32, Player, BOARD_HEIGHT, BOARD_WIDTH, PLAYER_1,
    PLAYER_2,
};

/// Floating point equality tolerance for heuristic scores.
const EPS: f64 = 1e-6;

/// Catalog of 10 distinct strategic archetypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    /// Fast advance along home flank (P2 r=12..14, maps to P1 r=0..2).
    FlankExpress,
    /// Direct raid across the board to seize enemy home flank (P2 r=0..2, maps to P1 r=12..14).
    CrossFlankRaider,
    /// Center control cutting the board in half (c=18..22, r=6..8).
    CenterChoker,
    /// Vertical barrier construction along meridians c=10, 20, 30.
    WallBuilder,
    /// Pure area maximizer (h * w) with deterministic tie-breaking.
    GreedyMaxArea,
    /// High compactness, minimal boundary perimeter, avoids noodles (1xN).
    CompactTurtle,
    /// Minimizes Manhattan distance to enemy cells and establishes contact.
    AggressiveChaser,
    /// Traps empty pockets and triggers closed loop captures (`findClosedLoops`).
    LoopHunter,
    /// Hugs outer boundary rows (0, 14) and columns (0, 39) with corner priority.
    PerimeterLurker,
    /// Pseudo-random legal move selection via deterministic Mulberry32 PRNG.
    ChaoticHopper,
}

impl Archetype {
    /// Returns the human-readable identifier for the archetype.
    pub fn name(&self) -> &'static str {
        match self {
            Archetype::FlankExpress => "FlankExpress",
            Archetype::CrossFlankRaider => "CrossFlankRaider",
            Archetype::CenterChoker => "CenterChoker",
            Archetype::WallBuilder => "WallBuilder",
            Archetype::GreedyMaxArea => "GreedyMaxArea",
            Archetype::CompactTurtle => "CompactTurtle",
            Archetype::AggressiveChaser => "AggressiveChaser",
            Archetype::LoopHunter => "LoopHunter",
            Archetype::PerimeterLurker => "PerimeterLurker",
            Archetype::ChaoticHopper => "ChaoticHopper",
        }
    }

    /// Returns an array of all 10 archetypes.
    pub fn all() -> [Archetype; 10] {
        [
            Archetype::FlankExpress,
            Archetype::CrossFlankRaider,
            Archetype::CenterChoker,
            Archetype::WallBuilder,
            Archetype::GreedyMaxArea,
            Archetype::CompactTurtle,
            Archetype::AggressiveChaser,
            Archetype::LoopHunter,
            Archetype::PerimeterLurker,
            Archetype::ChaoticHopper,
        ]
    }

    /// Selects a move with 180° rotation invariance.
    pub fn select_move(
        &self,
        board: &Board,
        player: Player,
        d1: usize,
        d2: usize,
        rng: Option<&mut Mulberry32>,
    ) -> Option<Move> {
        select_archetype_move(*self, board, player, d1, d2, rng)
    }

    /// Creates a boxed `ArchetypeBot` instance for this archetype.
    pub fn to_bot(&self, seed: u32) -> Box<dyn ArchetypeBot> {
        match self {
            Archetype::FlankExpress => Box::new(FlankExpressBot::new(seed)),
            Archetype::CrossFlankRaider => Box::new(CrossFlankRaiderBot::new(seed)),
            Archetype::CenterChoker => Box::new(CenterChokerBot::new(seed)),
            Archetype::WallBuilder => Box::new(WallBuilderBot::new(seed)),
            Archetype::GreedyMaxArea => Box::new(GreedyMaxAreaBot::new(seed)),
            Archetype::CompactTurtle => Box::new(CompactTurtleBot::new(seed)),
            Archetype::AggressiveChaser => Box::new(AggressiveChaserBot::new(seed)),
            Archetype::LoopHunter => Box::new(LoopHunterBot::new(seed)),
            Archetype::PerimeterLurker => Box::new(PerimeterLurkerBot::new(seed)),
            Archetype::ChaoticHopper => Box::new(ChaoticHopperBot::new(seed)),
        }
    }
}

/// Common trait for all bot implementations in the Zoo.
pub trait ArchetypeBot: Send + Sync {
    /// Selects the best move given the board state and dice roll.
    fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move>;

    /// Returns the human-readable name of the bot.
    fn name(&self) -> &'static str {
        "ArchetypeBot"
    }

    /// Returns the active archetype if this bot corresponds to one.
    fn archetype(&self) -> Option<Archetype> {
        None
    }
}

impl ArchetypeBot for Archetype {
    fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
        let mut rng = Mulberry32::new(board.turn as u32 ^ 0x9e3779b9);
        select_archetype_move(*self, board, player, d1, d2, Some(&mut rng))
    }

    fn name(&self) -> &'static str {
        Archetype::name(self)
    }

    fn archetype(&self) -> Option<Archetype> {
        Some(*self)
    }
}

macro_rules! define_archetype_bot {
    ($struct_name:ident, $archetype:expr, $doc:expr) => {
        #[doc = $doc]
        pub struct $struct_name {
            pub rng: Mulberry32,
        }

        impl $struct_name {
            pub fn new(seed: u32) -> Self {
                Self {
                    rng: Mulberry32::new(seed),
                }
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new(123456789)
            }
        }

        impl ArchetypeBot for $struct_name {
            fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
                select_archetype_move($archetype, board, player, d1, d2, Some(&mut self.rng))
            }

            fn name(&self) -> &'static str {
                $archetype.name()
            }

            fn archetype(&self) -> Option<Archetype> {
                Some($archetype)
            }
        }
    };
}

define_archetype_bot!(FlankExpressBot, Archetype::FlankExpress, "Flank Express bot.");
define_archetype_bot!(CrossFlankRaiderBot, Archetype::CrossFlankRaider, "Cross-Flank Raider bot.");
define_archetype_bot!(CenterChokerBot, Archetype::CenterChoker, "Center Choker bot.");
define_archetype_bot!(WallBuilderBot, Archetype::WallBuilder, "Wall Builder bot.");
define_archetype_bot!(GreedyMaxAreaBot, Archetype::GreedyMaxArea, "Greedy MaxArea bot.");
define_archetype_bot!(CompactTurtleBot, Archetype::CompactTurtle, "Compact Turtle bot.");
define_archetype_bot!(AggressiveChaserBot, Archetype::AggressiveChaser, "Aggressive Chaser bot.");
define_archetype_bot!(LoopHunterBot, Archetype::LoopHunter, "Loop Hunter bot.");
define_archetype_bot!(PerimeterLurkerBot, Archetype::PerimeterLurker, "Perimeter Lurker bot.");
define_archetype_bot!(ChaoticHopperBot, Archetype::ChaoticHopper, "Chaotic Hopper bot.");

/// Policy configuration for dynamic strategy transitions in `PhaseShiftBot`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhaseShiftPolicy {
    /// Sequential strategy transitions at turn thresholds T1 and T2.
    Sequential {
        opening: Archetype,
        midgame: Archetype,
        endgame: Archetype,
        t1: usize,
        t2: usize,
    },
    /// Stochastic mixture on every turn between two archetypes.
    StochasticMixer {
        bot_a: Archetype,
        bot_b: Archetype,
        prob_a: f64,
    },
    /// Dynamic switching driven by board events (contact distance and empty cells).
    EventDriven {
        base_bot: Archetype,
        contact_bot: Archetype,
        endgame_bot: Archetype,
        contact_dist_threshold: usize,
        empty_cells_threshold: usize,
    },
}

/// Combinatorial decorator for dynamic multi-phase strategy switching.
pub struct PhaseShiftBot {
    pub phases: Vec<(usize, Box<dyn ArchetypeBot>)>,
    pub current_phase_idx: usize,
    pub rng: Mulberry32,
    pub policy: Option<PhaseShiftPolicy>,
}

impl PhaseShiftBot {
    /// Creates a `PhaseShiftBot` from a vector of `(turn_threshold, bot)` pairs.
    pub fn new(phases: Vec<(usize, Box<dyn ArchetypeBot>)>, seed: u32) -> Self {
        Self {
            phases,
            current_phase_idx: 0,
            rng: Mulberry32::new(seed),
            policy: None,
        }
    }

    /// Creates a sequential `PhaseShiftBot` transitioning at `t1` and `t2`.
    pub fn new_sequential(
        opening: Archetype,
        midgame: Archetype,
        endgame: Archetype,
        t1: usize,
        t2: usize,
        seed: u32,
    ) -> Self {
        let phases: Vec<(usize, Box<dyn ArchetypeBot>)> = vec![
            (t1, opening.to_bot(seed)),
            (t2, midgame.to_bot(seed.wrapping_add(1))),
            (usize::MAX, endgame.to_bot(seed.wrapping_add(2))),
        ];
        Self {
            phases,
            current_phase_idx: 0,
            rng: Mulberry32::new(seed),
            policy: Some(PhaseShiftPolicy::Sequential {
                opening,
                midgame,
                endgame,
                t1,
                t2,
            }),
        }
    }

    /// Creates a stochastic mixing `PhaseShiftBot` choosing between `bot_a` (probability `prob_a`) and `bot_b`.
    pub fn new_stochastic(bot_a: Archetype, bot_b: Archetype, prob_a: f64, seed: u32) -> Self {
        Self {
            phases: Vec::new(),
            current_phase_idx: 0,
            rng: Mulberry32::new(seed),
            policy: Some(PhaseShiftPolicy::StochasticMixer {
                bot_a,
                bot_b,
                prob_a,
            }),
        }
    }

    /// Creates an event-driven `PhaseShiftBot` transitioning based on piece contact and empty cells.
    pub fn new_event_driven(
        base_bot: Archetype,
        contact_bot: Archetype,
        endgame_bot: Archetype,
        contact_dist_threshold: usize,
        empty_cells_threshold: usize,
        seed: u32,
    ) -> Self {
        Self {
            phases: Vec::new(),
            current_phase_idx: 0,
            rng: Mulberry32::new(seed),
            policy: Some(PhaseShiftPolicy::EventDriven {
                base_bot,
                contact_bot,
                endgame_bot,
                contact_dist_threshold,
                empty_cells_threshold,
            }),
        }
    }

    /// Returns the currently active archetype based on board state.
    pub fn active_archetype(&self, board: &Board) -> Option<Archetype> {
        if let Some(ref policy) = self.policy {
            match policy {
                PhaseShiftPolicy::Sequential {
                    opening,
                    midgame,
                    endgame,
                    t1,
                    t2,
                } => {
                    if board.turn <= *t1 {
                        Some(*opening)
                    } else if board.turn <= *t2 {
                        Some(*midgame)
                    } else {
                        Some(*endgame)
                    }
                }
                PhaseShiftPolicy::StochasticMixer { bot_a, .. } => Some(*bot_a),
                PhaseShiftPolicy::EventDriven {
                    base_bot,
                    contact_bot,
                    endgame_bot,
                    contact_dist_threshold,
                    empty_cells_threshold,
                } => {
                    let empty = board.empty_cells_count();
                    let contact_dist = compute_min_contact_distance(board);
                    if empty < *empty_cells_threshold {
                        Some(*endgame_bot)
                    } else if contact_dist <= *contact_dist_threshold {
                        Some(*contact_bot)
                    } else {
                        Some(*base_bot)
                    }
                }
            }
        } else if !self.phases.is_empty() {
            let mut chosen_idx = self.phases.len() - 1;
            for (idx, (threshold, _)) in self.phases.iter().enumerate() {
                if board.turn <= *threshold {
                    chosen_idx = idx;
                    break;
                }
            }
            self.phases[chosen_idx].1.archetype()
        } else {
            None
        }
    }
}

impl ArchetypeBot for PhaseShiftBot {
    fn select_move(&mut self, board: &Board, player: Player, d1: usize, d2: usize) -> Option<Move> {
        if let Some(ref policy) = self.policy {
            match policy {
                PhaseShiftPolicy::Sequential {
                    opening,
                    midgame,
                    endgame,
                    t1,
                    t2,
                } => {
                    let arch = if board.turn <= *t1 {
                        *opening
                    } else if board.turn <= *t2 {
                        *midgame
                    } else {
                        *endgame
                    };
                    select_archetype_move(arch, board, player, d1, d2, Some(&mut self.rng))
                }
                PhaseShiftPolicy::StochasticMixer {
                    bot_a,
                    bot_b,
                    prob_a,
                } => {
                    let r = self.rng.next_f64();
                    let arch = if r < *prob_a { *bot_a } else { *bot_b };
                    select_archetype_move(arch, board, player, d1, d2, Some(&mut self.rng))
                }
                PhaseShiftPolicy::EventDriven {
                    base_bot,
                    contact_bot,
                    endgame_bot,
                    contact_dist_threshold,
                    empty_cells_threshold,
                } => {
                    let empty = board.empty_cells_count();
                    let contact_dist = compute_min_contact_distance(board);
                    let arch = if empty < *empty_cells_threshold {
                        *endgame_bot
                    } else if contact_dist <= *contact_dist_threshold {
                        *contact_bot
                    } else {
                        *base_bot
                    };
                    select_archetype_move(arch, board, player, d1, d2, Some(&mut self.rng))
                }
            }
        } else if !self.phases.is_empty() {
            let mut chosen_idx = self.phases.len() - 1;
            for (idx, (threshold, _)) in self.phases.iter().enumerate() {
                if board.turn <= *threshold {
                    chosen_idx = idx;
                    break;
                }
            }
            self.current_phase_idx = chosen_idx;
            self.phases[chosen_idx].1.select_move(board, player, d1, d2)
        } else {
            None
        }
    }

    fn name(&self) -> &'static str {
        "PhaseShiftBot"
    }
}

/// Dispatches move selection for a given archetype with canonical 180° rotation invariance.
///
/// If `player == Player::Player1`:
/// 1. Rotates board 180° (`board.rotate_board_180()`).
/// 2. Evaluates optimal move from Player 2 perspective.
/// 3. Inverses move coordinates back to Player 1 frame (`Board::inverse_move_180`).
pub fn select_archetype_move(
    archetype: Archetype,
    board: &Board,
    player: Player,
    d1: usize,
    d2: usize,
    rng: Option<&mut Mulberry32>,
) -> Option<Move> {
    if player == Player::Player1 {
        let rotated = board.rotate_board_180();
        let rot_mv = select_archetype_move_p2(archetype, &rotated, d1, d2, rng)?;
        Some(Board::inverse_move_180(&rot_mv))
    } else {
        select_archetype_move_p2(archetype, board, d1, d2, rng)
    }
}

/// Computes minimum Manhattan distance between any Player 1 cell and any Player 2 cell.
pub fn compute_min_contact_distance(board: &Board) -> usize {
    let mut p1_cells: Vec<(usize, usize)> = Vec::new();
    let mut p2_cells: Vec<(usize, usize)> = Vec::new();

    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            let v = board.cells[row_offset + c];
            if v == PLAYER_1 {
                p1_cells.push((r, c));
            } else if v == PLAYER_2 {
                p2_cells.push((r, c));
            }
        }
    }

    if p1_cells.is_empty() || p2_cells.is_empty() {
        return BOARD_HEIGHT + BOARD_WIDTH;
    }

    let mut min_d = usize::MAX;
    for &(r1, c1) in &p1_cells {
        for &(r2, c2) in &p2_cells {
            let d = r1.abs_diff(r2) + c1.abs_diff(c2);
            if d < min_d {
                min_d = d;
            }
        }
    }
    min_d
}

/// Evaluates candidate moves from canonical Player 2 perspective.
fn select_archetype_move_p2(
    archetype: Archetype,
    board: &Board,
    d1: usize,
    d2: usize,
    rng: Option<&mut Mulberry32>,
) -> Option<Move> {
    let all_rects = board.generate_legal_moves(Player::Player2, d1, d2);
    if all_rects.is_empty() {
        return None;
    }

    match archetype {
        Archetype::FlankExpress => select_flank_express_p2(&all_rects),
        Archetype::CrossFlankRaider => select_cross_flank_raider_p2(&all_rects),
        Archetype::CenterChoker => select_center_choker_p2(&all_rects),
        Archetype::WallBuilder => select_wall_builder_p2(board, &all_rects),
        Archetype::GreedyMaxArea => select_greedy_max_area_p2(&all_rects),
        Archetype::CompactTurtle => select_compact_turtle_p2(board, &all_rects),
        Archetype::AggressiveChaser => select_aggressive_chaser_p2(board, &all_rects),
        Archetype::LoopHunter => select_loop_hunter_p2(board, &all_rects),
        Archetype::PerimeterLurker => select_perimeter_lurker_p2(&all_rects),
        Archetype::ChaoticHopper => select_chaotic_hopper_p2(board, d1, d2, &all_rects, rng),
    }
}

// -----------------------------------------------------------------------------
// Individual Archetype Evaluators (Canonical P2 Perspective)
// -----------------------------------------------------------------------------

/// 1. Flank Express: Advance along home flank (rows 12..14).
fn select_flank_express_p2(all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let r1 = cand.r + cand.h - 1;
        let c0 = cand.c as f64;

        let mut flank_bonus = 0.0;
        if cand.r + cand.h > 12 {
            flank_bonus += 600.0;
        }
        if cand.r + cand.h == 15 {
            flank_bonus += 300.0;
        }

        let off_flank_penalty = (14.0 - r1 as f64) * 90.0;
        let score = area * 100.0 + (39.0 - c0) * 60.0 + flank_bonus - off_flank_penalty;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: smallest c0 (furthest west), then largest area, smallest r0
            if cand.c < best_cand.c
                || (cand.c == best_cand.c && cand.area() > best_cand.area())
                || (cand.c == best_cand.c && cand.area() == best_cand.area() && cand.r < best_cand.r)
            {
                best_score = score;
                best_cand = cand;
            }
        }
    }

    Some(best_cand)
}

/// 2. Cross-Flank Raider: Strike northward to seize enemy corridor (rows 0..2).
fn select_cross_flank_raider_p2(all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let r0 = cand.r as f64;
        let c0 = cand.c as f64;

        let mut raider_bonus = 0.0;
        if cand.r <= 2 {
            raider_bonus += 700.0;
        }
        if cand.r == 0 {
            raider_bonus += 350.0;
        }

        let score = area * 100.0 + (39.0 - c0) * 40.0 + (14.0 - r0) * 90.0 + raider_bonus;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: smallest r0 (furthest north), then smallest c0, largest area
            if cand.r < best_cand.r
                || (cand.r == best_cand.r && cand.c < best_cand.c)
                || (cand.r == best_cand.r && cand.c == best_cand.c && cand.area() > best_cand.area())
            {
                best_score = score;
                best_cand = cand;
            }
        }
    }

    Some(best_cand)
}

/// 3. Center Choker: Drive into geometric center box (c=18..22, r=6..8).
fn select_center_choker_p2(all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let center_r = cand.r as f64 + 0.5 * cand.h as f64;
        let center_c = cand.c as f64 + 0.5 * cand.w as f64;
        let dist_center = (center_r - 7.0).abs() * 2.5 + (center_c - 19.5).abs() * 1.2;

        let r1 = cand.r + cand.h - 1;
        let c1 = cand.c + cand.w - 1;
        let mut core_bonus = 0.0;
        // Core box: r in [6..8], c in [18..22]
        if cand.r <= 8 && r1 >= 6 && cand.c <= 22 && c1 >= 18 {
            core_bonus = 850.0;
        }

        let score = area * 80.0 - dist_center * 75.0 + core_bonus;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: smallest dist_center, then largest area, smallest r0, smallest c0
            let best_r = best_cand.r as f64 + 0.5 * best_cand.h as f64;
            let best_c = best_cand.c as f64 + 0.5 * best_cand.w as f64;
            let best_dist = (best_r - 7.0).abs() * 2.5 + (best_c - 19.5).abs() * 1.2;

            if dist_center < best_dist - EPS
                || ((dist_center - best_dist).abs() <= EPS && cand.area() > best_cand.area())
                || ((dist_center - best_dist).abs() <= EPS
                    && cand.area() == best_cand.area()
                    && cand.r < best_cand.r)
                || ((dist_center - best_dist).abs() <= EPS
                    && cand.area() == best_cand.area()
                    && cand.r == best_cand.r
                    && cand.c < best_cand.c)
            {
                best_score = score;
                best_cand = cand;
            }
        }
    }

    Some(best_cand)
}

/// 4. Wall Builder: Build vertical walls along meridians c=10, 20, 30.
fn select_wall_builder_p2(board: &Board, all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let aspect = (cand.h as f64 - cand.w as f64) * 80.0;
        let c1 = cand.c + cand.w - 1;

        let mut meridian_bonus = 0.0;
        for &m in &[9, 10, 19, 20, 29, 30] {
            if cand.c <= m && m <= c1 {
                meridian_bonus = 500.0;
                break;
            }
        }

        let mut chain_count = 0;
        for c in cand.c..=c1 {
            if cand.r > 0 && board.get(cand.r - 1, c) == PLAYER_2 {
                chain_count += 1;
            }
            if cand.r + cand.h < BOARD_HEIGHT && board.get(cand.r + cand.h, c) == PLAYER_2 {
                chain_count += 1;
            }
        }
        let chain_bonus = (chain_count as f64) * 300.0;
        let score = area * 40.0 + aspect + meridian_bonus + chain_bonus;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
        } else if (score - best_score).abs() <= EPS {
            // Proximity to nearest meridian {10, 20, 30}
            let dist_m = |c0: usize, w: usize| -> usize {
                let end = c0 + w - 1;
                [10usize, 20, 30]
                    .iter()
                    .map(|&m| {
                        if m < c0 {
                            c0 - m
                        } else {
                            m.saturating_sub(end)
                        }
                    })
                    .min()
                    .unwrap_or(0)
            };

            let cand_dist = dist_m(cand.c, cand.w);
            let best_dist = dist_m(best_cand.c, best_cand.w);

            if cand.h > best_cand.h
                || (cand.h == best_cand.h && cand_dist < best_dist)
                || (cand.h == best_cand.h && cand_dist == best_dist && cand.area() > best_cand.area())
                || (cand.h == best_cand.h
                    && cand_dist == best_dist
                    && cand.area() == best_cand.area()
                    && cand.r < best_cand.r)
            {
                best_score = score;
                best_cand = cand;
            }
        }
    }

    Some(best_cand)
}

/// 5. Greedy MaxArea: Absolute maximum area h * w.
fn select_greedy_max_area_p2(all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut max_area = best_cand.area();

    for &cand in all_rects.iter().skip(1) {
        let area = cand.area();
        if area > max_area
            || (area == max_area && cand.r < best_cand.r)
            || (area == max_area && cand.r == best_cand.r && cand.c < best_cand.c)
        {
            max_area = area;
            best_cand = cand;
        }
    }

    Some(best_cand)
}

/// 6. Compact Turtle: High aspect ratio, dense clusters, avoids noodles.
fn select_compact_turtle_p2(board: &Board, all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;

    for &cand in all_rects {
        let min_dim = cand.h.min(cand.w) as f64;
        let max_dim = cand.h.max(cand.w) as f64;
        let aspect_ratio = min_dim / max_dim;
        let area = (cand.h * cand.w) as f64;
        let r1 = cand.r + cand.h - 1;
        let c1 = cand.c + cand.w - 1;

        let mut friendly_contacts = 0;
        if cand.r > 0 {
            for c in cand.c..=c1 {
                if board.get(cand.r - 1, c) == PLAYER_2 {
                    friendly_contacts += 1;
                }
            }
        }
        if r1 + 1 < BOARD_HEIGHT {
            for c in cand.c..=c1 {
                if board.get(r1 + 1, c) == PLAYER_2 {
                    friendly_contacts += 1;
                }
            }
        }
        if cand.c > 0 {
            for r in cand.r..=r1 {
                if board.get(r, cand.c - 1) == PLAYER_2 {
                    friendly_contacts += 1;
                }
            }
        }
        if c1 + 1 < BOARD_WIDTH {
            for r in cand.r..=r1 {
                if board.get(r, c1 + 1) == PLAYER_2 {
                    friendly_contacts += 1;
                }
            }
        }

        let noodle_penalty = if cand.h.min(cand.w) == 1 && cand.h * cand.w > 2 {
            600.0
        } else {
            0.0
        };

        let score = area * 60.0
            + aspect_ratio * 350.0
            + (friendly_contacts as f64) * 120.0
            - noodle_penalty
            + (cand.c as f64) * 15.0;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: highest friendly_contacts, largest min_dim, largest area, smallest r0, smallest c0
            let best_r1 = best_cand.r + best_cand.h - 1;
            let best_c1 = best_cand.c + best_cand.w - 1;
            let mut best_contacts = 0;
            if best_cand.r > 0 {
                for c in best_cand.c..=best_c1 {
                    if board.get(best_cand.r - 1, c) == PLAYER_2 {
                        best_contacts += 1;
                    }
                }
            }
            if best_r1 + 1 < BOARD_HEIGHT {
                for c in best_cand.c..=best_c1 {
                    if board.get(best_r1 + 1, c) == PLAYER_2 {
                        best_contacts += 1;
                    }
                }
            }
            if best_cand.c > 0 {
                for r in best_cand.r..=best_r1 {
                    if board.get(r, best_cand.c - 1) == PLAYER_2 {
                        best_contacts += 1;
                    }
                }
            }
            if best_c1 + 1 < BOARD_WIDTH {
                for r in best_cand.r..=best_r1 {
                    if board.get(r, best_c1 + 1) == PLAYER_2 {
                        best_contacts += 1;
                    }
                }
            }

            if friendly_contacts > best_contacts
                || (friendly_contacts == best_contacts && cand.h.min(cand.w) > best_cand.h.min(best_cand.w))
                || (friendly_contacts == best_contacts
                    && cand.h.min(cand.w) == best_cand.h.min(best_cand.w)
                    && cand.area() > best_cand.area())
                || (friendly_contacts == best_contacts
                    && cand.h.min(cand.w) == best_cand.h.min(best_cand.w)
                    && cand.area() == best_cand.area()
                    && cand.r < best_cand.r)
            {
                best_score = score;
                best_cand = cand;
            }
        }
    }

    Some(best_cand)
}

/// 7. Aggressive Chaser: Minimize Manhattan distance to enemy pieces.
fn select_aggressive_chaser_p2(board: &Board, all_rects: &[Move]) -> Option<Move> {
    let mut opp_cells: Vec<(usize, usize)> = Vec::with_capacity(32);
    for r in 0..BOARD_HEIGHT {
        let row_offset = r * BOARD_WIDTH;
        for c in 0..BOARD_WIDTH {
            if board.cells[row_offset + c] == PLAYER_1 {
                opp_cells.push((r, c));
            }
        }
    }

    if opp_cells.is_empty() {
        opp_cells.push((0, 0));
    }

    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;
    let mut best_min_dist = usize::MAX;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let r1 = cand.r + cand.h - 1;
        let c1 = cand.c + cand.w - 1;

        let mut min_dist = usize::MAX;
        for &(er, ec) in &opp_cells {
            let dr = if er < cand.r {
                cand.r - er
            } else {
                er.saturating_sub(r1)
            };
            let dc = if ec < cand.c {
                cand.c - ec
            } else {
                ec.saturating_sub(c1)
            };
            let d = dr + dc;
            if d < min_dist {
                min_dist = d;
            }
        }

        let direct_contact_bonus = if min_dist <= 1 { 1200.0 } else { 0.0 };
        let score = area * 70.0 - (min_dist as f64) * 160.0 + direct_contact_bonus + (39.0 - cand.c as f64) * 25.0;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
            best_min_dist = min_dist;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: smallest min_dist, largest area, smallest r0, smallest c0
            if min_dist < best_min_dist
                || (min_dist == best_min_dist && cand.area() > best_cand.area())
                || (min_dist == best_min_dist && cand.area() == best_cand.area() && cand.r < best_cand.r)
                || (min_dist == best_min_dist
                    && cand.area() == best_cand.area()
                    && cand.r == best_cand.r
                    && cand.c < best_cand.c)
            {
                best_score = score;
                best_cand = cand;
                best_min_dist = min_dist;
            }
        }
    }

    Some(best_cand)
}

/// 8. Loop Hunter: Creates enclosure traps and captures closed loops.
fn select_loop_hunter_p2(board: &Board, all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;
    let mut best_caps = 0usize;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let can_close = can_close_loop(board, cand.r, cand.c, cand.h, cand.w, Player::Player2);

        let mut captured_cells = 0;
        if can_close {
            let mut test_board = board.clone();
            test_board.apply_move(Player::Player2, &cand);
            let (_, p2_caps) = test_board.find_closed_loops();
            captured_cells = p2_caps.len();
        }

        let immediate_capture_bonus = (captured_cells as f64) * 35000.0;
        let pocket_shape_bonus = if can_close { 800.0 } else { 0.0 };
        let score = area * 50.0 + immediate_capture_bonus + pocket_shape_bonus;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
            best_caps = captured_cells;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: largest captured_cells, largest area, smallest r0, smallest c0
            if captured_cells > best_caps
                || (captured_cells == best_caps && cand.area() > best_cand.area())
                || (captured_cells == best_caps && cand.area() == best_cand.area() && cand.r < best_cand.r)
                || (captured_cells == best_caps
                    && cand.area() == best_cand.area()
                    && cand.r == best_cand.r
                    && cand.c < best_cand.c)
            {
                best_score = score;
                best_cand = cand;
                best_caps = captured_cells;
            }
        }
    }

    Some(best_cand)
}

/// 9. Perimeter Lurker: Hugs outer boundary rows (0, 14) and columns (0, 39).
fn select_perimeter_lurker_p2(all_rects: &[Move]) -> Option<Move> {
    let mut best_cand = all_rects[0];
    let mut best_score = -f64::INFINITY;
    let mut best_perim = 0usize;

    for &cand in all_rects {
        let area = (cand.h * cand.w) as f64;
        let r1 = cand.r + cand.h - 1;
        let c1 = cand.c + cand.w - 1;

        let mut perim_cells = 0;
        for r in cand.r..=r1 {
            for c in cand.c..=c1 {
                if r == 0 || r == BOARD_HEIGHT - 1 || c == 0 || c == BOARD_WIDTH - 1 {
                    perim_cells += 1;
                }
            }
        }

        let mut corner_bonus = 0.0;
        if (cand.r == 0 && cand.c == 0)
            || (cand.r == 0 && c1 == BOARD_WIDTH - 1)
            || (r1 == BOARD_HEIGHT - 1 && cand.c == 0)
            || (r1 == BOARD_HEIGHT - 1 && c1 == BOARD_WIDTH - 1)
        {
            corner_bonus = 500.0;
        }

        let interior_penalty = if perim_cells == 0 { 500.0 } else { 0.0 };
        let score = area * 50.0 + (perim_cells as f64) * 280.0 + corner_bonus - interior_penalty;

        if score > best_score + EPS {
            best_score = score;
            best_cand = cand;
            best_perim = perim_cells;
        } else if (score - best_score).abs() <= EPS {
            // Tie-breaker: highest perim_cells, largest area, smallest r0, smallest c0
            if perim_cells > best_perim
                || (perim_cells == best_perim && cand.area() > best_cand.area())
                || (perim_cells == best_perim && cand.area() == best_cand.area() && cand.r < best_cand.r)
                || (perim_cells == best_perim
                    && cand.area() == best_cand.area()
                    && cand.r == best_cand.r
                    && cand.c < best_cand.c)
            {
                best_score = score;
                best_cand = cand;
                best_perim = perim_cells;
            }
        }
    }

    Some(best_cand)
}

/// 10. Chaotic Hopper: Pseudo-random legal move selection via Mulberry32 PRNG.
fn select_chaotic_hopper_p2(
    board: &Board,
    d1: usize,
    d2: usize,
    all_rects: &[Move],
    rng: Option<&mut Mulberry32>,
) -> Option<Move> {
    if let Some(r) = rng {
        let idx = (r.next_f64() * all_rects.len() as f64).floor() as usize;
        Some(all_rects[idx.min(all_rects.len() - 1)])
    } else {
        // Deterministic fallback based on turn and dice rolls
        let idx = (board.turn * 31 + d1 * 7 + d2) % all_rects.len();
        Some(all_rects[idx])
    }
}
