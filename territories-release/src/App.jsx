import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { EFFORTS, formatLatency, t as getTranslations } from './i18n';
import { useSoundEffects } from './audio';
import BoardCanvas from './components/BoardCanvas';
import MatchDialogs from './components/MatchDialogs';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useDevRecorder } from './hooks/useDevRecorder';

const ROWS = 15;
const COLS = 40;
const WIN_SCORE = 301;
const devPanelLoader = import.meta.env.DEV ? import.meta.glob('./admin.local/AdminStudio.jsx') : {};

function rollDice() {
  return [1 + Math.floor(Math.random() * 6), 1 + Math.floor(Math.random() * 6)];
}

function DieFace({ value }) {
  const pips = {
    1: [[16, 16]],
    2: [
      [9, 9],
      [23, 23],
    ],
    3: [
      [9, 9],
      [16, 16],
      [23, 23],
    ],
    4: [
      [9, 9],
      [23, 9],
      [9, 23],
      [23, 23],
    ],
    5: [
      [9, 9],
      [23, 9],
      [16, 16],
      [9, 23],
      [23, 23],
    ],
    6: [
      [9, 8],
      [23, 8],
      [9, 16],
      [23, 16],
      [9, 24],
      [23, 24],
    ],
  }[value] || [[16, 16]];

  return (
    <svg className="die-svg" viewBox="0 0 32 32" role="img" aria-label={`Die ${value}`}>
      {pips.map(([cx, cy], idx) => (
        <circle key={idx} cx={cx} cy={cy} r="2.5" fill="currentColor" />
      ))}
    </svg>
  );
}

