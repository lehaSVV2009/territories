use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::bitboard::apply_move;
use crate::search::{
    prepare_2ply_expectimax_leaves, prepare_cnn_1ply_batch, score_2ply_expectimax_plan,
    xhigh_arbitrate_detailed, ArbitrationBreakdown, HeuristicCandidatePolicy,
};
use crate::topology::{bitboard_to_rows, encode_tensor, extract_features};
use crate::types::{
    DiceRoll, EffortLevel, GameState, Move, HEIGHT, TENSOR_SIZE, WIN_SCORE,
};
use crate::whale_engine::WhaleBot;
use crate::{
    apply_move_core, ort_bridge_infer, pass_turn_core, state_to_afterstate,
    ArbitrationTelemetryDTO, CandidateTelemetryDTO, EngineRuntime, Placement,
    PositionTelemetryDTO, ENGINE,
};

struct SearchOutcome {
    chosen: Option<Move>,
    cnn_1ply: Vec<(Move, f32)>,
    cnn_2ply: Option<Vec<(Move, f32)>>,
    arb: Option<ArbitrationBreakdown>,
}

fn to_placement(mv: Move) -> Placement {
    Placement {
        x: mv.x,
        y: mv.y,
        w: mv.w,
        h: mv.h,
    }
}

async fn infer_ort_scores(flat_tensor: &[f32], batch_size: usize) -> Result<Vec<f32>, JsValue> {
    if batch_size == 0 {
        return Ok(Vec::new());
    }
    let promise = ort_bridge_infer(flat_tensor, batch_size);
    let js_scores = JsFuture::from(promise).await?;
    let scores_array = js_sys::Float32Array::new(&js_scores);
    let scores = scores_array.to_vec();
    if scores.len() != batch_size {
        return Err(JsValue::from_str(&format!(
            "Inference score length mismatch: expected {}, got {}",
            batch_size,
            scores.len()
        )));
    }
    Ok(scores)
}

async fn run_cnn_1ply(
    state: &GameState,
    dice: DiceRoll,
    beam_k: usize,
) -> Result<(Option<Move>, Vec<(Move, f32)>), JsValue> {
    let batch = prepare_cnn_1ply_batch(state, dice, beam_k);
    if batch.candidates.is_empty() {
        return Ok((None, Vec::new()));
    }
    if let Some(mv) = batch.immediate_win_move {
        return Ok((Some(mv), vec![(mv, 1.0)]));
    }

    let mut flat = Vec::with_capacity(batch.leaves.len() * TENSOR_SIZE);
    for (after, feat) in &batch.leaves {
        flat.extend_from_slice(&encode_tensor(after, feat));
    }

    let scores = infer_ort_scores(&flat, batch.candidates.len()).await?;
    let mut pairs: Vec<(Move, f32)> = batch.candidates.into_iter().zip(scores).collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let best = pairs.first().map(|&(m, _)| m);
    Ok((best, pairs))
}

async fn run_cnn_2ply_expectimax(
    state: &GameState,
    dice: DiceRoll,
    beam_k: usize,
    pre_prune_k: usize,
    opp_top_k: usize,
) -> Result<(Vec<(Move, f32)>, Vec<(Move, f32)>), JsValue> {
    let batch = prepare_cnn_1ply_batch(state, dice, beam_k);
    if batch.candidates.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    if let Some(win_mv) = batch.immediate_win_move {
        let v = vec![(win_mv, 1.0)];
        return Ok((v.clone(), v));
    }

    let mut flat1 = Vec::with_capacity(batch.leaves.len() * TENSOR_SIZE);
    for (after, feat) in &batch.leaves {
        flat1.extend_from_slice(&encode_tensor(after, feat));
    }
    let scores1 = infer_ort_scores(&flat1, batch.candidates.len()).await?;
    let mut cnn_1ply_pairs: Vec<(Move, f32)> =
        batch.candidates.iter().copied().zip(scores1).collect();
    cnn_1ply_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let pre_pruned: Vec<Move> = if batch.candidates.len() > pre_prune_k {
        cnn_1ply_pairs.iter().take(pre_prune_k).map(|&(m, _)| m).collect()
    } else {
        batch.candidates
    };

    let plan = prepare_2ply_expectimax_leaves(state, &pre_pruned, opp_top_k);
    let leaf_scores = infer_ort_scores(&plan.flat_tensor, plan.batch_size).await?;
    Ok((score_2ply_expectimax_plan(&plan, &leaf_scores), cnn_1ply_pairs))
}

