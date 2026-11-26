import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

const ROWS = 15;
const COLS = 40;

export function useDevRecorder({
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
}) {
  const [devOpen, setDevOpen] = useState(false);
  const [devComp, setDevComp] = useState(null);
  const [devFrames, setDevFrames] = useState([]);
  const [devInspectIdx, setDevInspectIdx] = useState(null);
  const [devChannel, setDevChannel] = useState('off');
  const [devHoverCandidate, setDevHoverCandidate] = useState(null);
  const [devCandCaptureCells, setDevCandCaptureCells] = useState([]);
  const [devHoverReadout, setDevHoverReadout] = useState(null);
  const [devDiceOverride, setDevDiceOverride] = useState({ d1: 3, d2: 4, locked: false });
  const devOpenRef = useRef(false);
  const devDiceOverrideRef = useRef({ d1: 3, d2: 4, locked: false });

  useEffect(() => {
    if (!import.meta.env.DEV) return;
    devOpenRef.current = devOpen;
  }, [devOpen]);

  useEffect(() => {
    if (!import.meta.env.DEV) return;
    devDiceOverrideRef.current = devDiceOverride;
  }, [devDiceOverride]);

  const recordDevFrame = useCallback((entry) => {
    if (!import.meta.env.DEV) return;
    setDevFrames((prev) => {
      const idx = prev.length;
      return [...prev, { frameIndex: idx, timestamp: Date.now(), ...entry }];
    });
  }, []);

  const attachDevInspectToFrame = useCallback(
    async (frameIndex) => {
      if (!import.meta.env.DEV || !devOpenRef.current) return;
      try {
        const insp = await callWorker('INSPECT_POSITION');
        setDevFrames((prev) =>
          prev.map((f) => (f.frameIndex === frameIndex ? { ...f, debugTrace: f.debugTrace || insp } : f)),
        );
      } catch {

      }
    },
    [callWorker],
  );

  const toggleDevPanel = useCallback(
    async (forceOpen) => {
      if (!import.meta.env.DEV) return;
      const loader = Object.values(devPanelLoader)[0];
      if (!loader) return;
      setDevOpen((prev) => {
        const next = forceOpen !== undefined ? forceOpen : !prev;
        devOpenRef.current = next;
        return next;
      });
      if (!devComp) {
        const mod = await loader();
        setDevComp(() => mod.default);
        setDevFrames((prev) => {
          if (prev.length > 0 && !prev[prev.length - 1].debugTrace) {
            attachDevInspectToFrame(prev.length - 1);
          }
          return prev;
        });
      }
    },
    [attachDevInspectToFrame, devComp, devPanelLoader],
  );

  useEffect(() => {
    if (!import.meta.env.DEV) return;
    const params = new URLSearchParams(window.location.search);
    if (params.get('admin') === '1') {
      toggleDevPanel(true);
    }
  }, [toggleDevPanel]);

  useEffect(() => {
    if (!import.meta.env.DEV || !devHoverCandidate) {
      setDevCandCaptureCells([]);
      return;
    }
    let cancelled = false;
    callWorker('PREVIEW_CAPTURE', {
      x: devHoverCandidate.x,
      y: devHoverCandidate.y,
      w: devHoverCandidate.w,
      h: devHoverCandidate.h,
    })
      .then((res) => {
        if (!cancelled) setDevCandCaptureCells(res.cells || []);
      })
      .catch(() => {
        if (!cancelled) setDevCandCaptureCells([]);
      });
    return () => {
      cancelled = true;
    };
  }, [callWorker, devHoverCandidate]);

  const handleRestoreToFrame = useCallback(
    async (targetIdx, branchFrames) => {
      if (!import.meta.env.DEV) return;
      const frame = branchFrames[targetIdx];
      if (!frame) return;
      const epoch = ++matchEpochRef.current;
      const grid = frame.boardCells || frame.boardState?.grid;
      if (!grid) return;

      const restored = await callWorker('RESTORE_STATE', {
        grid,
        p1Score: frame.scores?.p1 ?? frame.boardState?.p1_score ?? 0,
        p2Score: frame.scores?.p2 ?? frame.boardState?.p2_score ?? 18,
        turn: frame.turn ?? frame.boardState?.turn ?? 0,
        consecutivePasses: frame.consecutivePasses ?? frame.boardState?.consecutive_passes ?? 0,
        komi: frame.komi ?? komi,
      });
      if (matchEpochRef.current !== epoch) return;

      setDevFrames(branchFrames.slice(0, targetIdx + 1));
      setDevInspectIdx(null);
      setGameState(restored);
      setLastOpponentMove(frame.move || null);
      setCaptureAnim(null);
      setOverlayDismissed(false);

      if (restored.game_over) {
        applyGameOverStatus(restored, mode, humanSide);
        return;
      }

      if (mode === 'vs_ai' && restored.active_player !== humanSide) {
        await runBotTurnRef.current(restored.active_player, epoch, humanSide);
      } else {
        await prepareHumanTurnRef.current(epoch, mode, restored.active_player);
      }
    },
    [
      applyGameOverStatus,
      callWorker,
      humanSide,
      komi,
      matchEpochRef,
      mode,
      prepareHumanTurnRef,
      runBotTurnRef,
      setCaptureAnim,
      setGameState,
      setLastOpponentMove,
      setOverlayDismissed,
    ],
  );

  const handleUndoStep = useCallback(async () => {
    if (!import.meta.env.DEV || devFrames.length <= 1) return;
    await handleRestoreToFrame(devFrames.length - 2, devFrames);
  }, [devFrames, handleRestoreToFrame]);

  const handleBranchFromFrame = useCallback(
    async (frame) => {
      if (!import.meta.env.DEV || !frame) return;
      await handleRestoreToFrame(frame.frameIndex, devFrames);
    },
    [devFrames, handleRestoreToFrame],
  );

  const handleForceBotMove = useCallback(async () => {
    if (!import.meta.env.DEV || !gameState || gameState.game_over) return;
    const epoch = ++matchEpochRef.current;
    await runBotTurnRef.current(gameState.active_player, epoch, humanSide);
  }, [gameState, humanSide, matchEpochRef, runBotTurnRef]);

  const handleLoadTraceFrames = useCallback(
    (loadedFrames, loadedSettings) => {
      if (!import.meta.env.DEV) return;
      const normalized = loadedFrames.map((f, idx) => ({ ...f, frameIndex: idx }));
      setDevFrames(normalized);
      setDevInspectIdx(normalized.length > 0 ? 0 : null);
      if (loadedSettings?.komi !== undefined) {
        setKomi(loadedSettings.komi);
      }
    },
    [setKomi],
  );

  const resetDevMatchFrames = useCallback(
    (state) => {
      if (!import.meta.env.DEV) return;
      setDevInspectIdx(null);
      setDevFrames([
        {
          frameIndex: 0,
          timestamp: Date.now(),
          turn: state.turn,
          activePlayer: state.active_player,
          movedPlayer: null,
          dice: [0, 0],
          effort: effortRef.current,
          move: null,
          passed: false,
          capturedCells: [],
          scores: { p1: state.p1_score, p2: state.p2_score },
          boardCells: state.grid,
          boardState: state,
          komi: state.komi,
          consecutivePasses: state.consecutive_passes,
          debugTrace: null,
        },
      ]);
      attachDevInspectToFrame(0);
    },
    [attachDevInspectToFrame, effortRef],
  );

  const isReplayInspecting =
    import.meta.env.DEV && devOpen && devInspectIdx !== null && devInspectIdx >= 0 && devInspectIdx < devFrames.length;
  const displayedFrame =
    import.meta.env.DEV && devOpen ? devFrames[isReplayInspecting ? devInspectIdx : devFrames.length - 1] || null : null;
  const displayedState = isReplayInspecting && displayedFrame?.boardState ? displayedFrame.boardState : gameState;
  const displayedGrid = isReplayInspecting ? displayedFrame?.boardCells || displayedFrame?.boardState?.grid : gameState?.grid;

  const activeDevChannelData = useMemo(() => {
    if (!import.meta.env.DEV || !devOpen || devChannel === 'off' || !displayedFrame?.debugTrace) {
      return null;
    }
    const tel = displayedFrame.debugTrace;
    if (devChannel === 'risk') {
      return tel.riskMap && tel.riskMap.length === ROWS * COLS ? tel.riskMap : null;
    }
    const chIdx = Number(devChannel);
    if (!Number.isInteger(chIdx) || chIdx < 0 || chIdx > 7 || !tel.inputTensor) return null;
    const start = chIdx * ROWS * COLS;
    return tel.inputTensor.length >= start + ROWS * COLS ? tel.inputTensor.slice(start, start + ROWS * COLS) : null;
  }, [devChannel, devOpen, displayedFrame]);

  return {
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
  };
}