export default function App() {
  const workerRef = useRef(null);
  const reqIdRef = useRef(0);
  const matchEpochRef = useRef(0);
  const pendingRef = useRef(new Map());
  const canvasRef = useRef(null);
  const menuDialogRef = useRef(null);
  const rulesDialogRef = useRef(null);
  const xhighWarmedRef = useRef(false);
  const prepareHumanTurnRef = useRef(null);
  const runBotTurnRef = useRef(null);

  const [engineReady, setEngineReady] = useState(false);
  const [loadError, setLoadError] = useState('');
  const [phase, setPhase] = useState('loading');
  const [gameState, setGameState] = useState(null);
  const [overlayDismissed, setOverlayDismissed] = useState(false);

  const [mode, setMode] = useState('vs_ai');
  const [humanSide, setHumanSide] = useState(1);
  const [komi, setKomi] = useState(18);
  const [effort, setEffort] = useState('med');
  const effortRef = useRef('med');
  useEffect(() => {
    effortRef.current = effort;
  }, [effort]);

  const [draftMode, setDraftMode] = useState('vs_ai');
  const [draftSide, setDraftSide] = useState(1);
  const [draftKomi, setDraftKomi] = useState(18);

  const [showAnchors, setShowAnchors] = useState(true);
  const [soundEnabled, setSoundEnabled] = useState(true);
  const [lang, setLang] = useState('en');
  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  const t = getTranslations(lang);
  const playTone = useSoundEffects(soundEnabled);

  const [dice, setDice] = useState([0, 0]);
  const [rotated, setRotated] = useState(false);
  const [legalPlacements, setLegalPlacements] = useState([]);
  const [capturePreviewMap, setCapturePreviewMap] = useState(() => new Map());
  const [hasAnyLegal, setHasAnyLegal] = useState(true);
  const [hoverCell, setHoverCell] = useState(null);
  const [lastOpponentMove, setLastOpponentMove] = useState(null);
  const [captureAnim, setCaptureAnim] = useState(null);
  const [lastBotMs, setLastBotMs] = useState(null);
  const [xhighWarming, setXhighWarming] = useState(false);
  const [statusSpec, setStatusSpec] = useState({ key: 'initializing', args: [] });

  const statusText = useMemo(() => {
    const entry = t[statusSpec.key];
    if (typeof entry === 'function') return entry(...statusSpec.args);
    return entry || '';
  }, [t, statusSpec]);

  const callWorker = useCallback((type, payload = {}) => {
    return new Promise((resolve, reject) => {
      if (!workerRef.current) {
        reject(new Error('Worker not initialized'));
        return;
      }
      const id = ++reqIdRef.current;
      pendingRef.current.set(id, { resolve, reject });
      workerRef.current.postMessage({ id, type, payload });
    });
  }, []);

  const ensureXhighWarmed = useCallback(async () => {
    if (xhighWarmedRef.current) return;
    setXhighWarming(true);
    try {
      await callWorker('WARM_XHIGH');
      xhighWarmedRef.current = true;
    } finally {
      setXhighWarming(false);
    }
  }, [callWorker]);

  const handleEffortSelect = useCallback(
    (nextEffort) => {
      setEffort(nextEffort);
      effortRef.current = nextEffort;
      if (nextEffort === 'xhigh' && !xhighWarmedRef.current && engineReady) {
        ensureXhighWarmed().catch(() => {});
      }
    },
    [engineReady, ensureXhighWarmed],
  );

  const activePiece = useMemo(() => {
    const [d1, d2] = dice;
    if (d1 === 0 || d2 === 0) return { w: 0, h: 0 };
    return rotated ? { w: d2, h: d1 } : { w: d1, h: d2 };
  }, [dice, rotated]);

  const legalSet = useMemo(() => {
    const set = new Set();
    for (const p of legalPlacements) {
      set.add(p.y * COLS + p.x);
    }
    return set;
  }, [legalPlacements]);

  const triggerCaptureAnimation = useCallback((capturedCells, player) => {
    if (!capturedCells || capturedCells.length === 0) return;
    setCaptureAnim({
      cells: new Set(capturedCells),
      player,
      start: performance.now(),
    });
  }, []);

  const refreshHumanLegal = useCallback(
    async (d1, d2, isRotated) => {
      const w = isRotated ? d2 : d1;
      const h = isRotated ? d1 : d2;
      return await callWorker('GET_LEGAL', { w, h, d1, d2 });
    },
    [callWorker],
  );

  const applyGameOverStatus = useCallback(
    (state, matchMode, matchHumanSide) => {
      setPhase('game_over');
      setOverlayDismissed(false);
      setLegalPlacements([]);
      setHoverCell(null);
      playTone('gameover');

      const w = state.winner;
      if (w === 0) {
        setStatusSpec({ key: 'matchDrawn', args: [] });
      } else if (matchMode === '2p') {
        setStatusSpec({ key: 'playerWins', args: [w, state.p1_score, state.p2_score] });
      } else if (w === matchHumanSide) {
        setStatusSpec({ key: 'youWin', args: [state.p1_score, state.p2_score] });
      } else {
        setStatusSpec({ key: 'aiWins', args: [state.p1_score, state.p2_score] });
      }
    },
    [playTone],
  );

  const {
    devOpen,
    devComp,
    devFrames,
    devInspectIdx,
    setDevInspectIdx,
    devChannel,
    setDevChannel,
    devHoverCandidate,
    setDevHoverCandidate,
    devCandCaptureCells,
    devHoverReadout,
    setDevHoverReadout,
    devDiceOverride,
    setDevDiceOverride,
    devOpenRef,
    devDiceOverrideRef,
    recordDevFrame,
    attachDevInspectToFrame,
    toggleDevPanel,
    handleUndoStep,
    handleBranchFromFrame,
    handleForceBotMove,
    handleLoadTraceFrames,
    resetDevMatchFrames,
    isReplayInspecting,
    displayedFrame,
    displayedState,
    displayedGrid,
    activeDevChannelData,
  } = useDevRecorder({
    devPanelLoader,
    callWorker,
    matchEpochRef,
    effortRef,
    komi,
    setKomi,
    mode,
    humanSide,
    gameState,
    setGameState,
    setLastOpponentMove,
    setCaptureAnim,
    setOverlayDismissed,
    applyGameOverStatus,
    prepareHumanTurnRef,
    runBotTurnRef,
  });

  const effectiveRollDice = useCallback(() => {
    if (import.meta.env.DEV && devDiceOverrideRef.current?.locked) {
      return [devDiceOverrideRef.current.d1, devDiceOverrideRef.current.d2];
    }
    return rollDice();
  }, [devDiceOverrideRef]);

  const syncLegalPlacements = useCallback((legalRes) => {
    setLegalPlacements(legalRes.placements || []);
    setHasAnyLegal(Boolean(legalRes.hasAny));
    const map = new Map();
    if (legalRes.captureMap) {
      for (const [k, cells] of Object.entries(legalRes.captureMap)) {
        map.set(Number(k), cells);
      }
    }
    setCapturePreviewMap(map);
  }, []);

  const prepareHumanTurn = useCallback(
    async (epoch, matchMode, activePlayer, prefixCaptureCount = 0) => {
      const d = effectiveRollDice();
      setDice(d);
      setRotated(false);
      setPhase('human_turn');

      const legalRes = await refreshHumanLegal(d[0], d[1], false);
      if (matchEpochRef.current !== epoch) return;

      syncLegalPlacements(legalRes);

      if (!legalRes.hasAny) {
        setStatusSpec({ key: 'noLegalMoves', args: [d[0], d[1]] });
      } else if (matchMode === '2p') {
        setStatusSpec({ key: 'playerTurnPlace', args: [activePlayer, d[0], d[1]] });
      } else if (prefixCaptureCount > 0) {
        setStatusSpec({ key: 'aiCaptured', args: [prefixCaptureCount, d[0], d[1]] });
      } else {
        setStatusSpec({ key: 'yourTurnPlace', args: [d[0], d[1]] });
      }
    },
    [syncLegalPlacements, effectiveRollDice, refreshHumanLegal],
  );
  prepareHumanTurnRef.current = prepareHumanTurn;

  const runBotTurn = useCallback(
    async (botSide, epoch, matchHumanSide) => {
      if (matchEpochRef.current !== epoch) return;
      setPhase('bot_thinking');
      setLegalPlacements([]);
      setCapturePreviewMap(new Map());
      setHoverCell(null);

      const [d1, d2] = effectiveRollDice();
      setDice([d1, d2]);
      setRotated(false);
      setStatusSpec({ key: 'aiThinking', args: [botSide, d1, d2] });

      const chosenEffort = effortRef.current;
      if (chosenEffort === 'xhigh' && !xhighWarmedRef.current) {
        setXhighWarming(true);
      }

      const wantDevData = import.meta.env.DEV ? devOpenRef.current : false;
      let res;
      try {
        res = await callWorker('BOT_MOVE', { d1, d2, effort: chosenEffort, includeTrace: wantDevData });
      } finally {
        if (chosenEffort === 'xhigh') {
          xhighWarmedRef.current = true;
          setXhighWarming(false);
        }
      }

      if (matchEpochRef.current !== epoch) return;

      const moveResult = res.result || res;
      const state = res.boardState || moveResult.state;
      const captured = res.capturedCells || moveResult.captured_cells || [];
      const lastMove = res.move !== undefined ? res.move : moveResult.last_move || null;
      const movedPlayer = moveResult.moved_player || botSide;

      setLastBotMs(res.elapsedMs ?? res.latencyMs ?? null);
      setGameState(state);
      setLastOpponentMove(lastMove);

      if (import.meta.env.DEV) {
        setDevInspectIdx(null);
        recordDevFrame({
          turn: state.turn,
          activePlayer: state.active_player,
          movedPlayer,
          dice: [d1, d2],
          effort: chosenEffort,
          move: lastMove,
          passed: Boolean(moveResult.passed),
          capturedCells: captured,
          scores: { p1: state.p1_score, p2: state.p2_score },
          boardCells: state.grid,
          boardState: state,
          komi: state.komi,
          consecutivePasses: state.consecutive_passes,
          debugTrace: res.debugTrace || null,
        });
      }

      if (captured.length > 0) {
        triggerCaptureAnimation(captured, movedPlayer);
        playTone('capture');
      } else if (lastMove) {
        playTone('place');
      }

      if (state.game_over) {
        applyGameOverStatus(state, 'vs_ai', matchHumanSide);
        return;
      }

      await prepareHumanTurn(epoch, 'vs_ai', state.active_player, captured.length);
    },
    [
      applyGameOverStatus,
      callWorker,
      devOpenRef,
      effectiveRollDice,
      playTone,
      prepareHumanTurn,
      recordDevFrame,
      setDevInspectIdx,
      triggerCaptureAnimation,
    ],
  );
  runBotTurnRef.current = runBotTurn;

  const startMatch = useCallback(
    async (nextMode = mode, nextSide = humanSide, nextKomi = komi) => {
      const epoch = ++matchEpochRef.current;
      setMode(nextMode);
      setHumanSide(nextSide);
      setKomi(nextKomi);
      setDraftMode(nextMode);
      setDraftSide(nextSide);
      setDraftKomi(nextKomi);
      setOverlayDismissed(false);
      setLastOpponentMove(null);
      setCaptureAnim(null);
      setHoverCell(null);
      setLastBotMs(null);
      setLegalPlacements([]);
      setCapturePreviewMap(new Map());

      const state = await callWorker('NEW_GAME', { komi: nextKomi });
      if (matchEpochRef.current !== epoch) return;
      setGameState(state);

      if (import.meta.env.DEV) {
        resetDevMatchFrames(state);
      }

      if (nextMode === '2p' || nextSide === 1) {
        const d = effectiveRollDice();
        setDice(d);
        setRotated(false);
        setPhase('human_turn');
        const legalRes = await refreshHumanLegal(d[0], d[1], false);
        if (matchEpochRef.current !== epoch) return;
        syncLegalPlacements(legalRes);
        setStatusSpec({
          key: nextMode === '2p' ? 'playerTurnPlace' : 'yourTurnPlace',
          args: nextMode === '2p' ? [state.active_player, d[0], d[1]] : [d[0], d[1]],
        });
      } else {
        await runBotTurn(1, epoch, nextSide);
      }
    },
    [syncLegalPlacements, resetDevMatchFrames, callWorker, effectiveRollDice, humanSide, komi, mode, refreshHumanLegal, runBotTurn],
  );

  useEffect(() => {
    const worker = new Worker(new URL('./worker/bot.worker.js', import.meta.url), {
      type: 'module',
    });
    workerRef.current = worker;

    worker.onmessage = (event) => {
      const { id, ok, data, error } = event.data;
      const pending = pendingRef.current.get(id);
      if (!pending) return;
      pendingRef.current.delete(id);
      if (ok) {
        pending.resolve(data);
      } else {
        pending.reject(new Error(error));
      }
    };

    worker.onerror = (err) => {
      setLoadError(err.message || 'Worker error');
      setPhase('loading');
    };

    (async () => {
      try {
        await callWorker('INIT');
        setEngineReady(true);
        await startMatch('vs_ai', 1, 18);
      } catch (err) {
        setLoadError(err instanceof Error ? err.message : String(err));
      }
    })();

    return () => {
      matchEpochRef.current += 1;
      pendingRef.current.clear();
      worker.terminate();
      workerRef.current = null;
    };

  }, [callWorker]);

  const openMenu = useCallback(() => {
    setDraftMode(mode);
    setDraftSide(humanSide);
    setDraftKomi(komi);
    if (menuDialogRef.current && !menuDialogRef.current.open) {
      menuDialogRef.current.showModal();
    }
  }, [humanSide, komi, mode]);

  const openRules = useCallback(() => {
    if (rulesDialogRef.current && !rulesDialogRef.current.open) {
      rulesDialogRef.current.showModal();
    }
  }, []);

  const handleRotate = useCallback(async () => {
    if (phase !== 'human_turn' || dice[0] === dice[1]) return;
    const epoch = matchEpochRef.current;
    const nextRot = !rotated;
    setRotated(nextRot);
    const nextW = nextRot ? dice[1] : dice[0];
    const nextH = nextRot ? dice[0] : dice[1];
    setHoverCell((prev) =>
      prev ? { x: Math.min(prev.x, COLS - nextW), y: Math.min(prev.y, ROWS - nextH) } : null,
    );
    const res = await refreshHumanLegal(dice[0], dice[1], nextRot);
    if (matchEpochRef.current !== epoch) return;
    syncLegalPlacements(res);
  }, [syncLegalPlacements, phase, rotated, dice, refreshHumanLegal]);

  const handlePass = useCallback(async () => {
    if (phase !== 'human_turn' || hasAnyLegal) return;
    const epoch = matchEpochRef.current;
    const res = await callWorker('PASS_TURN');
    if (matchEpochRef.current !== epoch) return;

    const moveResult = res.result || res;
    const state = res.boardState || moveResult.state;
    setGameState(state);

    if (import.meta.env.DEV) {
      setDevInspectIdx(null);
      const nextIdx = devFrames.length;
      recordDevFrame({
        turn: state.turn,
        activePlayer: state.active_player,
        movedPlayer: moveResult.moved_player,
        dice: [...dice],
        effort: effortRef.current,
        move: null,
        passed: true,
        capturedCells: [],
        scores: { p1: state.p1_score, p2: state.p2_score },
        boardCells: state.grid,
        boardState: state,
        komi: state.komi,
        consecutivePasses: state.consecutive_passes,
        debugTrace: null,
      });
      attachDevInspectToFrame(nextIdx);
    }

    if (state.game_over) {
      applyGameOverStatus(state, mode, humanSide);
      return;
    }

    if (mode === '2p') {
      await prepareHumanTurn(epoch, '2p', state.active_player);
    } else {
      const botSide = humanSide === 1 ? 2 : 1;
      await runBotTurn(botSide, epoch, humanSide);
    }
  }, [
    applyGameOverStatus,
    attachDevInspectToFrame,
    callWorker,
    devFrames.length,
    dice,
    hasAnyLegal,
    humanSide,
    mode,
    phase,
    prepareHumanTurn,
    recordDevFrame,
    runBotTurn,
    setDevInspectIdx,
  ]);

  const handlePlaceAt = useCallback(
    async (x, y) => {
      if (phase !== 'human_turn') return;
      const { w, h } = activePiece;
      if (!legalSet.has(y * COLS + x)) {
        playTone('illegal');
        return;
      }

      const epoch = matchEpochRef.current;
      const res = await callWorker('HUMAN_MOVE', { x, y, w, h });
      if (matchEpochRef.current !== epoch) return;

      const moveResult = res.result || res;
      const state = res.boardState || moveResult.state;
      const captured = res.capturedCells || moveResult.captured_cells || [];
      const movedPlayer = moveResult.moved_player || gameState?.active_player || humanSide;
      const lastMove = res.move !== undefined ? res.move : moveResult.last_move || { x, y, w, h };

      setGameState(state);
      if (mode === '2p') {
        setLastOpponentMove(lastMove);
      }

      if (import.meta.env.DEV) {
        setDevInspectIdx(null);
        const nextIdx = devFrames.length;
        recordDevFrame({
          turn: state.turn,
          activePlayer: state.active_player,
          movedPlayer,
          dice: [...dice],
          effort: effortRef.current,
          move: lastMove,
          passed: false,
          capturedCells: captured,
          scores: { p1: state.p1_score, p2: state.p2_score },
          boardCells: state.grid,
          boardState: state,
          komi: state.komi,
          consecutivePasses: state.consecutive_passes,
          debugTrace: null,
        });
        attachDevInspectToFrame(nextIdx);
      }

      if (captured.length > 0) {
        triggerCaptureAnimation(captured, movedPlayer);
        playTone('capture');
      } else {
        playTone('place');
      }

      if (state.game_over) {
        applyGameOverStatus(state, mode, humanSide);
        return;
      }

      if (mode === '2p') {
        await prepareHumanTurn(epoch, '2p', state.active_player);
      } else {
        const botSide = humanSide === 1 ? 2 : 1;
        await runBotTurn(botSide, epoch, humanSide);
      }
    },
    [
      phase,
      activePiece,
      legalSet,
      callWorker,
      gameState,
      humanSide,
      mode,
      devFrames.length,
      dice,
      recordDevFrame,
      attachDevInspectToFrame,
      playTone,
      triggerCaptureAnimation,
      applyGameOverStatus,
      prepareHumanTurn,
      runBotTurn,
      setDevInspectIdx,
    ],
  );

  useKeyboardShortcuts({
    menuDialogRef,
    rulesDialogRef,
    canvasRef,
    engineReady,
    openMenu,
    toggleDevPanel,
    handleRotate,
    phase,
    mode,
    gameState,
    humanSide,
    activePiece,
    hoverCell,
    setHoverCell,
    handlePlaceAt,
  });

  const activePlayerSide = mode === '2p' ? gameState?.active_player || 1 : humanSide;

  const hoverCapturePreview = useMemo(() => {
    if (phase !== 'human_turn' || isReplayInspecting || !hoverCell) return null;
    const cells = capturePreviewMap.get(hoverCell.y * COLS + hoverCell.x);
    return cells && cells.length > 0 ? cells : null;
  }, [capturePreviewMap, hoverCell, isReplayInspecting, phase]);

  const p1Score = displayedState ? displayedState.p1_score : 0;
  const p2Score = displayedState ? displayedState.p2_score : komi;
  const p1Pct = Math.min(100, (p1Score / WIN_SCORE) * 100);
  const p2Pct = Math.min(100, (p2Score / WIN_SCORE) * 100);
  const activeKomi = gameState ? gameState.komi : komi;
  const matchInProgress = engineReady && phase !== 'game_over' && phase !== 'loading';

  const p1Label = mode === '2p' ? t.p1Short : humanSide === 1 ? t.you : t.ai;
  const p2Label = mode === '2p' ? t.p2Short : humanSide === 2 ? t.you : t.ai;
  const latencyLabel = formatLatency(lastBotMs, t);

  return (
    <div className="app-shell">
      <header className="topbar" data-testid="topbar">
        <div className="brand">
          <h1>{t.title}</h1>
          <span className="dim-tag">15×40</span>
        </div>

        <div className="scoreboard" data-testid="scoreboard" aria-label={t.scoreboardAria}>
          <div className="scoreboard-row">
            <div className={`score-pill ${gameState?.active_player === 1 && !gameState?.game_over ? 'active-p1' : ''}`}>
              <span className="swatch swatch-p1" aria-hidden="true" />
              <span className="score-label">{p1Label}</span>
              <span className="score-num" data-testid="p1-score">
                {p1Score}
              </span>
            </div>

            <span className="score-target">{t.target}</span>

            <div className={`score-pill ${gameState?.active_player === 2 && !gameState?.game_over ? 'active-p2' : ''}`}>
              <span className="swatch swatch-p2" aria-hidden="true" />
              <span className="score-label">{p2Label}</span>
              <span className="score-num" data-testid="p2-score">
                {p2Score}
              </span>
              {activeKomi > 0 && (
                <span className="komi-badge" data-testid="komi-badge">
                  +{activeKomi}
                </span>
              )}
            </div>
          </div>

          <div className="territory-progress" data-testid="territory-progress" aria-hidden="true">
            <div className="progress-half progress-half-p1">
              <div className="progress-fill-p1" style={{ width: `${p1Pct}%` }} />
            </div>
            <div className="progress-center-tick" />
            <div className="progress-half progress-half-p2">
              <div className="progress-fill-p2" style={{ width: `${p2Pct}%` }} />
            </div>
          </div>
        </div>

        <div className="topbar-actions">
          <button
            type="button"
            className="btn btn-subtle"
            data-testid="rules-btn"
            onClick={openRules}
          >
            {t.rules}
          </button>
          <button
            type="button"
            className="btn"
            data-testid="menu-btn"
            disabled={!engineReady}
            onClick={openMenu}
          >
            {t.menu}
          </button>
        </div>
      </header>

      <div className={`workspace-row ${import.meta.env.DEV && devOpen && devComp ? 'with-side' : ''}`}>
        <main className="board-stage">
          <BoardCanvas
            canvasRef={canvasRef}
            phase={phase}
            isReplayInspecting={isReplayInspecting}
            t={t}
            displayedGrid={displayedGrid}
            displayedFrame={displayedFrame}
            hoverCell={hoverCell}
            setHoverCell={setHoverCell}
            hoverCapturePreview={hoverCapturePreview}
            activePiece={activePiece}
            legalPlacements={legalPlacements}
            legalSet={legalSet}
            activePlayerSide={activePlayerSide}
            lastOpponentMove={lastOpponentMove}
            captureAnim={captureAnim}
            showAnchors={showAnchors}
            activeDevChannelData={activeDevChannelData}
            devOpen={devOpen}
            devHoverCandidate={devHoverCandidate}
            devCandCaptureCells={devCandCaptureCells}
            setDevHoverReadout={setDevHoverReadout}
            onPlaceAt={handlePlaceAt}
            onRotate={handleRotate}
            loadError={loadError}
            overlayDismissed={overlayDismissed}
            setOverlayDismissed={setOverlayDismissed}
            gameState={gameState}
            statusText={statusText}
            p1Label={p1Label}
            p2Label={p2Label}
            p1Score={p1Score}
            p2Score={p2Score}
            onOpenMenu={openMenu}
            onPlayAgain={() => startMatch(mode, humanSide, komi)}
          />
        </main>

        {import.meta.env.DEV &&
          devOpen &&
          devComp &&
          React.createElement(devComp, {
            frames: devFrames,
            inspectIndex: devInspectIdx,
            onSelectFrame: setDevInspectIdx,
            onBranchFromFrame: handleBranchFromFrame,
            selectedChannel: devChannel,
            onSelectChannel: setDevChannel,
            hoverReadout: devHoverReadout,
            onHoverCandidate: setDevHoverCandidate,
            diceOverride: devDiceOverride,
            onChangeDiceOverride: setDevDiceOverride,
            onForceBotMove: handleForceBotMove,
            onUndoStep: handleUndoStep,
            onLoadTraceFrames: handleLoadTraceFrames,
            onClose: () => toggleDevPanel(false),
            currentSettings: { komi, effort, mode, humanSide },
            isBusy: phase === 'bot_thinking' || phase === 'loading',
          })}
      </div>

      <footer className="control-dock" data-testid="control-dock">
        <div className="dock-left">
          <div className="dice-group">
            {dice[0] > 0 && (
              <>
                <span data-testid="dice-d1">
                  <DieFace value={dice[0]} />
                </span>
                <span data-testid="dice-d2">
                  <DieFace value={dice[1]} />
                </span>
                <span className="dice-dims" data-testid="active-dims">
                  {activePiece.w}×{activePiece.h}
                </span>
                {hoverCapturePreview && hoverCapturePreview.length > 0 && (
                  <span className="capture-preview-badge" data-testid="capture-preview-badge">
                    +{hoverCapturePreview.length}
                  </span>
                )}
              </>
            )}
          </div>

          <button
            type="button"
            className="btn"
            disabled={phase !== 'human_turn' || dice[0] === dice[1]}
            onClick={handleRotate}
            data-testid="rotate-btn"
          >
            {t.rotate} <span className="kbd-hint">R</span>
          </button>

          {phase === 'human_turn' && !hasAnyLegal && (
            <button
              type="button"
              className="btn btn-primary"
              onClick={handlePass}
              data-testid="pass-btn"
            >
              {t.passTurn}
            </button>
          )}
        </div>

        <div className="dock-center">
          <div className="turn-status" aria-live="polite" data-testid="status-text">
            {phase === 'bot_thinking' && <span className="thinking-dot" aria-hidden="true" />}
            <span>{statusText}</span>
          </div>
          {mode === 'vs_ai' && latencyLabel && (
            <span className="latency-pill" data-testid="bot-latency">
              {latencyLabel}
            </span>
          )}
          {mode === 'vs_ai' && xhighWarming && (
            <span className="warming-pill" data-testid="xhigh-warming">
              {t.warmingXhigh}
            </span>
          )}
        </div>

        <div className="dock-right">
          {mode === 'vs_ai' && (
            <div
              className="segmented-pill"
              role="group"
              aria-label={t.effortGroupAria}
              data-testid="effort-pill"
            >
              {EFFORTS.map((tier) => (
                <button
                  key={tier}
                  type="button"
                  className="segment-btn"
                  aria-pressed={effort === tier}
                  data-testid={`effort-${tier}`}
                  onClick={() => handleEffortSelect(tier)}
                >
                  {t.effortLabels[tier]}
                </button>
              ))}
            </div>
          )}
        </div>
      </footer>

      <MatchDialogs
        menuDialogRef={menuDialogRef}
        rulesDialogRef={rulesDialogRef}
        t={t}
        draftMode={draftMode}
        setDraftMode={setDraftMode}
        draftSide={draftSide}
        setDraftSide={setDraftSide}
        draftKomi={draftKomi}
        setDraftKomi={setDraftKomi}
        effort={effort}
        onEffortSelect={handleEffortSelect}
        showAnchors={showAnchors}
        setShowAnchors={setShowAnchors}
        soundEnabled={soundEnabled}
        setSoundEnabled={setSoundEnabled}
        lang={lang}
        setLang={setLang}
        matchInProgress={matchInProgress}
        onStartMatch={startMatch}
      />
    </div>
  );
}
