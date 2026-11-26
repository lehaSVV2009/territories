use crate::types::{Afterstate, Bitboard, DiceRoll, GameState, Move, HEIGHT, WIDTH};
use std::ops::{BitAnd, BitOr, Not};

pub const BOARD_MASK: Bitboard = compute_board_mask();
pub const NOT_COL_0_MASK: Bitboard = compute_not_col_0_mask();
pub const NOT_LAST_COL_MASK: Bitboard = compute_not_last_col_mask();

const fn compute_board_mask() -> Bitboard {
    let mut words = [!0u64; 10];
    words[9] = (1u64 << 24) - 1;
    Bitboard(words)
}

const fn compute_not_col_0_mask() -> Bitboard {
    let mut words = [0u64; 10];
    let mut y = 0;
    while y < HEIGHT {
        let mut x = 1;
        while x < WIDTH {
            let idx = y * WIDTH + x;
            let w = idx / 64;
            let b = idx % 64;
            words[w] |= 1u64 << b;
            x += 1;
        }
        y += 1;
    }
    Bitboard(words)
}

const fn compute_not_last_col_mask() -> Bitboard {
    let mut words = [0u64; 10];
    let mut y = 0;
    while y < HEIGHT {
        let mut x = 0;
        while x < WIDTH - 1 {
            let idx = y * WIDTH + x;
            let w = idx / 64;
            let b = idx % 64;
            words[w] |= 1u64 << b;
            x += 1;
        }
        y += 1;
    }
    Bitboard(words)
}

