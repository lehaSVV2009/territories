import { useEffect } from 'react';

const ROWS = 15;
const COLS = 40;

export function useKeyboardShortcuts({
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
}) {
  useEffect(() => {
    const onKeyDown = (e) => {
      const dialogOpen = Boolean(menuDialogRef.current?.open || rulesDialogRef.current?.open);

      if (e.key === 'Escape') {
        if (!dialogOpen && !e.repeat && engineReady) {
          e.preventDefault();
          openMenu();
        }
        return;
      }

      if (import.meta.env.DEV && e.ctrlKey && e.shiftKey && e.code === 'KeyD') {
        e.preventDefault();
        toggleDevPanel();
        return;
      }

      if (dialogOpen) return;

      if (e.code === 'KeyR' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        handleRotate();
        return;
      }

      if (phase !== 'human_turn') return;

      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault();
        const activeSide = mode === '2p' ? gameState?.active_player || 1 : humanSide;
        setHoverCell((prev) => {
          const cur = prev || {
            x: activeSide === 1 ? 0 : Math.max(0, COLS - activePiece.w),
            y: activeSide === 1 ? 0 : Math.max(0, ROWS - activePiece.h),
          };
          let { x, y } = cur;
          if (e.key === 'ArrowLeft') x = Math.max(0, x - 1);
          if (e.key === 'ArrowRight') x = Math.min(Math.max(0, COLS - activePiece.w), x + 1);
          if (e.key === 'ArrowUp') y = Math.max(0, y - 1);
          if (e.key === 'ArrowDown') y = Math.min(Math.max(0, ROWS - activePiece.h), y + 1);
          return { x, y };
        });
      } else if ((e.key === 'Enter' || e.key === ' ') && hoverCell && document.activeElement === canvasRef.current) {
        e.preventDefault();
        handlePlaceAt(hoverCell.x, hoverCell.y);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
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
  ]);
}
