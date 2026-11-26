mod eval;
mod search;
mod tt;
mod zobrist;

pub use eval::{default_spatial_matrix, evaluate_board_state};
pub use search::{WhaleBot, WhaleStats};
pub use tt::{TTEntry, TTFlag, TranspositionTable};

use crate::bitboard::rect_mask;
use crate::topology::find_closed_loops;
use crate::types::{Bitboard, GameState, Move};

pub const DEFAULT_WHALE_DEPTH: u8 = 3;
pub const MAX_SEARCH_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhalePlayer {
    Player1,
    Player2,
}

impl WhalePlayer {
    #[inline(always)]
    pub fn opponent(self) -> Self {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        match self {
            Self::Player1 => 0,
            Self::Player2 => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhaleMove {
    pub r: u8,
    pub c: u8,
    pub h: u8,
    pub w: u8,
}

impl WhaleMove {
    #[inline(always)]
    pub fn from_move(mv: Move) -> Self {
        Self {
            r: mv.y,
            c: mv.x,
            h: mv.h,
            w: mv.w,
        }
    }

    #[inline(always)]
    pub fn to_move(self) -> Move {
        Move {
            x: self.c,
            y: self.r,
            w: self.w,
            h: self.h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WhaleBoard {
    pub p1_bb: Bitboard,
    pub p2_bb: Bitboard,
    pub p1_score: u16,
    pub p2_score: u16,
    pub passes: u8,
}

impl WhaleBoard {
    pub fn from_game_state(state: &GameState) -> (Self, WhalePlayer) {
        let is_p1 = (state.turn % 2) == 0;
        if is_p1 {
            (
                Self {
                    p1_bb: state.my_bb,
                    p2_bb: state.opp_bb,
                    p1_score: state.my_score,
                    p2_score: state.opp_score,
                    passes: 0,
                },
                WhalePlayer::Player1,
            )
        } else {
            (
                Self {
                    p1_bb: state.opp_bb,
                    p2_bb: state.my_bb,
                    p1_score: state.opp_score,
                    p2_score: state.my_score,
                    passes: 0,
                },
                WhalePlayer::Player2,
            )
        }
    }

    #[inline(always)]
    pub fn score(&self, player: WhalePlayer) -> u16 {
        match player {
            WhalePlayer::Player1 => self.p1_score,
            WhalePlayer::Player2 => self.p2_score,
        }
    }

    #[inline(always)]
    pub fn bb(&self, player: WhalePlayer) -> &Bitboard {
        match player {
            WhalePlayer::Player1 => &self.p1_bb,
            WhalePlayer::Player2 => &self.p2_bb,
        }
    }

    #[inline]
    pub fn apply_move_and_loops(&mut self, player: WhalePlayer, mv: WhaleMove) {
        let r_bb = rect_mask(mv.c, mv.r, mv.w, mv.h);
        let area = (mv.h as u16) * (mv.w as u16);
        match player {
            WhalePlayer::Player1 => {
                self.p1_bb = self.p1_bb | r_bb;
                self.p1_score += area;
            }
            WhalePlayer::Player2 => {
                self.p2_bb = self.p2_bb | r_bb;
                self.p2_score += area;
            }
        }
        if !self.p1_bb.is_empty() && !self.p2_bb.is_empty() {
            let (cap_p1, cap_p2) = find_closed_loops(&self.p1_bb, &self.p2_bb);
            if !cap_p1.is_empty() {
                self.p1_bb = self.p1_bb | cap_p1;
                self.p1_score += cap_p1.count_ones();
            }
            if !cap_p2.is_empty() {
                self.p2_bb = self.p2_bb | cap_p2;
                self.p2_score += cap_p2.count_ones();
            }
        }
        self.passes = 0;
    }

    pub(super) fn to_game_state(&self, player: WhalePlayer) -> GameState {
        match player {
            WhalePlayer::Player1 => GameState {
                my_bb: self.p1_bb,
                opp_bb: self.p2_bb,
                my_score: self.p1_score,
                opp_score: self.p2_score,
                turn: 0,
            },
            WhalePlayer::Player2 => GameState {
                my_bb: self.p2_bb,
                opp_bb: self.p1_bb,
                my_score: self.p2_score,
                opp_score: self.p1_score,
                turn: 1,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeuristicWeights {
    pub weight_area: f64,
    pub weight_voronoi_delta: f64,
    pub weight_safe_territory: f64,
    pub weight_spatial_matrix: f64,
}

impl Default for HeuristicWeights {
    fn default() -> Self {
        Self {
            weight_area: 50.0,
            weight_voronoi_delta: 26.0,
            weight_safe_territory: 25.0,
            weight_spatial_matrix: 0.32,
        }
    }
}
