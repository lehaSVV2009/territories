import React, { useEffect } from 'react';

const ROWS = 15;
const COLS = 40;

export function drawClusterPerimeter(ctx, cellSet, cellW, cellH, strokeStyle, lineWidth = 2, dash = []) {
  if (!cellSet || cellSet.size === 0) return;
  ctx.save();
  ctx.strokeStyle = strokeStyle;
  ctx.lineWidth = lineWidth;
  ctx.setLineDash(dash);
  ctx.beginPath();
  for (const idx of cellSet) {
    const x = idx % COLS;
    const y = Math.floor(idx / COLS);
    const x0 = x * cellW;
    const y0 = y * cellH;
    const x1 = x0 + cellW;
    const y1 = y0 + cellH;
    if (y === 0 || !cellSet.has((y - 1) * COLS + x)) {
      ctx.moveTo(x0, y0);
      ctx.lineTo(x1, y0);
    }
    if (y === ROWS - 1 || !cellSet.has((y + 1) * COLS + x)) {
      ctx.moveTo(x0, y1);
      ctx.lineTo(x1, y1);
    }
    if (x === 0 || !cellSet.has(y * COLS + (x - 1))) {
      ctx.moveTo(x0, y0);
      ctx.lineTo(x0, y1);
    }
    if (x === COLS - 1 || !cellSet.has(y * COLS + (x + 1))) {
      ctx.moveTo(x1, y0);
      ctx.lineTo(x1, y1);
    }
  }
  ctx.stroke();
  ctx.restore();
}

