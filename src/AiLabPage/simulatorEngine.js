import {
  CELL_TYPE,
  PLAYER_1,
  PLAYER_2,
  findClosedLoops,
  canDropRectangle,
  isEmptyCell,
  isOccupiedByPlayerOneCell
} from "territories-core";
import AI, { BOT_TYPES } from "../AI";

export const BOARD_ROWS = 15;
export const BOARD_COLS = 40;
export const TOTAL_CELLS = BOARD_ROWS * BOARD_COLS;

export function mulberry32(seed) {
  let s = seed | 0;
  return function() {
    s = (s + 0x6d2d679f) | 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function createEmptyBoard() {
  const board = [];
  for (let r = 0; r < BOARD_ROWS; r++) {
    const row = new Array(BOARD_COLS).fill(CELL_TYPE.EMPTY);
    board.push(row);
  }
  return board;
}

export function cloneBoard(board) {
  return board.map(row => row.slice());
}

export function countBoardCells(board) {
  let p1 = 0;
  let p2 = 0;
  let empty = 0;
  for (let r = 0; r < BOARD_ROWS; r++) {
    for (let c = 0; c < BOARD_COLS; c++) {
      const cell = board[r][c];
      if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
        p1++;
      } else if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_2) {
        p2++;
      } else {
        empty++;
      }
    }
  }
  return { p1, p2, empty };
}

export function applyMoveAndEnclosures(board, move, player) {
  const newBoard = cloneBoard(board);
  const { rowIndex: r0, columnIndex: c0, rectangleHeight: h, rectangleWidth: w } = move;

  const targetCellType =
    player === PLAYER_1
      ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
      : CELL_TYPE.OCCUPIED_BY_PLAYER_2;

  // 1. Fill rectangle
  for (let r = r0; r < r0 + h; r++) {
    for (let c = c0; c < c0 + w; c++) {
      if (r >= 0 && r < BOARD_ROWS && c >= 0 && c < BOARD_COLS) {
        newBoard[r][c] = targetCellType;
      }
    }
  }

  // 2. Check enclosed loops
  let enclosedCount = 0;
  const counts = countBoardCells(newBoard);
  if (counts.p1 > 0 && counts.p2 > 0) {
    const closedLoops = findClosedLoops({
      matrix: newBoard,
      loopHabitantsFilter: ({ value }) => isEmptyCell(value)
    });

    closedLoops.forEach(closedLoop => {
      if (closedLoop.neigboursValues.length === 1) {
        const neighbor = closedLoop.neigboursValues[0];
        const loopOwner = isOccupiedByPlayerOneCell(neighbor)
          ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
          : CELL_TYPE.OCCUPIED_BY_PLAYER_2;

        closedLoop.cells.forEach(cell => {
          if (newBoard[cell.rowIndex][cell.columnIndex] === CELL_TYPE.EMPTY) {
            newBoard[cell.rowIndex][cell.columnIndex] = loopOwner;
            enclosedCount++;
          }
        });
      }
    });
  }

  return { newBoard, enclosedCount };
}

