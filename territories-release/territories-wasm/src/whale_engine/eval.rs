use super::{HeuristicWeights, WhaleBoard, WhaleMove, WhalePlayer};
use crate::bitboard::{legal_moves, rect_mask, BOARD_MASK};
use crate::nnue::{NnueAccumulator, NnueModel};
use crate::types::{Bitboard, DiceRoll, HEIGHT, NUM_CELLS, WIDTH, WIN_SCORE};

pub fn default_spatial_matrix() -> [f64; NUM_CELLS] {
    let mut matrix = [0.0; NUM_CELLS];
    for r in 0..HEIGHT {
        for c in 0..WIDTH {
            let idx = r * WIDTH + c;
            let mut weight = 10.0;
            let dist_center_col = ((c as f64) - 19.5).abs();
            let dist_center_row = ((r as f64) - 7.0).abs();
            if dist_center_col <= 3.0 {
                weight += (4.0 - dist_center_col) * 8.0;
            }
            if dist_center_row <= 3.0 {
                weight += (4.0 - dist_center_row) * 4.0;
            }
            if r == 0 || r == HEIGHT - 1 {
                weight += 6.0;
            }
            if c == 10 || c == 20 || c == 30 {
                weight += 5.0;
            }
            matrix[idx] = weight;
        }
    }
    matrix
}

struct VoronoiField {
    p1_territory: i32,
    p2_territory: i32,
    p1_safe_territory: i32,
    p2_safe_territory: i32,
    spatial_score: f64,
}

