use crate::types::{
    Afterstate, Bitboard, DiceRoll, GameState, TopoFeatures, DICE_OUTCOMES, HEIGHT, NUM_CELLS,
    TENSOR_SIZE, WIDTH,
};

pub const ROW_MASK: u64 = (1u64 << WIDTH) - 1;
pub const BORDER_MASK: Bitboard = compute_border_mask();

const fn compute_border_mask() -> Bitboard {
    let mut words = [0u64; 10];
    let mut y = 0;
    while y < HEIGHT {
        let mut x = 0;
        while x < WIDTH {
            if x == 0 || x == WIDTH - 1 || y == 0 || y == HEIGHT - 1 {
                let idx = y * WIDTH + x;
                words[idx / 64] |= 1u64 << (idx % 64);
            }
            x += 1;
        }
        y += 1;
    }
    Bitboard(words)
}

#[inline(always)]
pub fn bitboard_to_rows(bb: &Bitboard) -> [u64; HEIGHT] {
    let mut rows = [0u64; HEIGHT];
    let carry_thresh = 64 - WIDTH;
    for y in 0..HEIGHT {
        let start = y * WIDTH;
        let w = start / 64;
        let s = start % 64;
        let lo = bb.0[w] >> s;
        let hi = if s > carry_thresh {
            bb.0[w + 1] << (64 - s)
        } else {
            0
        };
        rows[y] = (lo | hi) & ROW_MASK;
    }
    rows
}

#[inline(always)]
pub fn rows_to_bitboard(rows: &[u64; HEIGHT]) -> Bitboard {
    let mut words = [0u64; 10];
    let carry_thresh = 64 - WIDTH;
    for y in 0..HEIGHT {
        let r = rows[y] & ROW_MASK;
        if r == 0 {
            continue;
        }
        let start = y * WIDTH;
        let w = start / 64;
        let s = start % 64;
        words[w] |= r << s;
        if s > carry_thresh {
            words[w + 1] |= r >> (64 - s);
        }
    }
    Bitboard(words)
}

#[inline(always)]
pub fn dilate_rows(rows: &[u64; HEIGHT]) -> [u64; HEIGHT] {
    let mut out = [0u64; HEIGHT];
    for y in 0..HEIGHT {
        let r = rows[y];
        let mut d = r | (r >> 1) | ((r << 1) & ROW_MASK);
        if y > 0 {
            d |= rows[y - 1];
        }
        if y + 1 < HEIGHT {
            d |= rows[y + 1];
        }
        out[y] = d;
    }
    out
}

#[inline(always)]
pub fn horizontal_span(row_mask: u64, x: u32) -> u64 {
    let hi = (!row_mask & (!0u64 << x)).trailing_zeros();
    let below = !row_mask & ((1u64 << x) - 1);
    let lo = 64 - below.leading_zeros();
    (1u64 << hi) - (1u64 << lo)
}

#[inline(always)]
fn scanline_flood_fill<F: FnMut(usize, u64)>(
    remaining: &mut [u64; HEIGHT],
    stack_r: &mut [u8; 320],
    stack_s: &mut [u64; 320],
    mut stack_len: usize,
    mut on_span: F,
) {
    while stack_len > 0 {
        stack_len -= 1;
        let r = stack_r[stack_len] as usize;
        let cur_span = stack_s[stack_len];

        if r > 0 {
            let adj_r = r - 1;
            let mut candidates = remaining[adj_r] & cur_span;
            while candidates != 0 {
                let cx = candidates.trailing_zeros();
                let adj_span = horizontal_span(remaining[adj_r], cx);
                remaining[adj_r] &= !adj_span;
                on_span(adj_r, adj_span);
                stack_r[stack_len] = adj_r as u8;
                stack_s[stack_len] = adj_span;
                stack_len += 1;
                candidates &= !adj_span;
            }
        }
        if r + 1 < HEIGHT {
            let adj_r = r + 1;
            let mut candidates = remaining[adj_r] & cur_span;
            while candidates != 0 {
                let cx = candidates.trailing_zeros();
                let adj_span = horizontal_span(remaining[adj_r], cx);
                remaining[adj_r] &= !adj_span;
                on_span(adj_r, adj_span);
                stack_r[stack_len] = adj_r as u8;
                stack_s[stack_len] = adj_span;
                stack_len += 1;
                candidates &= !adj_span;
            }
        }
    }
}