pub(crate) fn build_telemetry(
    snapshot: &GameState,
    chosen: Option<Move>,
    moved_player: u8,
    cnn_1ply: &[(Move, f32)],
    cnn_2ply: Option<&[(Move, f32)]>,
    arb: Option<&ArbitrationBreakdown>,
) -> PositionTelemetryDTO {
    let check_loops = !snapshot.my_bb.is_empty() && !snapshot.opp_bb.is_empty();
    let my_rows = if check_loops {
        bitboard_to_rows(&snapshot.my_bb)
    } else {
        [0u64; HEIGHT]
    };

    let (channels_flat, risk_map) = match chosen {
        Some(mv) => {
            let after = apply_move(snapshot, mv);
            let feat = extract_features(&after);
            (encode_tensor(&after, &feat).to_vec(), feat.p_seal_opp.to_vec())
        }
        None => {
            let after = state_to_afterstate(snapshot);
            let feat = extract_features(&after);
            (encode_tensor(&after, &feat).to_vec(), feat.p_seal_opp.to_vec())
        }
    };

    let candidates: Vec<CandidateTelemetryDTO> = if let Some(ab) = arb {
        ab.candidates
            .iter()
            .map(|c| CandidateTelemetryDTO {
                x: c.mv.x,
                y: c.mv.y,
                w: c.mv.w,
                h: c.mv.h,
                heuristic_score: c.heuristic_score,
                capture_cells: c.capture_cells,
                capture_bonus: c.capture_bonus,
                p1_prob: c.p_cnn_1,
                p2_prob: c.p_cnn_2,
                aux_score: c.aux_score,
                aux_sigmoid: c.aux_sigmoid,
                vetoed: c.vetoed,
                penalty: c.penalty,
                final_score: c.final_score,
                selected: c.selected,
            })
            .collect()
    } else if let Some(ranked2) = cnn_2ply {
        ranked2
            .iter()
            .map(|&(mv, p2)| {
                let (h_score, cap_cells, cap_bonus) =
                    HeuristicCandidatePolicy::score_move_details(snapshot, mv, check_loops, &my_rows);
                let p1 = cnn_1ply.iter().find(|&&(m, _)| m == mv).map(|&(_, s)| s);
                CandidateTelemetryDTO {
                    x: mv.x,
                    y: mv.y,
                    w: mv.w,
                    h: mv.h,
                    heuristic_score: h_score,
                    capture_cells: cap_cells,
                    capture_bonus: cap_bonus,
                    p1_prob: p1,
                    p2_prob: Some(p2),
                    aux_score: None,
                    aux_sigmoid: None,
                    vetoed: false,
                    penalty: 0.0,
                    final_score: p2,
                    selected: chosen == Some(mv),
                }
            })
            .collect()
    } else {
        cnn_1ply
            .iter()
            .take(12)
            .map(|&(mv, p1)| {
                let (h_score, cap_cells, cap_bonus) =
                    HeuristicCandidatePolicy::score_move_details(snapshot, mv, check_loops, &my_rows);
                CandidateTelemetryDTO {
                    x: mv.x,
                    y: mv.y,
                    w: mv.w,
                    h: mv.h,
                    heuristic_score: h_score,
                    capture_cells: cap_cells,
                    capture_bonus: cap_bonus,
                    p1_prob: Some(p1),
                    p2_prob: None,
                    aux_score: None,
                    aux_sigmoid: None,
                    vetoed: false,
                    penalty: 0.0,
                    final_score: p1,
                    selected: chosen == Some(mv),
                }
            })
            .collect()
    };

    let arbitration = arb.map(|ab| ArbitrationTelemetryDTO {
        status: ab.status.clone(),
        primary_move: Some(to_placement(ab.primary_move)),
        aux_move: Some(to_placement(ab.aux_move)),
        veto_reason: ab.veto_reason.clone(),
    });

    let mover_prob = chosen
        .and_then(|mv| {
            cnn_2ply
                .and_then(|r2| r2.iter().find(|&&(m, _)| m == mv).map(|&(_, p)| p))
                .or_else(|| cnn_1ply.iter().find(|&&(m, _)| m == mv).map(|&(_, p)| p))
        })
        .unwrap_or(0.5);

    let eval_win_prob = if moved_player == 1 {
        mover_prob
    } else {
        1.0 - mover_prob
    };

    PositionTelemetryDTO {
        channels_flat,
        risk_map,
        candidates,
        arbitration,
        eval_win_prob,
        perspective_player: moved_player,
    }
}

