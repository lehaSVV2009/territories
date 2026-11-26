use std::collections::HashSet;

use crate::bitboard::{apply_move, flip_perspective, legal_moves};
use crate::topology::{
    bitboard_to_rows, dilate_rows, encode_tensor, extract_features, find_closed_loops,
};
use crate::types::{
    Afterstate, DiceRoll, Evaluator, GameState, Move, TopoFeatures, DICE_OUTCOMES, HEIGHT,
    REP_DICE_BUCKETS, TENSOR_SIZE, WIDTH, WIN_SCORE,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicCandidatePolicy;

#[derive(Clone, Debug)]
pub struct CandidateBreakdown {
    pub mv: Move,
    pub heuristic_score: i32,
    pub capture_cells: u16,
    pub capture_bonus: i32,
    pub p_cnn_1: Option<f32>,
    pub p_cnn_2: Option<f32>,
    pub aux_score: Option<f64>,
    pub aux_sigmoid: Option<f32>,
    pub vetoed: bool,
    pub penalty: f32,
    pub final_score: f32,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct ArbitrationBreakdown {
    pub status: String,
    pub primary_move: Move,
    pub aux_move: Move,
    pub veto_reason: Option<String>,
    pub candidates: Vec<CandidateBreakdown>,
}

impl HeuristicCandidatePolicy {
    pub fn score_move_details(
        state: &GameState,
        m: Move,
        check_loops: bool,
        my_rows: &[u64; HEIGHT],
    ) -> (i32, u16, i32) {
        let area = (m.w as i32) * (m.h as i32);
        let center_dist = (m.x as i32 + m.w as i32 / 2 - 20).abs()
            + (m.y as i32 + m.h as i32 / 2 - 7).abs();
        let mut capture_cells = 0u16;
        let mut capture_bonus = 0i32;

        if check_loops {
            let touches_border = m.x == 0
                || m.y == 0
                || (m.x as usize + m.w as usize) >= WIDTH
                || (m.y as usize + m.h as usize) >= HEIGHT;
            let r_bb = crate::bitboard::rect_mask(m.x, m.y, m.w, m.h);
            let r_rows = bitboard_to_rows(&r_bb);
            let r_dil = dilate_rows(&r_rows);
            let mut adj_cells = 0u32;
            for y in 0..HEIGHT {
                adj_cells += (r_dil[y] & my_rows[y]).count_ones();
            }
            if (touches_border && adj_cells >= 1) || adj_cells >= 2 {
                let new_my = state.my_bb | r_bb;
                let (cap_my, _) = find_closed_loops(&new_my, &state.opp_bb);
                capture_cells = cap_my.count_ones();
                capture_bonus = (capture_cells as i32) * 40;
            }
        }

        (area * 10 + capture_bonus - center_dist, capture_cells, capture_bonus)
    }

    pub fn score_move(
        state: &GameState,
        m: Move,
        check_loops: bool,
        my_rows: &[u64; HEIGHT],
    ) -> i32 {
        Self::score_move_details(state, m, check_loops, my_rows).0
    }

    pub fn policy_top_k_with_scores(
        state: &GameState,
        moves: &[Move],
        k: usize,
    ) -> Vec<(Move, i32)> {
        let check_loops = !state.my_bb.is_empty() && !state.opp_bb.is_empty();
        let my_rows = if check_loops {
            bitboard_to_rows(&state.my_bb)
        } else {
            [0u64; HEIGHT]
        };

        let mut scored: Vec<(Move, i32)> = moves
            .iter()
            .map(|&m| (m, Self::score_move(state, m, check_loops, &my_rows)))
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(k);
        scored
    }

    pub fn policy_top_k(state: &GameState, moves: &[Move], k: usize) -> Vec<Move> {
        if moves.len() <= k {
            return moves.to_vec();
        }
        Self::policy_top_k_with_scores(state, moves, k)
            .into_iter()
            .map(|(m, _)| m)
            .collect()
    }
}

pub struct PreparedBotBatch {
    pub candidates: Vec<Move>,
    pub immediate_win_move: Option<Move>,
    pub leaves: Vec<(Afterstate, TopoFeatures)>,
}

pub fn prepare_cnn_1ply_batch(state: &GameState, dice: DiceRoll, beam_k: usize) -> PreparedBotBatch {
    let moves = legal_moves(state, dice);
    if moves.is_empty() {
        return PreparedBotBatch {
            candidates: Vec::new(),
            immediate_win_move: None,
            leaves: Vec::new(),
        };
    }

    let candidates = HeuristicCandidatePolicy::policy_top_k(state, &moves, beam_k);
    for &m in &candidates {
        let after = apply_move(state, m);
        if after.my_score >= WIN_SCORE {
            return PreparedBotBatch {
                candidates,
                immediate_win_move: Some(m),
                leaves: Vec::new(),
            };
        }
    }

    let leaves: Vec<(Afterstate, TopoFeatures)> = candidates
        .iter()
        .map(|&m| {
            let after = apply_move(state, m);
            let feat = extract_features(&after);
            (after, feat)
        })
        .collect();

    PreparedBotBatch {
        candidates,
        immediate_win_move: None,
        leaves,
    }
}

#[derive(Clone, Debug)]
pub enum LeafTarget {
    Terminal(f32),
    TensorIdx { idx: usize, is_my_afterstate: bool },
}

#[derive(Clone, Debug)]
pub struct BucketPlan {
    pub weight: f32,
    pub opp_replies: Vec<LeafTarget>,
}

#[derive(Clone, Debug)]
pub struct ExpectimaxCandidatePlan {
    pub mv: Move,
    pub buckets: Vec<BucketPlan>,
}

pub struct ExpectimaxPlan {
    pub candidates: Vec<ExpectimaxCandidatePlan>,
    pub flat_tensor: Vec<f32>,
    pub batch_size: usize,
    pub leaf_states: Vec<(Afterstate, TopoFeatures)>,
}

pub fn prepare_2ply_expectimax_leaves(
    state: &GameState,
    pre_pruned: &[Move],
    opp_top_k: usize,
) -> ExpectimaxPlan {
    prepare_2ply_expectimax_leaves_impl(state, pre_pruned, opp_top_k, false)
}

fn prepare_2ply_expectimax_leaves_impl(
    state: &GameState,
    pre_pruned: &[Move],
    opp_top_k: usize,
    retain_states: bool,
) -> ExpectimaxPlan {
    let mut candidate_plans = Vec::with_capacity(pre_pruned.len());
    let mut flat_tensor = Vec::with_capacity(pre_pruned.len() * REP_DICE_BUCKETS.len() * opp_top_k * TENSOR_SIZE);
    let mut leaf_states = if retain_states {
        Vec::with_capacity(pre_pruned.len() * REP_DICE_BUCKETS.len() * opp_top_k)
    } else {
        Vec::new()
    };
    let mut batch_size = 0usize;

    for &mv in pre_pruned {
        let after_my = apply_move(state, mv);
        let mut my_afterstate_leaf_idx: Option<usize> = None;

        if after_my.my_score >= WIN_SCORE {
            let buckets = REP_DICE_BUCKETS
                .iter()
                .map(|&(_, _, weight)| BucketPlan {
                    weight,
                    opp_replies: vec![LeafTarget::Terminal(1.0)],
                })
                .collect();
            candidate_plans.push(ExpectimaxCandidatePlan { mv, buckets });
            continue;
        }

        let opp_state = flip_perspective(&after_my);
        let mut buckets = Vec::with_capacity(REP_DICE_BUCKETS.len());

        for &(d1, d2, weight) in &REP_DICE_BUCKETS {
            let opp_moves = legal_moves(&opp_state, DiceRoll(d1, d2));
            if opp_moves.is_empty() {
                let idx = match my_afterstate_leaf_idx {
                    Some(existing) => existing,
                    None => {
                        let feat_my = extract_features(&after_my);
                        flat_tensor.extend_from_slice(&encode_tensor(&after_my, &feat_my));
                        if retain_states {
                            leaf_states.push((after_my, feat_my));
                        }
                        let new_idx = batch_size;
                        batch_size += 1;
                        my_afterstate_leaf_idx = Some(new_idx);
                        new_idx
                    }
                };
                buckets.push(BucketPlan {
                    weight,
                    opp_replies: vec![LeafTarget::TensorIdx {
                        idx,
                        is_my_afterstate: true,
                    }],
                });
                continue;
            }

            let top_opp = HeuristicCandidatePolicy::policy_top_k(&opp_state, &opp_moves, opp_top_k);
            let mut opp_replies = Vec::with_capacity(top_opp.len());
            for opp_mv in top_opp {
                let after_opp = apply_move(&opp_state, opp_mv);
                if after_opp.my_score >= WIN_SCORE {
                    opp_replies.push(LeafTarget::Terminal(0.0));
                } else if after_opp.opp_score >= WIN_SCORE {
                    opp_replies.push(LeafTarget::Terminal(1.0));
                } else {
                    let feat_opp = extract_features(&after_opp);
                    flat_tensor.extend_from_slice(&encode_tensor(&after_opp, &feat_opp));
                    if retain_states {
                        leaf_states.push((after_opp, feat_opp));
                    }
                    let idx = batch_size;
                    batch_size += 1;
                    opp_replies.push(LeafTarget::TensorIdx {
                        idx,
                        is_my_afterstate: false,
                    });
                }
            }
            buckets.push(BucketPlan {
                weight,
                opp_replies,
            });
        }

        candidate_plans.push(ExpectimaxCandidatePlan { mv, buckets });
    }

    ExpectimaxPlan {
        candidates: candidate_plans,
        flat_tensor,
        batch_size,
        leaf_states,
    }
}

pub fn score_2ply_expectimax_plan(
    plan: &ExpectimaxPlan,
    tensor_scores: &[f32],
) -> Vec<(Move, f32)> {
    let mut ranked = Vec::with_capacity(plan.candidates.len());

    for cand in &plan.candidates {
        let mut expected_win = 0.0f32;
        for bucket in &cand.buckets {
            let mut min_my_win = f32::INFINITY;
            for target in &bucket.opp_replies {
                let my_win = match *target {
                    LeafTarget::Terminal(val) => val,
                    LeafTarget::TensorIdx {
                        idx,
                        is_my_afterstate,
                    } => {
                        let p = tensor_scores[idx].clamp(0.0, 1.0);
                        if is_my_afterstate {
                            p
                        } else {
                            1.0 - p
                        }
                    }
                };
                if my_win < min_my_win {
                    min_my_win = my_win;
                }
            }
            if min_my_win.is_finite() {
                expected_win += bucket.weight * min_my_win;
            }
        }
        ranked.push((cand.mv, expected_win.clamp(0.0, 1.0)));
    }

    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

pub fn evaluate_2ply_expectimax_with_evaluator<E: Evaluator>(
    state: &GameState,
    dice: DiceRoll,
    beam_k: usize,
    pre_prune_k: usize,
    opp_top_k: usize,
    evaluator: &E,
) -> Option<(Move, Vec<(Move, f32)>)> {
    let batch = prepare_cnn_1ply_batch(state, dice, beam_k);
    if batch.candidates.is_empty() {
        return None;
    }
    if let Some(win_mv) = batch.immediate_win_move {
        return Some((win_mv, vec![(win_mv, 1.0)]));
    }

    let pre_pruned: Vec<Move> = if batch.candidates.len() > pre_prune_k {
        let scores = evaluator.evaluate_batch(&batch.leaves);
        let mut pairs: Vec<(Move, f32)> = batch.candidates.into_iter().zip(scores).collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(pre_prune_k).map(|(m, _)| m).collect()
    } else {
        batch.candidates
    };

    let plan = prepare_2ply_expectimax_leaves_impl(state, &pre_pruned, opp_top_k, true);
    let tensor_scores = if plan.batch_size > 0 {
        evaluator.evaluate_batch(&plan.leaf_states)
    } else {
        Vec::new()
    };
    let ranked = score_2ply_expectimax_plan(&plan, &tensor_scores);
    let top_mv = ranked.first().map(|&(mv, _)| mv);
    top_mv.map(|mv| (mv, ranked))
}

pub fn tactical_safety_penalty(state: &GameState, mv: Move) -> (bool, f32) {
    let after_my = apply_move(state, mv);
    if after_my.my_score >= WIN_SCORE {
        return (false, 0.0);
    }

    let opp_state = flip_perspective(&after_my);
    let check_loops = !opp_state.my_bb.is_empty() && !opp_state.opp_bb.is_empty();
    let opp_rows = if check_loops {
        bitboard_to_rows(&opp_state.my_bb)
    } else {
        [0u64; HEIGHT]
    };

    let mut opp_win_prob = 0.0f32;
    let mut max_opp_cap = 0u16;

    for &(d1, d2, prob) in &DICE_OUTCOMES {
        let max_area = (d1 as u16) * (d2 as u16);
        let is_rep = matches!(
            (d1, d2),
            (1, 1) | (1, 3) | (2, 4) | (3, 4) | (4, 5) | (5, 6) | (6, 6)
        );
        let can_area_win = opp_state.my_score + max_area >= WIN_SCORE;

        if !is_rep && !can_area_win && opp_state.my_score + max_area + 40 < WIN_SCORE {
            continue;
        }

        let opp_moves = legal_moves(&opp_state, DiceRoll(d1, d2));
        if opp_moves.is_empty() {
            continue;
        }

        let mut roll_wins = false;
        for om in opp_moves {
            let area = (om.w as u16) * (om.h as u16);
            if opp_state.my_score + area >= WIN_SCORE {
                roll_wins = true;
                break;
            }
            if check_loops {
                let touches_border = om.x == 0
                    || om.y == 0
                    || (om.x as usize + om.w as usize) >= WIDTH
                    || (om.y as usize + om.h as usize) >= HEIGHT;
                let r_bb = crate::bitboard::rect_mask(om.x, om.y, om.w, om.h);
                let r_rows = bitboard_to_rows(&r_bb);
                let r_dil = dilate_rows(&r_rows);
                let mut adj = 0u32;
                for y in 0..HEIGHT {
                    adj += (r_dil[y] & opp_rows[y]).count_ones();
                }
                if (touches_border && adj >= 1) || adj >= 2 {
                    let new_opp = opp_state.my_bb | r_bb;
                    let (cap_opp, _) = find_closed_loops(&new_opp, &opp_state.opp_bb);
                    let cap_cells = cap_opp.count_ones();
                    if cap_cells > max_opp_cap {
                        max_opp_cap = cap_cells;
                    }
                    if opp_state.my_score + area + cap_cells >= WIN_SCORE {
                        roll_wins = true;
                        break;
                    }
                }
            }
        }
        if roll_wins {
            opp_win_prob += prob;
        }
    }

    if opp_win_prob > 0.0 {
        (true, 10.0 + opp_win_prob * 5.0)
    } else if max_opp_cap >= 15 {
        (false, 0.25 * ((max_opp_cap as f32) / 60.0).min(1.0))
    } else {
        (false, 0.0)
    }
}

pub fn xhigh_arbitrate_detailed(
    state: &GameState,
    cnn_1ply: &[(Move, f32)],
    cnn_ranked: &[(Move, f32)],
    whale_best: (Move, f64),
    whale_evals: &[(Move, f64)],
) -> (Move, ArbitrationBreakdown) {
    let cnn_1 = cnn_ranked.first().map(|&(m, _)| m).unwrap_or(whale_best.0);

    let mut union_moves: Vec<Move> = Vec::with_capacity(6);
    let mut seen: HashSet<Move> = HashSet::with_capacity(6);
    for &(mv, _) in cnn_ranked.iter().take(4) {
        if seen.insert(mv) {
            union_moves.push(mv);
        }
    }
    if seen.insert(whale_best.0) {
        union_moves.push(whale_best.0);
    }

    let check_loops = !state.my_bb.is_empty() && !state.opp_bb.is_empty();
    let my_rows = if check_loops {
        bitboard_to_rows(&state.my_bb)
    } else {
        [0u64; HEIGHT]
    };

    let mut s_min = f64::INFINITY;
    let mut s_max = f64::NEG_INFINITY;
    let mut s_sum = 0.0f64;
    let mut s_count = 0usize;
    let mut h_min = i32::MAX;
    let mut h_max = i32::MIN;

    struct RawEntry {
        mv: Move,
        h_raw: i32,
        cap_cells: u16,
        cap_bonus: i32,
        p1: Option<f32>,
        p2: f32,
        s_whale: Option<f64>,
        allows_win: bool,
        penalty: f32,
    }

    let is_consensus = cnn_1 == whale_best.0;
    let mut raw_entries: Vec<RawEntry> = Vec::with_capacity(union_moves.len());

    for &mv in &union_moves {
        let p1 = cnn_1ply.iter().find(|&&(m, _)| m == mv).map(|&(_, p)| p);
        let p2 = cnn_ranked
            .iter()
            .find(|&&(m, _)| m == mv)
            .map(|&(_, p)| p)
            .unwrap_or(0.5);
        let s_whale = whale_evals
            .iter()
            .find(|&&(m, _)| m == mv)
            .map(|&(_, s)| s)
            .or_else(|| if mv == whale_best.0 { Some(whale_best.1) } else { None });
        let (h_raw, cap_cells, cap_bonus) =
            HeuristicCandidatePolicy::score_move_details(state, mv, check_loops, &my_rows);
        let (allows_win, penalty) = if is_consensus {
            (false, 0.0)
        } else {
            tactical_safety_penalty(state, mv)
        };

        if let Some(sv) = s_whale {
            s_min = s_min.min(sv);
            s_max = s_max.max(sv);
            s_sum += sv;
            s_count += 1;
        }
        h_min = h_min.min(h_raw);
        h_max = h_max.max(h_raw);

        raw_entries.push(RawEntry {
            mv,
            h_raw,
            cap_cells,
            cap_bonus,
            p1,
            p2,
            s_whale,
            allows_win,
            penalty,
        });
    }

    let s_mean = if s_count > 0 { s_sum / (s_count as f64) } else { 0.0 };
    let temp = if s_count > 0 {
        400.0_f64.max((s_max - s_min) * 0.25)
    } else {
        400.0
    };
    let h_span = (h_max - h_min) as f32;

    let mut best_mv = if is_consensus { cnn_1 } else { union_moves[0] };
    let mut best_total = f32::NEG_INFINITY;
    let mut any_top_vetoed = false;
    let mut veto_reason: Option<String> = None;

    let mut breakdowns: Vec<CandidateBreakdown> = Vec::with_capacity(raw_entries.len());

    for e in raw_entries {
        let whale_sig_opt = e.s_whale.map(|sv| {
            let z = ((sv - s_mean) / temp).clamp(-20.0, 20.0);
            (1.0 / (1.0 + (-z).exp())) as f32
        });
        let whale_sig = whale_sig_opt.unwrap_or(0.5);
        let h_norm = if h_span > 0.0 {
            ((e.h_raw - h_min) as f32) / h_span
        } else {
            0.5
        };

        let total = 0.50 * e.p2 + 0.40 * whale_sig + 0.10 * h_norm - e.penalty;
        if !is_consensus && total > best_total {
            best_total = total;
            best_mv = e.mv;
        }

        if e.penalty > 0.0 {
            any_top_vetoed = true;
            if e.allows_win {
                veto_reason = Some("Immediate opponent win threat detected".to_string());
            } else if veto_reason.is_none() {
                veto_reason = Some("Large uncontested capture threat detected".to_string());
            }
        }

        breakdowns.push(CandidateBreakdown {
            mv: e.mv,
            heuristic_score: e.h_raw,
            capture_cells: e.cap_cells,
            capture_bonus: e.cap_bonus,
            p_cnn_1: e.p1,
            p_cnn_2: Some(e.p2),
            aux_score: e.s_whale,
            aux_sigmoid: whale_sig_opt,
            vetoed: e.penalty > 0.0,
            penalty: e.penalty,
            final_score: total,
            selected: false,
        });
    }

    for item in &mut breakdowns {
        item.selected = item.mv == best_mv;
    }
    breakdowns.sort_by(|a, b| {
        b.selected
            .cmp(&a.selected)
            .then_with(|| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal))
    });

    let status = if is_consensus {
        "CONSENSUS".to_string()
    } else if any_top_vetoed {
        "VETOED".to_string()
    } else {
        "ARBITRATED".to_string()
    };

    (
        best_mv,
        ArbitrationBreakdown {
            status,
            primary_move: cnn_1,
            aux_move: whale_best.0,
            veto_reason,
            candidates: breakdowns,
        },
    )
}

pub fn xhigh_arbitrate(
    state: &GameState,
    cnn_ranked: &[(Move, f32)],
    whale_best: (Move, f64),
    whale_evals: &[(Move, f64)],
) -> Move {
    xhigh_arbitrate_detailed(state, &[], cnn_ranked, whale_best, whale_evals).0
}

impl Evaluator for HeuristicCandidatePolicy {
    fn evaluate(&self, state: &Afterstate, feat: &TopoFeatures) -> f32 {
        let score_diff = (state.my_score as f32) - (state.opp_score as f32);
        let cauldron_diff = (feat.max_my_cauldron as f32) - (feat.max_opp_cauldron as f32);
        let elasticity_diff = (feat.my_elasticity - feat.opp_elasticity) * 50.0;
        let raw = score_diff * 1.0 + cauldron_diff * 0.5 + elasticity_diff * 0.2;
        (1.0 / (1.0 + (-raw / 30.0).exp())).clamp(0.0, 1.0)
    }

    fn policy_top_k(&self, state: &GameState, moves: &[Move], k: usize) -> Vec<Move> {
        Self::policy_top_k(state, moves, k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Bitboard;

    #[test]
    fn test_policy_top_k_prioritizes_closed_loop_capture() {
        let mut my_bb = Bitboard::new();
        for y in 0..=3 {
            my_bb.set(10, y);
            my_bb.set(15, y);
        }
        for x in 10..=13 {
            my_bb.set(x, 3);
        }
        let mut opp_bb = Bitboard::new();
        opp_bb.set(39, 14);
        let state = GameState {
            my_bb,
            opp_bb,
            my_score: my_bb.count_ones(),
            opp_score: 1,
            turn: 4,
        };
        let closing_move = Move { x: 14, y: 3, w: 1, h: 1 };
        let big_move_1 = Move { x: 0, y: 0, w: 6, h: 6 };
        let big_move_2 = Move { x: 20, y: 5, w: 6, h: 6 };
        let moves = vec![big_move_1, big_move_2, closing_move];
        let top = HeuristicCandidatePolicy::policy_top_k(&state, &moves, 1);
        assert_eq!(top, vec![closing_move]);
    }
}
