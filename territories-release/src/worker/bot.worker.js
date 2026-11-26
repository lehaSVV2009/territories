import * as ort from 'onnxruntime-web/wasm';
import initWasm, {
  init_model,
  init_nnue,
  is_nnue_loaded,
  new_game,
  get_legal_placements,
  has_any_legal_move,
  preview_move_capture,
  inspect_position,
  restore_state,
  pass_turn,
  apply_human_move,
  compute_bot_move,
} from '../pkg/territories_wasm.js';

let ortSession = null;
let nnueReady = false;
let nnueWarmPromise = null;
let msgQueue = Promise.resolve();

const VALID_EFFORTS = new Set(['low', 'med', 'high', 'xhigh']);

ort.env.wasm.numThreads = 1;
ort.env.wasm.simd = true;

globalThis.__territoriesOrtInit = async (bytes) => {
  const modelBytes = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  ortSession = await ort.InferenceSession.create(modelBytes, {
    executionProviders: ['wasm'],
    graphOptimizationLevel: 'all',
  });
};

globalThis.__territoriesOrtInfer = async (flatTensor, batchSize) => {
  if (!ortSession) {
    throw new Error('ONNX InferenceSession not initialized');
  }
  const inputData = new Float32Array(flatTensor);
  const inputTensor = new ort.Tensor('float32', inputData, [batchSize, 8, 15, 40]);
  const outputs = await ortSession.run({ input: inputTensor });
  const valueOutput = outputs.value || Object.values(outputs)[0];
  return new Float32Array(valueOutput.data);
};

async function ensureNnueReady() {
  if (nnueReady || is_nnue_loaded()) {
    nnueReady = true;
    return { nnueReady: true, alreadyLoaded: true, warmMs: 0 };
  }
  if (!nnueWarmPromise) {
    nnueWarmPromise = (async () => {
      const t0 = performance.now();
      const resp = await fetch(`${import.meta.env.BASE_URL}whale_3_0.nnue`);
      if (!resp.ok) {
        throw new Error(`Failed to fetch whale_3_0.nnue: HTTP ${resp.status}`);
      }
      const buf = await resp.arrayBuffer();
      await Promise.resolve(init_nnue(new Uint8Array(buf)));
      nnueReady = true;
      return {
        nnueReady: true,
        alreadyLoaded: false,
        warmMs: Math.round(performance.now() - t0),
      };
    })();
  }
  try {
    return await nnueWarmPromise;
  } catch (err) {
    nnueWarmPromise = null;
    throw err;
  }
}

function formatMoveResponse(res, elapsedMs, includeDevTrace) {
  const data = {
    move: res.last_move ?? null,
    boardState: res.state,
    capturedCells: res.captured_cells ?? [],
    scores: {
      p1: res.state.p1_score,
      p2: res.state.p2_score,
    },
    winner: res.state.winner,
    result: res,
  };
  if (elapsedMs !== undefined) {
    data.elapsedMs = elapsedMs;
    data.latencyMs = elapsedMs;
  }
  if (import.meta.env.DEV && includeDevTrace && res.telemetry) {
    const tel = res.telemetry;
    data.debugTrace = {
      inputTensor: Array.from(tel.channels_flat || []),
      channelLabels: [
        'Ch0: Active Player',
        'Ch1: Opponent',
        'Ch2: Perimeter',
        'Ch3: Pocket Size',
        'Ch4: Seal (Active)',
        'Ch5: Seal (Opp)',
        'Ch6: Score Norm',
        'Ch7: Elasticity',
      ],
      riskMap: Array.from(tel.risk_map || []),
      evalWinProb: tel.eval_win_prob,
      perspectivePlayer: tel.perspective_player,
      candidates: (tel.candidates || []).map((c) => ({
        x: c.x,
        y: c.y,
        w: c.w,
        h: c.h,
        heuristicScore: c.heuristic_score,
        loopCaptureCells: c.capture_cells,
        loopCaptureBonus: c.capture_bonus,
        cnnP1: c.p1_prob ?? null,
        cnnP2: c.p2_prob ?? null,
        whaleScore: c.aux_score ?? null,
        whaleSigmoid: c.aux_sigmoid ?? null,
        vetoed: Boolean(c.vetoed),
        penalty: c.penalty ?? 0,
        finalScore: c.final_score,
        selected: Boolean(c.selected),
      })),
      arbitration: tel.arbitration
        ? {
            status: tel.arbitration.status,
            cnnTopMove: tel.arbitration.primary_move ?? null,
            whaleTopMove: tel.arbitration.aux_move ?? null,
            vetoReason: tel.arbitration.veto_reason ?? null,
          }
        : null,
    };
  }
  return data;
}