fn compute_voronoi_influence(board: &WhaleBoard, matrix: &[f64; NUM_CELLS]) -> VoronoiField {
    let mut dist1 = [9999i16; NUM_CELLS];
    let mut dist2 = [9999i16; NUM_CELLS];
    let mut q1 = [0u16; NUM_CELLS];
    let mut q2 = [0u16; NUM_CELLS];
    let mut head1 = 0usize;
    let mut tail1 = 0usize;
    let mut head2 = 0usize;
    let mut tail2 = 0usize;

    for word_idx in 0..10 {
        let mut w1 = board.p1_bb.0[word_idx];
        while w1 != 0 {
            let bit = w1.trailing_zeros() as usize;
            let idx = word_idx * 64 + bit;
            if idx < NUM_CELLS {
                dist1[idx] = 0;
                q1[tail1] = idx as u16;
                tail1 += 1;
            }
            w1 &= w1 - 1;
        }
        let mut w2 = board.p2_bb.0[word_idx];
        while w2 != 0 {
            let bit = w2.trailing_zeros() as usize;
            let idx = word_idx * 64 + bit;
            if idx < NUM_CELLS {
                dist2[idx] = 0;
                q2[tail2] = idx as u16;
                tail2 += 1;
            }
            w2 &= w2 - 1;
        }
    }

    while head1 < tail1 {
        let curr = q1[head1] as usize;
        head1 += 1;
        let d = dist1[curr] + 1;
        let cr = curr / WIDTH;
        let cc = curr % WIDTH;

        if cr > 0 {
            let n = curr - WIDTH;
            if dist2[n] != 0 && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cr + 1 < HEIGHT {
            let n = curr + WIDTH;
            if dist2[n] != 0 && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cc > 0 {
            let n = curr - 1;
            if dist2[n] != 0 && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
        if cc + 1 < WIDTH {
            let n = curr + 1;
            if dist2[n] != 0 && dist1[n] > d {
                dist1[n] = d;
                q1[tail1] = n as u16;
                tail1 += 1;
            }
        }
    }

    while head2 < tail2 {
        let curr = q2[head2] as usize;
        head2 += 1;
        let d = dist2[curr] + 1;
        let cr = curr / WIDTH;
        let cc = curr % WIDTH;

        if cr > 0 {
            let n = curr - WIDTH;
            if dist1[n] != 0 && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cr + 1 < HEIGHT {
            let n = curr + WIDTH;
            if dist1[n] != 0 && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cc > 0 {
            let n = curr - 1;
            if dist1[n] != 0 && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
        if cc + 1 < WIDTH {
            let n = curr + 1;
            if dist1[n] != 0 && dist2[n] > d {
                dist2[n] = d;
                q2[tail2] = n as u16;
                tail2 += 1;
            }
        }
    }

    let mut p1_territory = 0;
    let mut p2_territory = 0;
    let mut p1_safe_territory = 0;
    let mut p2_safe_territory = 0;
    let mut spatial_score = 0.0;

    for i in 0..NUM_CELLS {
        let d1 = dist1[i];
        let d2 = dist2[i];
        if d1 != 0 && d2 != 0 {
            let cell_value = matrix[i];
            if d2 < d1 {
                p2_territory += 1;
                spatial_score += cell_value;
                if d1 >= 9999 {
                    p2_safe_territory += 1;
                    spatial_score += cell_value * 2.0;
                }
            } else if d1 < d2 {
                p1_territory += 1;
                spatial_score -= cell_value;
                if d2 >= 9999 {
                    p1_safe_territory += 1;
                    spatial_score -= cell_value * 2.0;
                }
            }
        }
    }

    VoronoiField {
        p1_territory,
        p2_territory,
        p1_safe_territory,
        p2_safe_territory,
        spatial_score,
    }
}

#[inline]
pub(super) fn can_close_loop(board: &WhaleBoard, mv: WhaleMove, target: WhalePlayer) -> bool {
    let occupied = board.p1_bb | board.p2_bb;
    let empty = (!occupied) & BOARD_MASK;
    let r_bb = rect_mask(mv.c, mv.r, mv.w, mv.h);
    let dil = r_bb.shift_left() | r_bb.shift_right() | r_bb.shift_up() | r_bb.shift_down();

    if (dil & empty).is_empty() {
        return false;
    }

    let target_bb = *board.bb(target);
    let friendly_neighbors = (dil & target_bb).count_ones();
    let touches_boundary = mv.r == 0
        || (mv.r as usize + mv.h as usize) >= HEIGHT
        || mv.c == 0
        || (mv.c as usize + mv.w as usize) >= WIDTH;

    (touches_boundary && friendly_neighbors >= 1) || friendly_neighbors >= 2
}

pub(super) fn generate_legal_moves_for_board(
    board: &WhaleBoard,
    player: WhalePlayer,
    d1: u8,
    d2: u8,
) -> Vec<WhaleMove> {
    let gs = board.to_game_state(player);
    legal_moves(&gs, DiceRoll(d1, d2))
        .into_iter()
        .map(WhaleMove::from_move)
        .collect()
}

pub fn evaluate_board_state(
    board: &WhaleBoard,
    player: WhalePlayer,
    weights: &HeuristicWeights,
    matrix: &[f64; NUM_CELLS],
    nnue: &NnueModel,
) -> f64 {
    let p_score = board.score(player) as f64;
    let opp_score = board.score(player.opponent()) as f64;

    if p_score >= WIN_SCORE as f64 {
        return 50_000.0;
    }
    if opp_score >= WIN_SCORE as f64 {
        return -50_000.0;
    }

    let score_diff = p_score - opp_score;
    let field = compute_voronoi_influence(board, matrix);

    let (voronoi_delta, safe_delta, spatial) = match player {
        WhalePlayer::Player2 => (
            ((field.p2_territory - field.p1_territory) as f64) * weights.weight_voronoi_delta,
            ((field.p2_safe_territory - field.p1_safe_territory) as f64)
                * weights.weight_safe_territory,
            field.spatial_score * weights.weight_spatial_matrix,
        ),
        WhalePlayer::Player1 => (
            ((field.p1_territory - field.p2_territory) as f64) * weights.weight_voronoi_delta,
            ((field.p1_safe_territory - field.p2_safe_territory) as f64)
                * weights.weight_safe_territory,
            -field.spatial_score * weights.weight_spatial_matrix,
        ),
    };

    let hce = (score_diff * weights.weight_area * 10.0) + voronoi_delta + safe_delta + spatial;

    let mut acc = NnueAccumulator::new(nnue);
    acc.update_bitboard(nnue, board.bb(player), true);
    acc.update_bitboard(nnue, board.bb(player.opponent()), false);
    let nnue_score = nnue.forward(&acc) * 100.0;

    0.85 * nnue_score + 0.15 * hce
}

fn has_sufficient_pieces(board: &WhaleBoard) -> bool {
    let p1_occ = board.p1_bb.count_ones();
    let p2_occ = board.p2_bb.count_ones();
    p1_occ >= 20 && p2_occ >= 20 && (p1_occ + p2_occ) < 520
}

fn max_occupied_col(bb: &Bitboard) -> usize {
    let mut max_col = 0usize;
    for word_idx in 0..10 {
        let mut w = bb.0[word_idx];
        while w != 0 {
            let idx = word_idx * 64 + w.trailing_zeros() as usize;
            if idx < NUM_CELLS {
                let c = idx % WIDTH;
                if c > max_col {
                    max_col = c;
                }
            }
            w &= w - 1;
        }
    }
    max_col
}

fn min_occupied_col(bb: &Bitboard) -> usize {
    let mut min_col = WIDTH;
    for word_idx in 0..10 {
        let mut w = bb.0[word_idx];
        while w != 0 {
            let idx = word_idx * 64 + w.trailing_zeros() as usize;
            if idx < NUM_CELLS {
                let c = idx % WIDTH;
                if c < min_col {
                    min_col = c;
                }
            }
            w &= w - 1;
        }
    }
    min_col
}

fn has_moderate_contact(board: &WhaleBoard) -> bool {
    let p1_max_col = max_occupied_col(&board.p1_bb);
    let p2_min_col = min_occupied_col(&board.p2_bb);
    p2_min_col <= p1_max_col + 5
}

pub(super) fn has_sufficient_pieces_and_moderate_contact(board: &WhaleBoard) -> bool {
    has_sufficient_pieces(board) && has_moderate_contact(board)
}