impl Bitboard {
    #[inline(always)]
    pub const fn new() -> Self {
        Bitboard([0u64; 10])
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= WIDTH || y >= HEIGHT {
            return false;
        }
        let idx = y * WIDTH + x;
        let w = idx / 64;
        let b = idx % 64;
        (self.0[w] & (1u64 << b)) != 0
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize) {
        if x < WIDTH && y < HEIGHT {
            let idx = y * WIDTH + x;
            self.0[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    #[inline(always)]
    pub fn clear(&mut self, x: usize, y: usize) {
        if x < WIDTH && y < HEIGHT {
            let idx = y * WIDTH + x;
            self.0[idx / 64] &= !(1u64 << (idx % 64));
        }
    }

    #[inline(always)]
    pub fn count_ones(&self) -> u16 {
        let mut count = 0u16;
        for &w in &self.0 {
            count += w.count_ones() as u16;
        }
        count
    }

    #[inline(always)]
    pub fn and(&self, other: &Bitboard) -> Bitboard {
        let mut res = [0u64; 10];
        for i in 0..10 {
            res[i] = self.0[i] & other.0[i];
        }
        Bitboard(res)
    }

    #[inline(always)]
    pub fn or(&self, other: &Bitboard) -> Bitboard {
        let mut res = [0u64; 10];
        for i in 0..10 {
            res[i] = self.0[i] | other.0[i];
        }
        Bitboard(res)
    }

    #[inline(always)]
    pub fn not_masked(&self) -> Bitboard {
        let mut res = [0u64; 10];
        for i in 0..9 {
            res[i] = !self.0[i];
        }
        res[9] = (!self.0[9]) & ((1u64 << 24) - 1);
        Bitboard(res)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        for &w in &self.0 {
            if w != 0 {
                return false;
            }
        }
        true
    }

    #[inline(always)]
    pub fn shift_left(&self) -> Bitboard {
        let masked = self.and(&NOT_COL_0_MASK);
        let mut res = [0u64; 10];
        for i in 0..9 {
            res[i] = (masked.0[i] >> 1) | (masked.0[i + 1] << 63);
        }
        res[9] = masked.0[9] >> 1;
        Bitboard(res).and(&BOARD_MASK)
    }

    #[inline(always)]
    pub fn shift_right(&self) -> Bitboard {
        let masked = self.and(&NOT_LAST_COL_MASK);
        let mut res = [0u64; 10];
        res[0] = masked.0[0] << 1;
        for i in 1..10 {
            res[i] = (masked.0[i] << 1) | (masked.0[i - 1] >> 63);
        }
        Bitboard(res).and(&BOARD_MASK)
    }

    #[inline(always)]
    pub fn shift_up(&self) -> Bitboard {
        let mut res = [0u64; 10];
        for i in 0..9 {
            res[i] = (self.0[i] >> WIDTH) | (self.0[i + 1] << (64 - WIDTH));
        }
        res[9] = self.0[9] >> WIDTH;
        Bitboard(res).and(&BOARD_MASK)
    }

    #[inline(always)]
    pub fn shift_down(&self) -> Bitboard {
        let mut res = [0u64; 10];
        res[0] = self.0[0] << WIDTH;
        for i in 1..10 {
            res[i] = (self.0[i] << WIDTH) | (self.0[i - 1] >> (64 - WIDTH));
        }
        Bitboard(res).and(&BOARD_MASK)
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(&rhs)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(&rhs)
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn not(self) -> Self::Output {
        self.not_masked()
    }
}

pub fn rect_mask(x: u8, y: u8, w: u8, h: u8) -> Bitboard {
    let mut bb = Bitboard::new();
    let x_end = (x + w) as usize;
    let y_end = (y + h) as usize;
    for r in (y as usize)..y_end {
        if r >= HEIGHT {
            break;
        }
        for c in (x as usize)..x_end {
            if c < WIDTH {
                bb.set(c, r);
            }
        }
    }
    bb
}

pub fn legal_moves(state: &GameState, dice: DiceRoll) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    let shapes = if dice.0 == dice.1 {
        vec![(dice.0, dice.1)]
    } else {
        vec![(dice.0, dice.1), (dice.1, dice.0)]
    };

    let occupied = state.my_bb | state.opp_bb;

    if state.my_bb.is_empty() {
        for &(w, h) in &shapes {
            if (w as usize) <= WIDTH && (h as usize) <= HEIGHT {
                let (x, y) = if state.turn % 2 == 0 {
                    (0u8, 0u8)
                } else {
                    ((WIDTH - w as usize) as u8, (HEIGHT - h as usize) as u8)
                };
                let r_bb = rect_mask(x, y, w, h);
                if (r_bb & occupied).is_empty() {
                    moves.push(Move { x, y, w, h });
                }
            }
        }
        return moves;
    }

    let dilated = state.my_bb.shift_left()
        | state.my_bb.shift_right()
        | state.my_bb.shift_up()
        | state.my_bb.shift_down();

    for &(w, h) in &shapes {
        let max_x = if WIDTH >= w as usize { WIDTH - w as usize } else { continue };
        let max_y = if HEIGHT >= h as usize { HEIGHT - h as usize } else { continue };

        for y in 0..=max_y {
            for x in 0..=max_x {
                let r_bb = rect_mask(x as u8, y as u8, w, h);
                if (r_bb & occupied).is_empty() && !(r_bb & dilated).is_empty() {
                    moves.push(Move {
                        x: x as u8,
                        y: y as u8,
                        w,
                        h,
                    });
                }
            }
        }
    }

    moves
}

pub fn has_legal_move(state: &GameState, dice: DiceRoll) -> bool {
    let occupied = state.my_bb | state.opp_bb;
    if state.my_bb.is_empty() {
        let (w, h) = (dice.0, dice.1);
        if (w as usize) <= WIDTH && (h as usize) <= HEIGHT {
            let (x, y) = if state.turn % 2 == 0 {
                (0u8, 0u8)
            } else {
                ((WIDTH - w as usize) as u8, (HEIGHT - h as usize) as u8)
            };
            if (rect_mask(x, y, w, h) & occupied).is_empty() {
                return true;
            }
        }
        if dice.0 != dice.1 {
            let (w, h) = (dice.1, dice.0);
            if (w as usize) <= WIDTH && (h as usize) <= HEIGHT {
                let (x, y) = if state.turn % 2 == 0 {
                    (0u8, 0u8)
                } else {
                    ((WIDTH - w as usize) as u8, (HEIGHT - h as usize) as u8)
                };
                if (rect_mask(x, y, w, h) & occupied).is_empty() {
                    return true;
                }
            }
        }
        return false;
    }

    let dilated = state.my_bb.shift_left()
        | state.my_bb.shift_right()
        | state.my_bb.shift_up()
        | state.my_bb.shift_down();
    if (dilated & (!occupied) & BOARD_MASK).is_empty() {
        return false;
    }

    let shapes = if dice.0 == dice.1 {
        [(dice.0, dice.1), (0, 0)]
    } else {
        [(dice.0, dice.1), (dice.1, dice.0)]
    };

    for &(w, h) in &shapes {
        if w == 0 {
            continue;
        }
        let max_x = if WIDTH >= w as usize { WIDTH - w as usize } else { continue };
        let max_y = if HEIGHT >= h as usize { HEIGHT - h as usize } else { continue };

        for y in 0..=max_y {
            for x in 0..=max_x {
                let r_bb = rect_mask(x as u8, y as u8, w, h);
                if (r_bb & occupied).is_empty() && !(r_bb & dilated).is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn apply_move(state: &GameState, mv: Move) -> Afterstate {
    let r_bb = rect_mask(mv.x, mv.y, mv.w, mv.h);
    let placed_area = r_bb.count_ones();
    let mut my_bb = state.my_bb | r_bb;
    let mut opp_bb = state.opp_bb;
    let mut my_score = state.my_score + placed_area;
    let mut opp_score = state.opp_score;

    if !my_bb.is_empty() && !opp_bb.is_empty() {
        let (cap_my, cap_opp) = crate::topology::find_closed_loops(&my_bb, &opp_bb);
        if !cap_my.is_empty() {
            my_bb = my_bb | cap_my;
            my_score += cap_my.count_ones();
        }
        if !cap_opp.is_empty() {
            opp_bb = opp_bb | cap_opp;
            opp_score += cap_opp.count_ones();
        }
    }

    Afterstate {
        my_bb,
        opp_bb,
        my_score,
        opp_score,
        turn: state.turn,
    }
}

pub fn flip_perspective(after: &Afterstate) -> GameState {
    GameState {
        my_bb: after.opp_bb,
        opp_bb: after.my_bb,
        my_score: after.opp_score,
        opp_score: after.my_score,
        turn: after.turn + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_mask_area() {
        let r = rect_mask(2, 3, 4, 5);
        assert_eq!(r.count_ones(), 20);
    }

    #[test]
    fn test_wraparound_at_x0_and_last_col() {
        let mut bb = Bitboard::new();
        bb.set(0, 5);
        let left = bb.shift_left();
        assert!(left.is_empty(), "Shift left at x=0 should not wrap to (39, 4)");

        let mut bb2 = Bitboard::new();
        bb2.set(WIDTH - 1, 5);
        let right = bb2.shift_right();
        assert!(right.is_empty(), "Shift right at x=39 should not wrap to (0, 6)");
    }

    #[test]
    fn test_apply_move_preserves_komi_and_captures_loops() {
        let p2_state = GameState {
            my_bb: Bitboard::new(),
            opp_bb: rect_mask(0, 0, 3, 4),
            my_score: 18,
            opp_score: 12,
            turn: 1,
        };
        let after = apply_move(&p2_state, Move { x: 35, y: 12, w: 5, h: 3 });
        assert_eq!(after.my_score, 18 + 15);
        assert_eq!(after.opp_score, 12);

        let opp_bb = rect_mask(39, 14, 1, 1);
        let mut my_bb2 = Bitboard::new();
        for y in 0..=3 {
            my_bb2.set(10, y);
            my_bb2.set(15, y);
        }
        for x in 10..=13 {
            my_bb2.set(x, 3);
        }
        let state2 = GameState {
            my_bb: my_bb2,
            opp_bb,
            my_score: my_bb2.count_ones(),
            opp_score: 1,
            turn: 4,
        };
        let after2 = apply_move(&state2, Move { x: 14, y: 3, w: 1, h: 1 });
        assert_eq!(after2.my_score, state2.my_score + 1 + 12);
    }
}