pub fn find_closed_loops(my_bb: &Bitboard, opp_bb: &Bitboard) -> (Bitboard, Bitboard) {
    if my_bb.is_empty() || opp_bb.is_empty() {
        return (Bitboard::new(), Bitboard::new());
    }

    let my_rows = bitboard_to_rows(my_bb);
    let opp_rows = bitboard_to_rows(opp_bb);
    let mut empty_rows = [0u64; HEIGHT];
    for y in 0..HEIGHT {
        empty_rows[y] = !(my_rows[y] | opp_rows[y]) & ROW_MASK;
    }

    let my_dilated = dilate_rows(&my_rows);
    let opp_dilated = dilate_rows(&opp_rows);

    let mut cap_my_rows = [0u64; HEIGHT];
    let mut cap_opp_rows = [0u64; HEIGHT];

    let mut stack_r = [0u8; 320];
    let mut stack_s = [0u64; 320];
    let mut comp_remaining = empty_rows;

    for start_y in 0..HEIGHT {
        while comp_remaining[start_y] != 0 {
            let x = comp_remaining[start_y].trailing_zeros();
            let span = horizontal_span(comp_remaining[start_y], x);
            comp_remaining[start_y] &= !span;

            stack_r[0] = start_y as u8;
            stack_s[0] = span;

            let mut comp = [0u64; HEIGHT];
            comp[start_y] = span;

            scanline_flood_fill(&mut comp_remaining, &mut stack_r, &mut stack_s, 1, |adj_r, adj_span| {
                comp[adj_r] |= adj_span;
            });

            let mut touches_my = false;
            let mut touches_opp = false;
            for y in 0..HEIGHT {
                if (comp[y] & my_dilated[y]) != 0 {
                    touches_my = true;
                }
                if (comp[y] & opp_dilated[y]) != 0 {
                    touches_opp = true;
                }
            }

            if touches_my && !touches_opp {
                for y in 0..HEIGHT {
                    cap_my_rows[y] |= comp[y];
                }
            } else if touches_opp && !touches_my {
                for y in 0..HEIGHT {
                    cap_opp_rows[y] |= comp[y];
                }
            }
        }
    }

    (rows_to_bitboard(&cap_my_rows), rows_to_bitboard(&cap_opp_rows))
}

pub fn extract_features(state: &Afterstate) -> TopoFeatures {
    let my_rows = bitboard_to_rows(&state.my_bb);
    let opp_rows = bitboard_to_rows(&state.opp_bb);

    let mut empty_rows = [0u64; HEIGHT];
    for y in 0..HEIGHT {
        empty_rows[y] = !(my_rows[y] | opp_rows[y]) & ROW_MASK;
    }

    let my_dilated = dilate_rows(&my_rows);
    let opp_dilated = dilate_rows(&opp_rows);

    let mut stack_r = [0u8; 320];
    let mut stack_s = [0u64; 320];
    let mut stack_len = 0usize;

    let mut remaining = empty_rows;

    while remaining[0] != 0 {
        let x = remaining[0].trailing_zeros();
        let span = horizontal_span(remaining[0], x);
        remaining[0] &= !span;
        stack_r[stack_len] = 0;
        stack_s[stack_len] = span;
        stack_len += 1;
    }

    let last_r = HEIGHT - 1;
    while remaining[last_r] != 0 {
        let x = remaining[last_r].trailing_zeros();
        let span = horizontal_span(remaining[last_r], x);
        remaining[last_r] &= !span;
        stack_r[stack_len] = last_r as u8;
        stack_s[stack_len] = span;
        stack_len += 1;
    }

    let right_bit = 1u64 << (WIDTH - 1);
    for y in 1..last_r {
        if (remaining[y] & 1) != 0 {
            let span = horizontal_span(remaining[y], 0);
            remaining[y] &= !span;
            stack_r[stack_len] = y as u8;
            stack_s[stack_len] = span;
            stack_len += 1;
        }
        if (remaining[y] & right_bit) != 0 {
            let span = horizontal_span(remaining[y], (WIDTH - 1) as u32);
            remaining[y] &= !span;
            stack_r[stack_len] = y as u8;
            stack_s[stack_len] = span;
            stack_len += 1;
        }
    }

    scanline_flood_fill(&mut remaining, &mut stack_r, &mut stack_s, stack_len, |_, _| {});

    let cauldrons_rows = remaining;
    let mut cauldron_size_map = [0.0f32; NUM_CELLS];
    let mut max_my_cauldron = 0u16;
    let mut max_opp_cauldron = 0u16;

    let mut comp_remaining = cauldrons_rows;
    for start_y in 0..HEIGHT {
        while comp_remaining[start_y] != 0 {
            let x = comp_remaining[start_y].trailing_zeros();
            let span = horizontal_span(comp_remaining[start_y], x);
            comp_remaining[start_y] &= !span;

            stack_r[0] = start_y as u8;
            stack_s[0] = span;

            let mut comp = [0u64; HEIGHT];
            comp[start_y] = span;
            let mut size = span.count_ones() as u16;

            scanline_flood_fill(&mut comp_remaining, &mut stack_r, &mut stack_s, 1, |adj_r, adj_span| {
                comp[adj_r] |= adj_span;
                size += adj_span.count_ones() as u16;
            });

            let mut touches_my = false;
            let mut touches_opp = false;
            for y in 0..HEIGHT {
                if (comp[y] & my_dilated[y]) != 0 {
                    touches_my = true;
                }
                if (comp[y] & opp_dilated[y]) != 0 {
                    touches_opp = true;
                }
            }

            if touches_my && size > max_my_cauldron {
                max_my_cauldron = size;
            }
            if touches_opp && size > max_opp_cauldron {
                max_opp_cauldron = size;
            }

            let norm_size = (size as f32) / 300.0;
            for y in 0..HEIGHT {
                let mut w = comp[y];
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    cauldron_size_map[y * WIDTH + bit] = norm_size;
                    w &= w - 1;
                }
            }
        }
    }

    let cauldrons_dilated = dilate_rows(&cauldrons_rows);
    let mut p_seal_my = [0.0f32; NUM_CELLS];
    let mut p_seal_opp = [0.0f32; NUM_CELLS];

    for y in 0..HEIGHT {
        let mut w_my = cauldrons_dilated[y] & my_dilated[y] & empty_rows[y];
        while w_my != 0 {
            let bit = w_my.trailing_zeros() as usize;
            p_seal_my[y * WIDTH + bit] = 0.55;
            w_my &= w_my - 1;
        }
        let mut w_opp = cauldrons_dilated[y] & opp_dilated[y] & empty_rows[y];
        while w_opp != 0 {
            let bit = w_opp.trailing_zeros() as usize;
            p_seal_opp[y * WIDTH + bit] = 0.55;
            w_opp &= w_opp - 1;
        }
    }

    let my_state = GameState {
        my_bb: state.my_bb,
        opp_bb: state.opp_bb,
        my_score: state.my_score,
        opp_score: state.opp_score,
        turn: state.turn,
    };
    let opp_state = GameState {
        my_bb: state.opp_bb,
        opp_bb: state.my_bb,
        my_score: state.opp_score,
        opp_score: state.my_score,
        turn: state.turn + 1,
    };

    let mut my_legal_count = 0;
    let mut opp_legal_count = 0;
    for &(d1, d2, _) in &DICE_OUTCOMES {
        let roll = DiceRoll(d1, d2);
        if crate::bitboard::has_legal_move(&my_state, roll) {
            my_legal_count += 1;
        }
        if crate::bitboard::has_legal_move(&opp_state, roll) {
            opp_legal_count += 1;
        }
    }

    let my_elasticity = (my_legal_count as f32) / 21.0;
    let opp_elasticity = (opp_legal_count as f32) / 21.0;

    TopoFeatures {
        cauldron_size_map,
        p_seal_my,
        p_seal_opp,
        my_elasticity,
        opp_elasticity,
        max_my_cauldron,
        max_opp_cauldron,
    }
}