export default function BoardCanvas({
  canvasRef,
  phase,
  isReplayInspecting,
  t,
  displayedGrid,
  displayedFrame,
  hoverCell,
  setHoverCell,
  hoverCapturePreview,
  activePiece,
  legalPlacements,
  legalSet,
  activePlayerSide,
  lastOpponentMove,
  captureAnim,
  showAnchors,
  activeDevChannelData,
  devOpen,
  devHoverCandidate,
  devCandCaptureCells,
  setDevHoverReadout,
  onPlaceAt,
  onRotate,
  loadError,
  overlayDismissed,
  setOverlayDismissed,
  gameState,
  statusText,
  p1Label,
  p2Label,
  p1Score,
  p2Score,
  onOpenMenu,
  onPlayAgain,
}) {
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let animFrame = 0;

    const draw = () => {
      const rect = canvas.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;

      const dpr = window.devicePixelRatio || 1;
      const targetW = Math.round(rect.width * dpr);
      const targetH = Math.round(rect.height * dpr);
      if (canvas.width !== targetW || canvas.height !== targetH) {
        canvas.width = targetW;
        canvas.height = targetH;
      }

      const ctx = canvas.getContext('2d');
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const bg = '#0f1218';
      const gridLine = 'rgba(255, 255, 255, 0.075)';
      const p1Color = '#4c8bf5';
      const p2Color = '#f29f24';
      const dangerColor = '#f87171';
      const captureColor = '#34d399';
      const ringColor = '#eef1f6';

      const cellW = rect.width / COLS;
      const cellH = rect.height / ROWS;

      ctx.fillStyle = bg;
      ctx.fillRect(0, 0, rect.width, rect.height);

      if (displayedGrid) {
        for (let y = 0; y < ROWS; y++) {
          for (let x = 0; x < COLS; x++) {
            const val = displayedGrid[y * COLS + x];
            if (val === 1 || val === 2) {
              ctx.fillStyle = val === 1 ? p1Color : p2Color;
              ctx.fillRect(x * cellW, y * cellH, cellW + 0.35, cellH + 0.35);
            }
          }
        }
      }

      if (displayedGrid) {
        if (displayedGrid[0] === 0) {
          ctx.fillStyle = p1Color;
          ctx.globalAlpha = 0.25;
          ctx.fillRect(0, 0, cellW, cellH);
          ctx.globalAlpha = 0.85;
          ctx.strokeStyle = p1Color;
          ctx.lineWidth = 1.5;
          ctx.strokeRect(1, 1, cellW - 2, cellH - 2);
          ctx.globalAlpha = 1;
        }
        if (displayedGrid[ROWS * COLS - 1] === 0) {
          ctx.fillStyle = p2Color;
          ctx.globalAlpha = 0.25;
          ctx.fillRect((COLS - 1) * cellW, (ROWS - 1) * cellH, cellW, cellH);
          ctx.globalAlpha = 0.85;
          ctx.strokeStyle = p2Color;
          ctx.lineWidth = 1.5;
          ctx.strokeRect((COLS - 1) * cellW + 1, (ROWS - 1) * cellH + 1, cellW - 2, cellH - 2);
          ctx.globalAlpha = 1;
        }
      }

      let keepAnimating = false;
      if (captureAnim && !isReplayInspecting) {
        const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        const elapsed = performance.now() - captureAnim.start;
        const duration = reducedMotion ? 0 : 500;
        if (elapsed < duration) {
          keepAnimating = true;
          const tNorm = elapsed / duration;
          const fillAlpha = Math.sin(tNorm * Math.PI) * 0.42;
          ctx.fillStyle = '#ffffff';
          ctx.globalAlpha = fillAlpha;
          for (const idx of captureAnim.cells) {
            const cx = idx % COLS;
            const cy = Math.floor(idx / COLS);
            ctx.fillRect(cx * cellW, cy * cellH, cellW, cellH);
          }
          ctx.globalAlpha = Math.max(0.2, (1 - tNorm) * 0.95);
          drawClusterPerimeter(
            ctx,
            captureAnim.cells,
            cellW,
            cellH,
            '#ffffff',
            2.5,
            [],
          );
          ctx.globalAlpha = 1;
        }
      }

      if (import.meta.env.DEV && activeDevChannelData) {
        for (let i = 0; i < ROWS * COLS; i++) {
          const val = Math.max(0, Math.min(1, Number(activeDevChannelData[i] || 0)));
          if (val <= 0.005) continue;
          const x = i % COLS;
          const y = Math.floor(i / COLS);
          const hue = 195 - val * 175;
          ctx.fillStyle = `hsla(${hue.toFixed(0)}, 90%, 56%, ${(0.18 + val * 0.56).toFixed(2)})`;
          ctx.fillRect(x * cellW, y * cellH, cellW, cellH);
        }
      }

      ctx.strokeStyle = gridLine;
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x = 1; x < COLS; x++) {
        const px = Math.round(x * cellW) + 0.5;
        ctx.moveTo(px, 0);
        ctx.lineTo(px, rect.height);
      }
      for (let y = 1; y < ROWS; y++) {
        const py = Math.round(y * cellH) + 0.5;
        ctx.moveTo(0, py);
        ctx.lineTo(rect.width, py);
      }
      ctx.stroke();

      const shownLastMove = isReplayInspecting ? displayedFrame?.move : lastOpponentMove;
      if (shownLastMove) {
        ctx.strokeStyle = ringColor;
        ctx.lineWidth = 2;
        ctx.strokeRect(
          shownLastMove.x * cellW + 1,
          shownLastMove.y * cellH + 1,
          shownLastMove.w * cellW - 2,
          shownLastMove.h * cellH - 2,
        );
      }

      if (!isReplayInspecting && showAnchors && phase === 'human_turn' && legalPlacements.length > 0) {
        ctx.fillStyle = activePlayerSide === 1 ? p1Color : p2Color;
        ctx.globalAlpha = 0.72;
        const dotR = Math.max(2, Math.min(cellW, cellH) * 0.14);
        for (const p of legalPlacements) {
          ctx.beginPath();
          ctx.arc((p.x + 0.5) * cellW, (p.y + 0.5) * cellH, dotR, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.globalAlpha = 1;
      }

      if (!isReplayInspecting && phase === 'human_turn' && hoverCell && activePiece.w > 0) {
        const { x, y } = hoverCell;
        const { w, h } = activePiece;
        const isLegal = legalSet.has(y * COLS + x);
        const gx = x * cellW;
        const gy = y * cellH;
        const gw = w * cellW;
        const gh = h * cellH;
        const activeColor = activePlayerSide === 1 ? p1Color : p2Color;

        if (isLegal) {
          if (hoverCapturePreview && hoverCapturePreview.length > 0) {
            ctx.fillStyle = captureColor;
            ctx.globalAlpha = 0.22;
            for (const cIdx of hoverCapturePreview) {
              const cx = cIdx % COLS;
              const cy = Math.floor(cIdx / COLS);
              ctx.fillRect(cx * cellW, cy * cellH, cellW, cellH);
            }
            ctx.globalAlpha = 0.92;
            drawClusterPerimeter(ctx, new Set(hoverCapturePreview), cellW, cellH, captureColor, 2, [5, 3]);
            ctx.globalAlpha = 1;
          }

          ctx.fillStyle = activeColor;
          ctx.globalAlpha = 0.36;
          ctx.fillRect(gx, gy, gw, gh);
          ctx.globalAlpha = 1;
          ctx.strokeStyle = '#eef1f6';
          ctx.lineWidth = 2;
          ctx.strokeRect(gx + 1, gy + 1, gw - 2, gh - 2);
        } else {
          ctx.fillStyle = dangerColor;
          ctx.globalAlpha = 0.22;
          ctx.fillRect(gx, gy, gw, gh);
          ctx.globalAlpha = 1;
          ctx.save();
          ctx.setLineDash([4, 3]);
          ctx.strokeStyle = dangerColor;
          ctx.lineWidth = 2;
          ctx.strokeRect(gx + 1, gy + 1, gw - 2, gh - 2);
          ctx.setLineDash([]);
          ctx.beginPath();
          ctx.moveTo(gx + 4, gy + 4);
          ctx.lineTo(gx + gw - 4, gy + gh - 4);
          ctx.moveTo(gx + gw - 4, gy + 4);
          ctx.lineTo(gx + 4, gy + gh - 4);
          ctx.stroke();
          ctx.restore();
        }
      }

      if (import.meta.env.DEV && devOpen && devHoverCandidate) {
        if (devCandCaptureCells.length > 0) {
          ctx.fillStyle = captureColor;
          ctx.globalAlpha = 0.25;
          for (const cIdx of devCandCaptureCells) {
            const cx = cIdx % COLS;
            const cy = Math.floor(cIdx / COLS);
            ctx.fillRect(cx * cellW, cy * cellH, cellW, cellH);
          }
          ctx.globalAlpha = 0.95;
          drawClusterPerimeter(ctx, new Set(devCandCaptureCells), cellW, cellH, captureColor, 2, [4, 2]);
          ctx.globalAlpha = 1;
        }
        ctx.save();
        ctx.strokeStyle = '#38bdf8';
        ctx.lineWidth = 2.5;
        ctx.setLineDash([6, 3]);
        ctx.strokeRect(
          devHoverCandidate.x * cellW + 1,
          devHoverCandidate.y * cellH + 1,
          devHoverCandidate.w * cellW - 2,
          devHoverCandidate.h * cellH - 2,
        );
        ctx.restore();
      }

      if (keepAnimating) {
        animFrame = requestAnimationFrame(draw);
      }
    };

    draw();
    const ro = new ResizeObserver(draw);
    ro.observe(canvas);
    return () => {
      ro.disconnect();
      if (animFrame) cancelAnimationFrame(animFrame);
    };
  }, [
    canvasRef,
    displayedGrid,
    displayedFrame,
    isReplayInspecting,
    phase,
    hoverCell,
    hoverCapturePreview,
    activePiece,
    legalPlacements,
    legalSet,
    activePlayerSide,
    lastOpponentMove,
    captureAnim,
    showAnchors,
    activeDevChannelData,
    devOpen,
    devHoverCandidate,
    devCandCaptureCells,
  ]);

  const getCellFromEvent = (e, clampPiece = true) => {
    const canvas = canvasRef.current;
    if (!canvas) return null;
    const rect = canvas.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return null;
    const cellW = rect.width / COLS;
    const cellH = rect.height / ROWS;
    const rawX = Math.floor((e.clientX - rect.left) / cellW);
    const rawY = Math.floor((e.clientY - rect.top) / cellH);
    if (!clampPiece || activePiece.w === 0) {
      return {
        x: Math.max(0, Math.min(COLS - 1, rawX)),
        y: Math.max(0, Math.min(ROWS - 1, rawY)),
      };
    }
    const x = Math.max(0, Math.min(COLS - activePiece.w, rawX));
    const y = Math.max(0, Math.min(ROWS - activePiece.h, rawY));
    return { x, y };
  };

  const handleCanvasPointerMove = (e) => {
    if (import.meta.env.DEV && activeDevChannelData) {
      const rawCell = getCellFromEvent(e, false);
      if (rawCell) {
        const val = Number(activeDevChannelData[rawCell.y * COLS + rawCell.x] || 0);
        setDevHoverReadout({ x: rawCell.x, y: rawCell.y, value: val });
      }
    }
    if (phase !== 'human_turn' || activePiece.w === 0 || isReplayInspecting) return;
    const cell = getCellFromEvent(e, true);
    if (cell) setHoverCell(cell);
  };

  const handleCanvasClick = (e) => {
    if (phase !== 'human_turn' || isReplayInspecting) return;
    const cell = e.detail > 0 ? getCellFromEvent(e, true) : hoverCell || getCellFromEvent(e, true);
    if (!cell) return;
    setHoverCell(cell);
    onPlaceAt(cell.x, cell.y);
  };

  const handleCanvasContextMenu = (e) => {
    e.preventDefault();
    onRotate();
  };

  return (
    <div className="board-frame">
      <canvas
        ref={canvasRef}
        data-testid="board-canvas"
        className={`board-canvas ${phase !== 'human_turn' || isReplayInspecting ? 'readonly' : ''}`}
        tabIndex={0}
        aria-label={t.boardAria}
        onPointerMove={handleCanvasPointerMove}
        onPointerLeave={() => {
          setHoverCell(null);
          if (import.meta.env.DEV) setDevHoverReadout(null);
        }}
        onClick={handleCanvasClick}
        onContextMenu={handleCanvasContextMenu}
      />

      {loadError && (
        <div className="board-overlay error" role="alert" data-testid="load-error">
          <div className="overlay-head">
            <h2 className="overlay-title">{t.engineError}</h2>
          </div>
          <div className="overlay-scores">{loadError}</div>
        </div>
      )}

      {phase === 'game_over' && !overlayDismissed && gameState && (
        <div className="board-overlay" data-testid="gameover-banner" role="region" aria-label={statusText}>
          <div className="overlay-head">
            <h2 className="overlay-title">{statusText}</h2>
            <button
              type="button"
              className="btn btn-subtle icon-close"
              aria-label={t.dismissOverlay}
              onClick={() => setOverlayDismissed(true)}
            >
              ×
            </button>
          </div>
          <div className="overlay-scores">
            <span className="overlay-score-item">
              <span className="swatch swatch-p1" aria-hidden="true" />
              {p1Label}: {p1Score}
            </span>
            <span>—</span>
            <span className="overlay-score-item">
              <span className="swatch swatch-p2" aria-hidden="true" />
              {p2Label}: {p2Score}
            </span>
          </div>
          <div className="overlay-actions">
            <button
              type="button"
              className="btn"
              data-testid="gameover-menu-btn"
              onClick={onOpenMenu}
            >
              {t.menu}
            </button>
            <button
              type="button"
              className="btn btn-primary"
              data-testid="play-again-btn"
              onClick={onPlayAgain}
            >
              {t.playAgain}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