async fn run_low_or_med(
    snapshot: &GameState,
    dice: DiceRoll,
    beam_k: usize,
    collect_tel: bool,
) -> Result<SearchOutcome, JsValue> {
    let (mv_opt, pairs) = run_cnn_1ply(snapshot, dice, beam_k).await?;
    Ok(SearchOutcome {
        chosen: mv_opt,
        cnn_1ply: if collect_tel { pairs } else { Vec::new() },
        cnn_2ply: None,
        arb: None,
    })
}

async fn run_high(
    snapshot: &GameState,
    dice: DiceRoll,
    collect_tel: bool,
) -> Result<SearchOutcome, JsValue> {
    let (ranked, pairs1) = run_cnn_2ply_expectimax(snapshot, dice, 24, 6, 3).await?;
    let mv_opt = ranked.first().map(|&(m, _)| m);
    Ok(SearchOutcome {
        chosen: mv_opt,
        cnn_1ply: if collect_tel { pairs1 } else { Vec::new() },
        cnn_2ply: if collect_tel { Some(ranked) } else { None },
        arb: None,
    })
}

async fn resolve_consensus(
    snapshot: &GameState,
    mut cnn_ranked: Vec<(Move, f32)>,
    pairs1: Vec<(Move, f32)>,
    whale_res: Option<(Move, f64)>,
    collect_tel: bool,
) -> Result<SearchOutcome, JsValue> {
    match whale_res {
        None => {
            let mv_opt = cnn_ranked.first().map(|&(m, _)| m);
            Ok(SearchOutcome {
                chosen: mv_opt,
                cnn_1ply: if collect_tel { pairs1 } else { Vec::new() },
                cnn_2ply: if collect_tel { Some(cnn_ranked) } else { None },
                arb: None,
            })
        }
        Some((m_whale, s_whale)) => {
            if cnn_ranked[0].0 == m_whale && !collect_tel {
                return Ok(SearchOutcome {
                    chosen: Some(m_whale),
                    cnn_1ply: Vec::new(),
                    cnn_2ply: None,
                    arb: None,
                });
            }

            if !cnn_ranked.iter().any(|&(m, _)| m == m_whale) {
                let extra_plan = prepare_2ply_expectimax_leaves(snapshot, &[m_whale], 3);
                let extra_scores =
                    infer_ort_scores(&extra_plan.flat_tensor, extra_plan.batch_size).await?;
                let extra_ranked = score_2ply_expectimax_plan(&extra_plan, &extra_scores);
                if let Some(&item) = extra_ranked.first() {
                    cnn_ranked.push(item);
                }
            }

            let mut union_moves: Vec<Move> =
                cnn_ranked.iter().take(4).map(|&(m, _)| m).collect();
            if !union_moves.contains(&m_whale) {
                union_moves.push(m_whale);
            }

            let whale_evals = ENGINE.with(|cell| {
                let mut rt = cell.borrow_mut();
                let EngineRuntime {
                    ref mut nnue_model,
                    ref mut whale_bot,
                    ..
                } = *rt;
                let nnue = nnue_model.as_ref().expect("model loaded");
                let bot = whale_bot.get_or_insert_with(|| Box::new(WhaleBot::default()));
                bot.evaluate_candidates(snapshot, nnue, &union_moves, Some((m_whale, s_whale)))
            });

            let (chosen, breakdown) = xhigh_arbitrate_detailed(
                snapshot,
                &pairs1,
                &cnn_ranked,
                (m_whale, s_whale),
                &whale_evals,
            );
            Ok(SearchOutcome {
                chosen: Some(chosen),
                cnn_1ply: if collect_tel { pairs1 } else { Vec::new() },
                cnn_2ply: if collect_tel { Some(cnn_ranked) } else { None },
                arb: if collect_tel { Some(breakdown) } else { None },
            })
        }
    }
}