pub fn encode_tensor(state: &Afterstate, feat: &TopoFeatures) -> [f32; TENSOR_SIZE] {
    let mut tensor = [0.0f32; TENSOR_SIZE];
    let plane_size = HEIGHT * WIDTH;

    let my_rows = bitboard_to_rows(&state.my_bb);
    let opp_rows = bitboard_to_rows(&state.opp_bb);
    let my_dil = dilate_rows(&my_rows);

    for y in 0..HEIGHT {
        let mut w = my_rows[y];
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            tensor[y * WIDTH + bit] = 1.0;
            w &= w - 1;
        }
    }

    let ch1_offset = 1 * plane_size;
    for y in 0..HEIGHT {
        let mut w = opp_rows[y];
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            tensor[ch1_offset + y * WIDTH + bit] = 1.0;
            w &= w - 1;
        }
    }

    let ch2_offset = 2 * plane_size;
    for y in 0..HEIGHT {
        let empty_r = !(my_rows[y] | opp_rows[y]) & ROW_MASK;
        let mut w = my_dil[y] & empty_r;
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            tensor[ch2_offset + y * WIDTH + bit] = 1.0;
            w &= w - 1;
        }
    }

    tensor[3 * plane_size..4 * plane_size].copy_from_slice(&feat.cauldron_size_map);

    tensor[4 * plane_size..5 * plane_size].copy_from_slice(&feat.p_seal_my);

    tensor[5 * plane_size..6 * plane_size].copy_from_slice(&feat.p_seal_opp);

    let score_plane = (state.my_score as f32) / 300.0;
    tensor[6 * plane_size..7 * plane_size].fill(score_plane);

    let elasticity_plane = feat.my_elasticity;
    tensor[7 * plane_size..8 * plane_size].fill(elasticity_plane);

    tensor
}

#[cfg(test)]
#[path = "topology_tests.rs"]
mod tests;