async function handleMessage({ id, type, payload = {} }) {
  try {
    switch (type) {
      case 'INIT': {
        const t0 = performance.now();
        await initWasm();
        const resp = await fetch(`${import.meta.env.BASE_URL}bot.onnx`);
        if (!resp.ok) {
          throw new Error(`Failed to fetch bot.onnx: HTTP ${resp.status}`);
        }
        const buf = await resp.arrayBuffer();
        await init_model(new Uint8Array(buf));
        const initMs = Math.round(performance.now() - t0);
        self.postMessage({ id, ok: true, data: { initMs } });
        break;
      }
      case 'WARM_XHIGH': {
        const warm = await ensureNnueReady();
        self.postMessage({ id, ok: true, data: warm });
        break;
      }
      case 'NEW_GAME': {
        const komi = payload?.komi ?? 18;
        const state = new_game(komi);
        self.postMessage({ id, ok: true, data: state });
        break;
      }
      case 'GET_LEGAL': {
        const placements = get_legal_placements(payload.w, payload.h);
        const hasAny = has_any_legal_move(payload.d1, payload.d2);
        const captureMap = {};
        for (const p of placements) {
          const cells = preview_move_capture(p.x, p.y, p.w, p.h);
          if (cells && cells.length > 0) {
            captureMap[p.y * 40 + p.x] = Array.from(cells);
          }
        }
        self.postMessage({ id, ok: true, data: { placements, hasAny, captureMap } });
        break;
      }
      case 'HUMAN_MOVE': {
        const res = apply_human_move(payload.x, payload.y, payload.w, payload.h);
        self.postMessage({ id, ok: true, data: formatMoveResponse(res, undefined, false) });
        break;
      }
      case 'PASS_TURN': {
        const res = pass_turn();
        self.postMessage({ id, ok: true, data: formatMoveResponse(res, undefined, false) });
        break;
      }
      case 'BOT_MOVE': {
        const effort = typeof payload?.effort === 'string' ? payload.effort.toLowerCase() : 'med';
        if (!VALID_EFFORTS.has(effort)) {
          throw new Error(`Invalid effort level: ${effort}`);
        }
        const wantTrace = import.meta.env.DEV ? Boolean(payload?.includeTrace) : false;
        const t0 = performance.now();
        if (effort === 'xhigh') {
          await ensureNnueReady();
        }
        const res = await compute_bot_move(payload.d1, payload.d2, effort, wantTrace);
        const elapsedMs = Math.round(performance.now() - t0);
        self.postMessage({ id, ok: true, data: formatMoveResponse(res, elapsedMs, wantTrace) });
        break;
      }
      case 'INSPECT_POSITION': {
        if (!import.meta.env.DEV) {
          throw new Error('Disabled in production');
        }
        const insp = inspect_position();
        const inputTensor = Array.from(insp.channels_flat || []);
        let evalWinProb = 0.5;
        if (ortSession && inputTensor.length === 4800) {
          const out = await globalThis.__territoriesOrtInfer(inputTensor, 1);
          const pActive = Number(out[0] ?? 0.5);
          evalWinProb = insp.perspective_player === 1 ? pActive : 1.0 - pActive;
        }
        self.postMessage({
          id,
          ok: true,
          data: {
            inputTensor,
            channelLabels: [
              'Ch0: Active Player',
              'Ch1: Opponent',
              'Ch2: Perimeter',
              'Ch3: Pocket Size',
              'Ch4: Seal (Active)',
              'Ch5: Seal (Opp)',
              'Ch6: Score Norm',
              'Ch7: Elasticity',
            ],
            riskMap: Array.from(insp.risk_map || []),
            evalWinProb,
            perspectivePlayer: insp.perspective_player,
          },
        });
        break;
      }
      case 'PREVIEW_CAPTURE': {
        if (!import.meta.env.DEV) {
          throw new Error('Disabled in production');
        }
        const cells = preview_move_capture(payload.x, payload.y, payload.w, payload.h);
        self.postMessage({ id, ok: true, data: { cells: Array.from(cells || []) } });
        break;
      }
      case 'RESTORE_STATE': {
        if (!import.meta.env.DEV) {
          throw new Error('Disabled in production');
        }
        const restored = restore_state(
          new Uint8Array(payload.grid),
          payload.p1Score ?? 0,
          payload.p2Score ?? 0,
          payload.turn ?? 0,
          payload.consecutivePasses ?? 0,
          payload.komi ?? 18,
        );
        self.postMessage({ id, ok: true, data: restored });
        break;
      }
      default:
        throw new Error(`Unknown worker message type: ${type}`);
    }
  } catch (err) {
    self.postMessage({
      id,
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
}

self.onmessage = (event) => {
  msgQueue = msgQueue.then(() => handleMessage(event.data));
};
