pub mod bitboard;
mod bot_orchestrator;
pub mod nnue;
pub mod search;
pub mod topology;
pub mod types;
pub mod whale_engine;

use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::bitboard::{apply_move, has_legal_move, legal_moves, rect_mask};
use crate::nnue::NnueModel;
use crate::topology::{encode_tensor, extract_features};
use crate::types::{
    Bitboard, DiceRoll, GameState, Move, HEIGHT, NUM_CELLS, P2_KOMI, WIDTH, WIN_SCORE,
};
use crate::whale_engine::WhaleBot;

#[wasm_bindgen(inline_js = "
export async function ortBridgeInit(bytes) {
  if (typeof globalThis.__territoriesOrtInit !== 'function') {
    throw new Error('ONNX bridge not initialized in worker');
  }
  return await globalThis.__territoriesOrtInit(bytes);
}
export async function ortBridgeInfer(flatTensor, batchSize) {
  if (typeof globalThis.__territoriesOrtInfer !== 'function') {
    throw new Error('ONNX inference bridge not initialized in worker');
  }
  return await globalThis.__territoriesOrtInfer(flatTensor, batchSize);
}
")]
extern "C" {
    #[wasm_bindgen(js_name = ortBridgeInit)]
    fn ort_bridge_init(bytes: &[u8]) -> js_sys::Promise;

    #[wasm_bindgen(js_name = ortBridgeInfer)]
    pub(crate) fn ort_bridge_infer(flat_tensor: &[f32], batch_size: usize) -> js_sys::Promise;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Placement {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStateDTO {
    pub grid: Vec<u8>,
    pub p1_score: u16,
    pub p2_score: u16,
    pub turn: u16,
    pub active_player: u8,
    pub consecutive_passes: u8,
    pub game_over: bool,
    pub winner: u8,
    pub komi: u16,
    pub win_score: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateTelemetryDTO {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
    pub heuristic_score: i32,
    pub capture_cells: u16,
    pub capture_bonus: i32,
    pub p1_prob: Option<f32>,
    pub p2_prob: Option<f32>,
    pub aux_score: Option<f64>,
    pub aux_sigmoid: Option<f32>,
    pub vetoed: bool,
    pub penalty: f32,
    pub final_score: f32,
    pub selected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrationTelemetryDTO {
    pub status: String,
    pub primary_move: Option<Placement>,
    pub aux_move: Option<Placement>,
    pub veto_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionTelemetryDTO {
    pub channels_flat: Vec<f32>,
    pub risk_map: Vec<f32>,
    pub candidates: Vec<CandidateTelemetryDTO>,
    pub arbitration: Option<ArbitrationTelemetryDTO>,
    pub eval_win_prob: f32,
    pub perspective_player: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionInspectDTO {
    pub channels_flat: Vec<f32>,
    pub risk_map: Vec<f32>,
    pub perspective_player: u8,
    pub p1_score: u16,
    pub p2_score: u16,
    pub turn: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveResultDTO {
    pub state: GameStateDTO,
    pub moved_player: u8,
    pub passed: bool,
    pub last_move: Option<Placement>,
    pub placed_cells: Vec<u16>,
    pub captured_cells: Vec<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<PositionTelemetryDTO>,
}

pub(crate) struct EngineRuntime {
    pub(crate) state: GameState,
    pub(crate) komi: u16,
    pub(crate) consecutive_passes: u8,
    pub(crate) game_over: bool,
    pub(crate) winner: u8,
    pub(crate) model_ready: bool,
    pub(crate) nnue_model: Option<NnueModel>,
    pub(crate) whale_bot: Option<Box<WhaleBot>>,
}

impl EngineRuntime {
    fn new_with_komi(komi: u16) -> Self {
        Self {
            state: GameState {
                my_bb: Bitboard::new(),
                opp_bb: Bitboard::new(),
                my_score: 0,
                opp_score: komi,
                turn: 0,
            },
            komi,
            consecutive_passes: 0,
            game_over: false,
            winner: 0,
            model_ready: false,
            nnue_model: None,
            whale_bot: None,
        }
    }
}

impl Default for EngineRuntime {
    fn default() -> Self {
        Self::new_with_komi(P2_KOMI)
    }
}

thread_local! {
    pub(crate) static ENGINE: RefCell<EngineRuntime> = RefCell::new(EngineRuntime::default());
}

fn build_state_dto(rt: &EngineRuntime) -> GameStateDTO {
    let mut grid = vec![0u8; NUM_CELLS];
    let is_p1_turn = (rt.state.turn % 2) == 0;
    let (p1_bb, p2_bb, p1_score, p2_score, active_player) = if is_p1_turn {
        (
            &rt.state.my_bb,
            &rt.state.opp_bb,
            rt.state.my_score,
            rt.state.opp_score,
            1u8,
        )
    } else {
        (
            &rt.state.opp_bb,
            &rt.state.my_bb,
            rt.state.opp_score,
            rt.state.my_score,
            2u8,
        )
    };

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = y * WIDTH + x;
            if p1_bb.get(x, y) {
                grid[idx] = 1;
            } else if p2_bb.get(x, y) {
                grid[idx] = 2;
            }
        }
    }

    GameStateDTO {
        grid,
        p1_score,
        p2_score,
        turn: rt.state.turn,
        active_player,
        consecutive_passes: rt.consecutive_passes,
        game_over: rt.game_over,
        winner: rt.winner,
        komi: rt.komi,
        win_score: WIN_SCORE,
    }
}

fn bb_to_cell_indices(bb: &Bitboard) -> Vec<u16> {
    let mut out = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if bb.get(x, y) {
                out.push((y * WIDTH + x) as u16);
            }
        }
    }
    out
}

fn is_move_legal_exact(state: &GameState, mv: Move) -> bool {
    if mv.w == 0 || mv.h == 0 {
        return false;
    }
    if (mv.x as usize) + (mv.w as usize) > WIDTH || (mv.y as usize) + (mv.h as usize) > HEIGHT {
        return false;
    }
    let occupied = state.my_bb | state.opp_bb;
    let r_bb = rect_mask(mv.x, mv.y, mv.w, mv.h);
    if !(r_bb & occupied).is_empty() {
        return false;
    }
    if state.my_bb.is_empty() {
        let (exp_x, exp_y) = if state.turn % 2 == 0 {
            (0u8, 0u8)
        } else {
            ((WIDTH - mv.w as usize) as u8, (HEIGHT - mv.h as usize) as u8)
        };
        return mv.x == exp_x && mv.y == exp_y;
    }
    let dilated = state.my_bb.shift_left()
        | state.my_bb.shift_right()
        | state.my_bb.shift_up()
        | state.my_bb.shift_down();
    !(r_bb & dilated).is_empty()
}

pub(crate) fn apply_move_core(rt: &mut EngineRuntime, mv: Move) -> Result<MoveResultDTO, String> {
    if rt.game_over {
        return Err("Game is already over".to_string());
    }
    if !is_move_legal_exact(&rt.state, mv) {
        return Err(format!("Illegal placement: ({}, {}, {}x{})", mv.x, mv.y, mv.w, mv.h));
    }

    let moved_player = if (rt.state.turn % 2) == 0 { 1u8 } else { 2u8 };
    let other_player = if moved_player == 1 { 2u8 } else { 1u8 };

    let before_my = rt.state.my_bb;
    let before_opp = rt.state.opp_bb;
    let r_bb = rect_mask(mv.x, mv.y, mv.w, mv.h);

    let after = apply_move(&rt.state, mv);
    let captured_my = after.my_bb & (!before_my) & (!r_bb);
    let captured_opp = after.opp_bb & (!before_opp);
    let captured_all = captured_my | captured_opp;

    let placed_cells = bb_to_cell_indices(&r_bb);
    let captured_cells = bb_to_cell_indices(&captured_all);

    rt.consecutive_passes = 0;

    if after.my_score >= WIN_SCORE {
        rt.game_over = true;
        rt.winner = moved_player;
    } else if after.opp_score >= WIN_SCORE {
        rt.game_over = true;
        rt.winner = other_player;
    } else if (after.my_bb.count_ones() + after.opp_bb.count_ones()) as usize >= NUM_CELLS {
        rt.game_over = true;
        rt.winner = if after.my_score > after.opp_score {
            moved_player
        } else if after.opp_score > after.my_score {
            other_player
        } else {
            0
        };
    }

    rt.state = crate::bitboard::flip_perspective(&after);

    Ok(MoveResultDTO {
        state: build_state_dto(rt),
        moved_player,
        passed: false,
        last_move: Some(Placement {
            x: mv.x,
            y: mv.y,
            w: mv.w,
            h: mv.h,
        }),
        placed_cells,
        captured_cells,
        telemetry: None,
    })
}

pub(crate) fn pass_turn_core(rt: &mut EngineRuntime) -> MoveResultDTO {
    let moved_player = if (rt.state.turn % 2) == 0 { 1u8 } else { 2u8 };
    rt.consecutive_passes += 1;

    rt.state = GameState {
        my_bb: rt.state.opp_bb,
        opp_bb: rt.state.my_bb,
        my_score: rt.state.opp_score,
        opp_score: rt.state.my_score,
        turn: rt.state.turn + 1,
    };

    if rt.consecutive_passes >= 2 {
        rt.game_over = true;
        let dto = build_state_dto(rt);
        rt.winner = if dto.p1_score > dto.p2_score {
            1
        } else if dto.p2_score > dto.p1_score {
            2
        } else {
            0
        };
    }

    MoveResultDTO {
        state: build_state_dto(rt),
        moved_player,
        passed: true,
        last_move: None,
        placed_cells: Vec::new(),
        captured_cells: Vec::new(),
        telemetry: None,
    }
}

#[wasm_bindgen]
pub async fn init_model(onnx_bytes: &[u8]) -> Result<(), JsValue> {
    let promise = ort_bridge_init(onnx_bytes);
    JsFuture::from(promise).await?;
    ENGINE.with(|cell| {
        cell.borrow_mut().model_ready = true;
    });
    Ok(())
}

#[wasm_bindgen]
pub fn init_nnue(nnue_bytes: &[u8]) -> Result<(), JsValue> {
    let model = NnueModel::from_bytes(nnue_bytes).map_err(|e| JsValue::from_str(&e))?;
    ENGINE.with(|cell| {
        let mut rt = cell.borrow_mut();
        rt.nnue_model = Some(model);
        if rt.whale_bot.is_none() {
            rt.whale_bot = Some(Box::new(WhaleBot::default()));
        }
    });
    Ok(())
}

#[wasm_bindgen]
pub fn is_nnue_loaded() -> bool {
    ENGINE.with(|cell| cell.borrow().nnue_model.is_some())
}

#[wasm_bindgen]
pub fn new_game(komi: Option<u16>) -> Result<JsValue, JsValue> {
    let active_komi = komi.unwrap_or(P2_KOMI);
    let dto = ENGINE.with(|cell| {
        let mut rt = cell.borrow_mut();
        let model_ready = rt.model_ready;
        let nnue_model = rt.nnue_model.take();
        let mut whale_bot = rt.whale_bot.take();
        if let Some(ref mut wb) = whale_bot {
            wb.tt.new_search();
            wb.reset_stats();
        }
        *rt = EngineRuntime::new_with_komi(active_komi);
        rt.model_ready = model_ready;
        rt.nnue_model = nnue_model;
        rt.whale_bot = whale_bot;
        build_state_dto(&rt)
    });
    serde_wasm_bindgen::to_value(&dto).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_legal_placements(w: u8, h: u8) -> Result<JsValue, JsValue> {
    let placements: Vec<Placement> = ENGINE.with(|cell| {
        let rt = cell.borrow();
        if rt.game_over || w == 0 || h == 0 {
            return Vec::new();
        }
        let all = legal_moves(&rt.state, DiceRoll(w, h));
        all.into_iter()
            .filter(|m| m.w == w && m.h == h)
            .map(|m| Placement {
                x: m.x,
                y: m.y,
                w: m.w,
                h: m.h,
            })
            .collect()
    });
    serde_wasm_bindgen::to_value(&placements).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn has_any_legal_move(d1: u8, d2: u8) -> bool {
    ENGINE.with(|cell| {
        let rt = cell.borrow();
        if rt.game_over {
            return false;
        }
        has_legal_move(&rt.state, DiceRoll(d1, d2))
    })
}

#[wasm_bindgen]
pub fn pass_turn() -> Result<JsValue, JsValue> {
    let res = ENGINE.with(|cell| {
        let mut rt = cell.borrow_mut();
        pass_turn_core(&mut rt)
    });
    serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn apply_human_move(x: u8, y: u8, w: u8, h: u8) -> Result<JsValue, JsValue> {
    let res = ENGINE.with(|cell| {
        let mut rt = cell.borrow_mut();
        apply_move_core(&mut rt, Move { x, y, w, h })
    });
    match res {
        Ok(dto) => serde_wasm_bindgen::to_value(&dto).map_err(|e| JsValue::from_str(&e.to_string())),
        Err(msg) => Err(JsValue::from_str(&msg)),
    }
}

fn preview_capture_core(rt: &EngineRuntime, x: u8, y: u8, w: u8, h: u8) -> Vec<u16> {
    let mv = Move { x, y, w, h };
    if rt.game_over || !is_move_legal_exact(&rt.state, mv) {
        return Vec::new();
    }
    let before_my = rt.state.my_bb;
    let r_bb = rect_mask(mv.x, mv.y, mv.w, mv.h);
    let after = apply_move(&rt.state, mv);
    let captured_my = after.my_bb & (!before_my) & (!r_bb);
    bb_to_cell_indices(&captured_my)
}

#[wasm_bindgen]
pub fn preview_move_capture(x: u8, y: u8, w: u8, h: u8) -> Result<JsValue, JsValue> {
    let cells = ENGINE.with(|cell| preview_capture_core(&cell.borrow(), x, y, w, h));
    serde_wasm_bindgen::to_value(&cells).map_err(|e| JsValue::from_str(&e.to_string()))
}

pub(crate) fn state_to_afterstate(state: &GameState) -> crate::types::Afterstate {
    crate::types::Afterstate {
        my_bb: state.my_bb,
        opp_bb: state.opp_bb,
        my_score: state.my_score,
        opp_score: state.opp_score,
        turn: state.turn,
    }
}

fn inspect_position_core(rt: &EngineRuntime) -> PositionInspectDTO {
    let after = state_to_afterstate(&rt.state);
    let feat = extract_features(&after);
    let tensor = encode_tensor(&after, &feat);
    let s = build_state_dto(rt);
    PositionInspectDTO {
        channels_flat: tensor.to_vec(),
        risk_map: feat.p_seal_opp.to_vec(),
        perspective_player: s.active_player,
        p1_score: s.p1_score,
        p2_score: s.p2_score,
        turn: s.turn,
    }
}

#[wasm_bindgen]
pub fn inspect_position() -> Result<JsValue, JsValue> {
    let dto = ENGINE.with(|cell| inspect_position_core(&cell.borrow()));
    serde_wasm_bindgen::to_value(&dto).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn restore_state_core(
    rt: &mut EngineRuntime,
    grid: &[u8],
    p1_score: u16,
    p2_score: u16,
    turn: u16,
    consecutive_passes: u8,
    komi: u16,
) -> Result<GameStateDTO, String> {
    if grid.len() != NUM_CELLS {
        return Err(format!("Expected {} grid cells, got {}", NUM_CELLS, grid.len()));
    }
    let mut p1_bb = Bitboard::new();
    let mut p2_bb = Bitboard::new();
    for (idx, &val) in grid.iter().enumerate() {
        let x = idx % WIDTH;
        let y = idx / WIDTH;
        if val == 1 {
            p1_bb.set(x, y);
        } else if val == 2 {
            p2_bb.set(x, y);
        }
    }
    let is_p1_turn = (turn % 2) == 0;
    rt.state = if is_p1_turn {
        GameState {
            my_bb: p1_bb,
            opp_bb: p2_bb,
            my_score: p1_score,
            opp_score: p2_score,
            turn,
        }
    } else {
        GameState {
            my_bb: p2_bb,
            opp_bb: p1_bb,
            my_score: p2_score,
            opp_score: p1_score,
            turn,
        }
    };
    rt.komi = komi;
    rt.consecutive_passes = consecutive_passes;
    let occupied = (p1_bb.count_ones() + p2_bb.count_ones()) as usize;
    if p1_score >= WIN_SCORE {
        rt.game_over = true;
        rt.winner = 1;
    } else if p2_score >= WIN_SCORE {
        rt.game_over = true;
        rt.winner = 2;
    } else if consecutive_passes >= 2 || occupied >= NUM_CELLS {
        rt.game_over = true;
        rt.winner = if p1_score > p2_score {
            1
        } else if p2_score > p1_score {
            2
        } else {
            0
        };
    } else {
        rt.game_over = false;
        rt.winner = 0;
    }
    if let Some(ref mut wb) = rt.whale_bot {
        wb.tt.new_search();
        wb.reset_stats();
    }
    Ok(build_state_dto(rt))
}

#[wasm_bindgen]
pub fn restore_state(
    grid: &[u8],
    p1_score: u16,
    p2_score: u16,
    turn: u16,
    consecutive_passes: u8,
    komi: u16,
) -> Result<JsValue, JsValue> {
    let dto = ENGINE
        .with(|cell| restore_state_core(&mut cell.borrow_mut(), grid, p1_score, p2_score, turn, consecutive_passes, komi))
        .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&dto).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub async fn compute_bot_move(
    d1: u8,
    d2: u8,
    effort: Option<String>,
    include_telemetry: Option<bool>,
) -> Result<JsValue, JsValue> {
    bot_orchestrator::execute_bot_move(d1, d2, effort, include_telemetry).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot_orchestrator::build_telemetry;
    use crate::search::{
        evaluate_2ply_expectimax_with_evaluator, prepare_2ply_expectimax_leaves,
        prepare_cnn_1ply_batch, xhigh_arbitrate, xhigh_arbitrate_detailed,
        HeuristicCandidatePolicy,
    };
    use crate::types::{Evaluator, TENSOR_SIZE};

    const TEST_NNUE_BYTES: &[u8] = include_bytes!("../../public/whale_3_0.nnue");

    #[test]
    fn test_runtime_komi_18_and_0_and_closed_loop_capture_1x1() {
        let mut rt = EngineRuntime::default();
        let s0 = build_state_dto(&rt);
        assert_eq!(s0.komi, 18);
        assert_eq!(s0.p1_score, 0);
        assert_eq!(s0.p2_score, 18);

        let r1 = apply_move_core(&mut rt, Move { x: 0, y: 0, w: 4, h: 3 }).unwrap();
        assert_eq!(r1.state.p1_score, 12);
        assert_eq!(r1.state.p2_score, 18);

        let r2 = apply_move_core(&mut rt, Move { x: 35, y: 12, w: 5, h: 3 }).unwrap();
        assert_eq!(r2.state.p2_score, 18 + 15);
        assert_eq!(r2.state.p1_score, 12);

        let mut rt_classic = EngineRuntime::new_with_komi(0);
        let sc0 = build_state_dto(&rt_classic);
        assert_eq!(sc0.komi, 0);
        assert_eq!(sc0.p1_score, 0);
        assert_eq!(sc0.p2_score, 0);
        let rc1 = apply_move_core(&mut rt_classic, Move { x: 0, y: 0, w: 3, h: 2 }).unwrap();
        assert_eq!(rc1.state.p1_score, 6);
        assert_eq!(rc1.state.p2_score, 0);
        let rc2 = apply_move_core(&mut rt_classic, Move { x: 37, y: 13, w: 3, h: 2 }).unwrap();
        assert_eq!(rc2.state.p2_score, 6);

        let mut p1_bb = Bitboard::new();
        for y in 0..=3 {
            p1_bb.set(10, y);
            p1_bb.set(15, y);
        }
        for x in 10..=13 {
            p1_bb.set(x, 3);
        }
        let mut p2_bb = Bitboard::new();
        p2_bb.set(39, 14);
        rt.state = GameState {
            my_bb: p1_bb,
            opp_bb: p2_bb,
            my_score: p1_bb.count_ones(),
            opp_score: 19,
            turn: 4,
        };

        let loop_res = apply_move_core(&mut rt, Move { x: 14, y: 3, w: 1, h: 1 }).unwrap();
        assert_eq!(loop_res.placed_cells.len(), 1);
        assert_eq!(loop_res.captured_cells.len(), 12);
        assert_eq!(loop_res.state.p1_score, p1_bb.count_ones() + 1 + 12);
    }

    #[test]
    fn test_high_batched_2ply_expectimax_packs_leaves_and_avoids_immediate_opp_win() {
        let mut p1_bb = Bitboard::new();
        for y in 0..10 {
            for x in 0..26 {
                p1_bb.set(x, y);
            }
        }
        let mut p2_bb = Bitboard::new();
        for y in 10..15 {
            for x in 20..40 {
                p2_bb.set(x, y);
            }
        }
        let state = GameState {
            my_bb: p1_bb,
            opp_bb: p2_bb,
            my_score: 260,
            opp_score: 295,
            turn: 20,
        };

        let batch = prepare_cnn_1ply_batch(&state, DiceRoll(2, 3), 24);
        assert!(!batch.candidates.is_empty());
        let pre_pruned: Vec<Move> = batch.candidates.into_iter().take(6).collect();
        let plan = prepare_2ply_expectimax_leaves(&state, &pre_pruned, 3);

        assert_eq!(plan.flat_tensor.len(), plan.batch_size * TENSOR_SIZE);

        struct MockEval;
        impl Evaluator for MockEval {
            fn evaluate(&self, state: &crate::types::Afterstate, feat: &crate::types::TopoFeatures) -> f32 {
                HeuristicCandidatePolicy.evaluate(state, feat)
            }
            fn policy_top_k(&self, state: &GameState, moves: &[Move], k: usize) -> Vec<Move> {
                HeuristicCandidatePolicy::policy_top_k(state, moves, k)
            }
        }
        let res = evaluate_2ply_expectimax_with_evaluator(&state, DiceRoll(2, 3), 24, 6, 3, &MockEval);
        assert!(res.is_some());
    }

    #[test]
    fn test_xhigh_whale_nnue_komi_opening_and_arbitration() {
        let nnue = NnueModel::from_bytes(TEST_NNUE_BYTES).expect("Valid whale_3_0.nnue");
        let mut bot = WhaleBot::default();

        let p2_opening = GameState {
            my_bb: Bitboard::new(),
            opp_bb: rect_mask(0, 0, 3, 4),
            my_score: 18,
            opp_score: 12,
            turn: 1,
        };
        let (mv, score) = bot
            .select_move_with_score(&p2_opening, &nnue, 2, 5)
            .expect("P2 should find legal opening move with komi=18");
        assert_eq!(mv.x as usize + mv.w as usize, WIDTH);
        assert_eq!(mv.y as usize + mv.h as usize, HEIGHT);
        assert!(score.is_finite());

        let cand_a = Move { x: 4, y: 0, w: 3, h: 2 };
        let cand_b = Move { x: 0, y: 3, w: 2, h: 3 };
        let mid_state = GameState {
            my_bb: rect_mask(0, 0, 4, 3),
            opp_bb: rect_mask(35, 12, 5, 3),
            my_score: 12,
            opp_score: 33,
            turn: 2,
        };
        let consensus = xhigh_arbitrate(
            &mid_state,
            &[(cand_a, 0.72), (cand_b, 0.61)],
            (cand_a, 150.0),
            &[(cand_a, 150.0), (cand_b, 120.0)],
        );
        assert_eq!(consensus, cand_a);

        let mut p1_bb = Bitboard::new();
        for y in 0..=4 {
            p1_bb.set(10, y);
            p1_bb.set(24, y);
        }
        for x in 10..=22 {
            p1_bb.set(x, 4);
        }
        let mut p2_bb = Bitboard::new();
        p2_bb.set(39, 14);
        let danger_state = GameState {
            my_bb: p1_bb,
            opp_bb: p2_bb,
            my_score: 250,
            opp_score: 295,
            turn: 10,
        };
        let win_mv = Move { x: 23, y: 4, w: 1, h: 1 };
        let unsafe_mv = Move { x: 9, y: 0, w: 1, h: 1 };
        let chosen = xhigh_arbitrate(
            &danger_state,
            &[(unsafe_mv, 0.85), (win_mv, 0.80)],
            (win_mv, 200.0),
            &[(unsafe_mv, 190.0), (win_mv, 200.0)],
        );
        assert_eq!(
            chosen, win_mv,
            "Tactical safety must veto unsafe move allowing immediate opponent win"
        );

        let (chosen_rev, breakdown_rev) = xhigh_arbitrate_detailed(
            &danger_state,
            &[(win_mv, 0.78), (unsafe_mv, 0.72)],
            &[(win_mv, 0.78), (unsafe_mv, 0.72)],
            (unsafe_mv, 250.0),
            &[(win_mv, 150.0), (unsafe_mv, 250.0)],
        );
        assert_eq!(
            chosen_rev, win_mv,
            "Tactical safety must still veto unsafe_mv even when Whale rates it higher"
        );
        assert_eq!(breakdown_rev.status, "VETOED");
        assert!(breakdown_rev.veto_reason.is_some());
    }

    #[test]
    fn test_preview_capture_restore_and_telemetry() {
        let mut p1_bb = Bitboard::new();
        for y in 0..=3 {
            p1_bb.set(10, y);
            p1_bb.set(15, y);
        }
        for x in 10..=13 {
            p1_bb.set(x, 3);
        }
        let mut p2_bb = Bitboard::new();
        p2_bb.set(39, 14);

        let mut rt = EngineRuntime::new_with_komi(18);
        rt.state = GameState {
            my_bb: p1_bb,
            opp_bb: p2_bb,
            my_score: p1_bb.count_ones(),
            opp_score: 19,
            turn: 4,
        };

        let score_before = rt.state.my_score;
        let preview = preview_capture_core(&rt, 14, 3, 1, 1);
        assert_eq!(preview.len(), 12);
        assert_eq!(rt.state.my_score, score_before);
        assert!(preview_capture_core(&rt, 0, 10, 2, 2).is_empty());

        let insp = inspect_position_core(&rt);
        assert_eq!(insp.channels_flat.len(), TENSOR_SIZE);
        assert_eq!(insp.risk_map.len(), NUM_CELLS);
        assert_eq!(insp.perspective_player, 1);

        let dto_before = build_state_dto(&rt);
        let mut rt2 = EngineRuntime::new_with_komi(0);
        let restored = restore_state_core(
            &mut rt2,
            &dto_before.grid,
            dto_before.p1_score,
            dto_before.p2_score,
            dto_before.turn,
            0,
            dto_before.komi,
        )
        .expect("Valid restore");
        assert_eq!(restored.grid, dto_before.grid);
        assert_eq!(restored.p1_score, dto_before.p1_score);
        assert_eq!(restored.p2_score, dto_before.p2_score);
        assert_eq!(restored.komi, 18);

        let closing = Move { x: 14, y: 3, w: 1, h: 1 };
        let tel = build_telemetry(&rt.state, Some(closing), 1, &[(closing, 0.88)], None, None);
        assert_eq!(tel.channels_flat.len(), TENSOR_SIZE);
        assert!((tel.eval_win_prob - 0.88).abs() < 1e-6);
        assert_eq!(tel.candidates.len(), 1);
        assert_eq!(tel.candidates[0].capture_cells, 12);
        assert_eq!(tel.candidates[0].capture_bonus, 480);
    }
}