async fn run_xhigh(
    snapshot: &GameState,
    dice: DiceRoll,
    aux_loaded: bool,
    collect_tel: bool,
) -> Result<SearchOutcome, JsValue> {
    if !aux_loaded {
        let (ranked, pairs1) = run_cnn_2ply_expectimax(snapshot, dice, 32, 8, 4).await?;
        let mv_opt = ranked.first().map(|&(m, _)| m);
        return Ok(SearchOutcome {
            chosen: mv_opt,
            cnn_1ply: if collect_tel { pairs1 } else { Vec::new() },
            cnn_2ply: if collect_tel { Some(ranked) } else { None },
            arb: None,
        });
    }

    let (cnn_ranked, pairs1) = run_cnn_2ply_expectimax(snapshot, dice, 24, 6, 3).await?;
    if cnn_ranked.is_empty() {
        return Ok(SearchOutcome {
            chosen: None,
            cnn_1ply: Vec::new(),
            cnn_2ply: None,
            arb: None,
        });
    }

    if cnn_ranked[0].1 >= 1.0 && apply_move(snapshot, cnn_ranked[0].0).my_score >= WIN_SCORE {
        let mv = cnn_ranked[0].0;
        return Ok(SearchOutcome {
            chosen: Some(mv),
            cnn_1ply: if collect_tel { pairs1 } else { Vec::new() },
            cnn_2ply: if collect_tel { Some(cnn_ranked) } else { None },
            arb: None,
        });
    }

    let whale_res = ENGINE.with(|cell| {
        let mut rt = cell.borrow_mut();
        let EngineRuntime {
            ref mut nnue_model,
            ref mut whale_bot,
            ..
        } = *rt;
        let nnue = nnue_model.as_ref().expect("model loaded");
        let bot = whale_bot.get_or_insert_with(|| Box::new(WhaleBot::default()));
        bot.select_move_with_score(snapshot, nnue, dice.0, dice.1)
    });

    resolve_consensus(snapshot, cnn_ranked, pairs1, whale_res, collect_tel).await
}

pub(crate) async fn execute_bot_move(
    d1: u8,
    d2: u8,
    effort: Option<String>,
    include_telemetry: Option<bool>,
) -> Result<JsValue, JsValue> {
    let effort_level =
        EffortLevel::from_opt_str(effort.as_deref()).map_err(|e| JsValue::from_str(&e))?;
    let collect_tel = include_telemetry.unwrap_or(false);

    let (snapshot, aux_loaded) = ENGINE
        .with(|cell| -> Result<(GameState, bool), String> {
            let rt = cell.borrow();
            if rt.game_over {
                return Err("Game is already over".to_string());
            }
            if !rt.model_ready {
                return Err("Bot model not initialized".to_string());
            }
            Ok((rt.state, rt.nnue_model.is_some()))
        })
        .map_err(|e| JsValue::from_str(&e))?;

    let moved_player = if (snapshot.turn % 2) == 0 { 1u8 } else { 2u8 };
    let dice = DiceRoll(d1, d2);

    let outcome = match effort_level {
        EffortLevel::Low => run_low_or_med(&snapshot, dice, 12, collect_tel).await?,
        EffortLevel::Med => run_low_or_med(&snapshot, dice, 24, collect_tel).await?,
        EffortLevel::High => run_high(&snapshot, dice, collect_tel).await?,
        EffortLevel::XHigh => run_xhigh(&snapshot, dice, aux_loaded, collect_tel).await?,
    };

    let mut result = ENGINE
        .with(|cell| {
            let mut rt = cell.borrow_mut();
            match outcome.chosen {
                Some(mv) => apply_move_core(&mut rt, mv),
                None => Ok(pass_turn_core(&mut rt)),
            }
        })
        .map_err(|e| JsValue::from_str(&e))?;

    if collect_tel {
        result.telemetry = Some(build_telemetry(
            &snapshot,
            outcome.chosen,
            moved_player,
            &outcome.cnn_1ply,
            outcome.cnn_2ply.as_deref(),
            outcome.arb.as_ref(),
        ));
    }

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