export function simulateSingleMatch({
  p1Type = BOT_TYPES.G40,
  p2Type = BOT_TYPES.SPATIAL_SLAYER,
  seed = Math.floor(Math.random() * 1000000),
  maxTurns = 120,
  recordHistory = true
} = {}) {
  const rng = mulberry32(seed);

  const p1Bot = AI({ playerID: PLAYER_1, type: p1Type, name: "P1" });
  const p2Bot = AI({ playerID: PLAYER_2, type: p2Type, name: "P2" });

  let board = createEmptyBoard();
  let consecutiveSkips = 0;
  let turn = 1;
  const history = [];

  if (recordHistory) {
    history.push({
      turn: 0,
      player: null,
      dices: [0, 0],
      move: null,
      skipped: false,
      board: cloneBoard(board),
      scores: countBoardCells(board),
      enclosedCount: 0,
      commentary: "Старт матча"
    });
  }

  while (turn <= maxTurns) {
    const isP1 = turn % 2 === 1;
    const currentPlayer = isP1 ? PLAYER_1 : PLAYER_2;
    const currentBot = isP1 ? p1Bot : p2Bot;

    // Roll two dice (1..6)
    const d1 = Math.floor(rng() * 6) + 1;
    const d2 = Math.floor(rng() * 6) + 1;

    const cellCounts = countBoardCells(board);
    if (cellCounts.empty === 0) {
      break;
    }

    const decision = currentBot.guessCellToDropRectangle({
      turn,
      currentPlayer,
      rectangleHeight: d1,
      rectangleWidth: d2,
      rows: board,
      emptyCellsCount: cellCounts.empty
    });

    let moveValid = false;
    let actualMove = null;

    if (
      decision &&
      decision.rowIndex !== -1 &&
      decision.columnIndex !== -1
    ) {
      const h = decision.rectangleHeight || d1;
      const w = decision.rectangleWidth || d2;

      // Verify that this rectangle is legal
      const canDrop = canDropRectangle({
        rowIndex: decision.rowIndex,
        columnIndex: decision.columnIndex,
        value: board[decision.rowIndex][decision.columnIndex],
        rows: board,
        rectangleHeight: h,
        rectangleWidth: w,
        currentPlayer
      });

      if (canDrop) {
        moveValid = true;
        actualMove = {
          rowIndex: decision.rowIndex,
          columnIndex: decision.columnIndex,
          rectangleHeight: h,
          rectangleWidth: w
        };
      }
    }

    if (moveValid) {
      consecutiveSkips = 0;
      const { newBoard, enclosedCount } = applyMoveAndEnclosures(
        board,
        actualMove,
        currentPlayer
      );
      board = newBoard;

      if (recordHistory) {
        const scores = countBoardCells(board);
        const playerName = isP1 ? "Игрок 1 (Синий)" : "Игрок 2 (Оранжевый)";
        const captureInfo =
          enclosedCount > 0
            ? ` Захвачено контуром: +${enclosedCount} клеток!`
            : "";
        history.push({
          turn,
          player: currentPlayer,
          dices: [actualMove.rectangleHeight, actualMove.rectangleWidth],
          move: actualMove,
          skipped: false,
          board: cloneBoard(board),
          scores,
          enclosedCount,
          commentary: `${playerName}: прямоугольник ${actualMove.rectangleHeight}x${actualMove.rectangleWidth} в (${actualMove.rowIndex}, ${actualMove.columnIndex}).${captureInfo}`
        });
      }
    } else {
      consecutiveSkips++;
      if (recordHistory) {
        const scores = countBoardCells(board);
        const playerName = isP1 ? "Игрок 1 (Синий)" : "Игрок 2 (Оранжевый)";
        history.push({
          turn,
          player: currentPlayer,
          dices: [d1, d2],
          move: null,
          skipped: true,
          board: cloneBoard(board),
          scores,
          enclosedCount: 0,
          commentary: `${playerName}: пропуск хода (нет доступных ходов для ${d1}x${d2}).`
        });
      }

      if (consecutiveSkips >= 2) {
        // Both players cannot place any rectangle
        break;
      }
    }

    turn++;
  }

  const finalScores = countBoardCells(board);
  let winner = "DRAW";
  if (finalScores.p1 > finalScores.p2) {
    winner = PLAYER_1;
  } else if (finalScores.p2 > finalScores.p1) {
    winner = PLAYER_2;
  }

  const margin = finalScores.p1 - finalScores.p2;

  return {
    seed,
    p1Type,
    p2Type,
    winner,
    p1Score: finalScores.p1,
    p2Score: finalScores.p2,
    emptyScore: finalScores.empty,
    margin,
    totalTurns: turn,
    finalBoard: board,
    history
  };
}

export async function runTournamentBatch({
  p1Type,
  p2Type,
  matchCount = 10,
  startSeed = 1000,
  onProgress = () => {},
  shouldStop = () => false
}) {
  const matches = [];
  let p1Wins = 0;
  let p2Wins = 0;
  let draws = 0;
  let totalMargin = 0;
  let totalP1Cells = 0;
  let totalP2Cells = 0;

  for (let i = 0; i < matchCount; i++) {
    if (shouldStop()) {
      break;
    }

    const currentSeed = startSeed + i * 17;
    // Fast match simulation without heavy snapshot history
    const match = simulateSingleMatch({
      p1Type,
      p2Type,
      seed: currentSeed,
      recordHistory: false
    });

    matches.push(match);

    if (match.winner === PLAYER_1) {
      p1Wins++;
    } else if (match.winner === PLAYER_2) {
      p2Wins++;
    } else {
      draws++;
    }

    totalMargin += match.margin;
    totalP1Cells += match.p1Score;
    totalP2Cells += match.p2Score;

    const progress = {
      completed: i + 1,
      total: matchCount,
      percentage: Math.round(((i + 1) / matchCount) * 100),
      currentMatch: match,
      p1Wins,
      p2Wins,
      draws,
      p1Winrate: Math.round((p1Wins / (i + 1)) * 1000) / 10,
      p2Winrate: Math.round((p2Wins / (i + 1)) * 1000) / 10,
      drawRate: Math.round((draws / (i + 1)) * 1000) / 10,
      avgMargin: Math.round((totalMargin / (i + 1)) * 10) / 10,
      avgP1Cells: Math.round((totalP1Cells / (i + 1)) * 10) / 10,
      avgP2Cells: Math.round((totalP2Cells / (i + 1)) * 10) / 10
    };

    onProgress(progress);

    // Yield control to UI thread every match so animations stay smooth
    await new Promise(resolve => setTimeout(resolve, 0));
  }

  return {
    matches,
    p1Wins,
    p2Wins,
    draws,
    totalMatches: matches.length,
    p1Winrate:
      matches.length > 0 ? Math.round((p1Wins / matches.length) * 1000) / 10 : 0,
    p2Winrate:
      matches.length > 0 ? Math.round((p2Wins / matches.length) * 1000) / 10 : 0,
    drawRate:
      matches.length > 0 ? Math.round((draws / matches.length) * 1000) / 10 : 0,
    avgMargin:
      matches.length > 0 ? Math.round((totalMargin / matches.length) * 10) / 10 : 0,
    avgP1Cells:
      matches.length > 0 ? Math.round((totalP1Cells / matches.length) * 10) / 10 : 0,
    avgP2Cells:
      matches.length > 0 ? Math.round((totalP2Cells / matches.length) * 10) / 10 : 0
  };
}
