import Bot from "./Bot";
import {
  findPotentiallyOccupiedRectangles,
  findClosedLoops,
  CELL_TYPE,
  PLAYER_1,
  PLAYER_2
} from "territories-core";

const BOARD_ROWS = 15;
const BOARD_COLS = 40;
const TOTAL_CELLS = BOARD_ROWS * BOARD_COLS;
const INF = 9999;

export function generateDefaultMatrix() {
  const matrix = new Array(TOTAL_CELLS).fill(0.0);
  for (let r = 0; r < BOARD_ROWS; r++) {
    for (let c = 0; c < BOARD_COLS; c++) {
      const idx = r * BOARD_COLS + c;
      let w = 10.0;
      const distCol = Math.abs(c - 19.5);
      const distRow = Math.abs(r - 7.0);
      if (distCol <= 3.0) {
        w += (4.0 - distCol) * 8.0;
      }
      if (distRow <= 3.0) {
        w += (4.0 - distRow) * 4.0;
      }
      if (r === 0 || r === 14) {
        w += 6.0;
      }
      if (c === 10 || c === 20 || c === 30) {
        w += 5.0;
      }
      matrix[idx] = w;
    }
  }
  return matrix;
}

export const DEFAULT_V2_WEIGHTS = {
  WEIGHT_AREA: 50.0,
  WEIGHT_CAPTURE: 95.0,
  WEIGHT_DENY_CAPTURE: 85.0,
  WEIGHT_VORONOI_DELTA: 26.0,
  WEIGHT_SAFE_TERRITORY: 23.088,
  WEIGHT_COMPACTNESS: 12.973,
  WEIGHT_SPATIAL_MATRIX: 0.322,
  WEIGHT_FRONTIER: 17.307,
  WEIGHT_OPP_CHOKE: 18.0,
  WEIGHT_BOTTLENECK: 12.0
};

export default class SpatialSlayerBot extends Bot {
  constructor({ playerID, name, weights, matrix } = {}) {
    super({ playerID, name: name || "SpatialSlayer" });
    this.weights = Object.assign({}, DEFAULT_V2_WEIGHTS, weights);
    this.matrix = matrix || generateDefaultMatrix();
  }

  rotateBoard180(rows) {
    const rotated = [];
    for (let r = 0; r < BOARD_ROWS; r++) {
      const row = [];
      const origR = BOARD_ROWS - 1 - r;
      for (let c = 0; c < BOARD_COLS; c++) {
        const origC = BOARD_COLS - 1 - c;
        const val = rows[origR][origC];
        if (val === CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
          row.push(CELL_TYPE.OCCUPIED_BY_PLAYER_2);
        } else if (val === CELL_TYPE.OCCUPIED_BY_PLAYER_2) {
          row.push(CELL_TYPE.OCCUPIED_BY_PLAYER_1);
        } else {
          row.push(CELL_TYPE.EMPTY);
        }
      }
      rotated.push(row);
    }
    return rotated;
  }

  evaluateMoveAsP2(board, rect, emptyCellsCount) {
    const { rowIndex: r0, columnIndex: c0, rectangleHeight: h, rectangleWidth: w } = rect;
    const weights = this.weights;
    const matrix = this.matrix;

    // 1. Area score
    const area = h * w;
    let score = area * weights.WEIGHT_AREA;

    // 2. Compactness
    const minDim = Math.min(h, w);
    if (emptyCellsCount <= 490) {
      if (minDim >= 2) {
        score += weights.WEIGHT_COMPACTNESS;
      } else if (minDim === 1 && area > 2) {
        score -= 1.5 * weights.WEIGHT_COMPACTNESS;
      }
    }

    // 3. Multi-source BFS for Voronoi influence
    // d1: Distance from P1 (enemy)
    // d2: Distance from P2 + candidate cells
    const d1 = new Int16Array(TOTAL_CELLS).fill(INF);
    const d2 = new Int16Array(TOTAL_CELLS).fill(INF);

    const q1 = [];
    const q2 = [];

    // Copy board and place candidate temporarily
    const candidateCells = new Set();
    for (let r = r0; r < r0 + h; r++) {
      for (let c = c0; c < c0 + w; c++) {
        candidateCells.add(r * BOARD_COLS + c);
      }
    }

    for (let r = 0; r < BOARD_ROWS; r++) {
      for (let c = 0; c < BOARD_COLS; c++) {
        const idx = r * BOARD_COLS + c;
        const cell = board[r][c];

        if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
          d1[idx] = 0;
          q1.push(idx);
        } else if (cell === CELL_TYPE.OCCUPIED_BY_PLAYER_2 || candidateCells.has(idx)) {
          d2[idx] = 0;
          q2.push(idx);
        }
      }
    }

    // Run BFS for d1 (P1 waves cannot pass through P2 or candidate)
    let head = 0;
    while (head < q1.length) {
      const curr = q1[head++];
      const dist = d1[curr];
      const r = Math.floor(curr / BOARD_COLS);
      const c = curr % BOARD_COLS;

      const neighbors = [
        r > 0 ? (r - 1) * BOARD_COLS + c : -1,
        r < BOARD_ROWS - 1 ? (r + 1) * BOARD_COLS + c : -1,
        c > 0 ? r * BOARD_COLS + (c - 1) : -1,
        c < BOARD_COLS - 1 ? r * BOARD_COLS + (c + 1) : -1
      ];

      for (let i = 0; i < 4; i++) {
        const nIdx = neighbors[i];
        if (nIdx !== -1 && d1[nIdx] === INF) {
          const nR = Math.floor(nIdx / BOARD_COLS);
          const nC = nIdx % BOARD_COLS;
          if (
            board[nR][nC] !== CELL_TYPE.OCCUPIED_BY_PLAYER_2 &&
            !candidateCells.has(nIdx)
          ) {
            d1[nIdx] = dist + 1;
            q1.push(nIdx);
          }
        }
      }
    }

    // Run BFS for d2 (P2 waves cannot pass through P1)
    head = 0;
    while (head < q2.length) {
      const curr = q2[head++];
      const dist = d2[curr];
      const r = Math.floor(curr / BOARD_COLS);
      const c = curr % BOARD_COLS;

      const neighbors = [
        r > 0 ? (r - 1) * BOARD_COLS + c : -1,
        r < BOARD_ROWS - 1 ? (r + 1) * BOARD_COLS + c : -1,
        c > 0 ? r * BOARD_COLS + (c - 1) : -1,
        c < BOARD_COLS - 1 ? r * BOARD_COLS + (c + 1) : -1
      ];

      for (let i = 0; i < 4; i++) {
        const nIdx = neighbors[i];
        if (nIdx !== -1 && d2[nIdx] === INF) {
          const nR = Math.floor(nIdx / BOARD_COLS);
          const nC = nIdx % BOARD_COLS;
          if (board[nR][nC] !== CELL_TYPE.OCCUPIED_BY_PLAYER_1) {
            d2[nIdx] = dist + 1;
            q2.push(nIdx);
          }
        }
      }
    }

    let p1Territory = 0;
    let p2Territory = 0;
    let p1SafeTerritory = 0;
    let p2SafeTerritory = 0;
    let spatialScore = 0.0;
    let oppChokedCells = 0;

    for (let r = 0; r < BOARD_ROWS; r++) {
      for (let c = 0; c < BOARD_COLS; c++) {
        const idx = r * BOARD_COLS + c;
        if (
          board[r][c] === CELL_TYPE.EMPTY &&
          !candidateCells.has(idx)
        ) {
          const dist1 = d1[idx];
          const dist2 = d2[idx];

          if (dist2 < dist1) {
            p2Territory++;
            spatialScore += matrix[idx];
            if (dist1 === INF) {
              p2SafeTerritory++;
              spatialScore += 2.0 * matrix[idx];
            }
          } else if (dist1 < dist2) {
            p1Territory++;
            spatialScore -= matrix[idx];
            if (dist2 === INF) {
              p1SafeTerritory++;
              spatialScore -= 2.0 * matrix[idx];
            }
          }

          if (dist1 <= 2 && dist2 <= dist1 + 1) {
            oppChokedCells++;
          }
        }
      }
    }

    score +=
      (p2Territory - p1Territory) * weights.WEIGHT_VORONOI_DELTA +
      (p2SafeTerritory - p1SafeTerritory) * weights.WEIGHT_SAFE_TERRITORY +
      spatialScore * weights.WEIGHT_SPATIAL_MATRIX +
      oppChokedCells * weights.WEIGHT_OPP_CHOKE;

    // 4. Frontier advancement
    const scoreFrontier =
      ((39.0 - c0) / 39.0 + 1.0 - Math.abs(r0 + 0.5 * h - 7.5) / 7.5) *
      weights.WEIGHT_FRONTIER;
    score += scoreFrontier;

    // 5. Bottleneck and outer edge bonus
    if (r0 === 0 || r0 + h === 15 || c0 === 0 || c0 + w === 40) {
      if (c0 < 25) {
        score += weights.WEIGHT_BOTTLENECK;
      }
    }

    // 6. Fast Loop Capture Check
    // Create test board with candidate placed to check findClosedLoops
    const testBoard = board.map(row => row.slice());
    for (let r = r0; r < r0 + h; r++) {
      for (let c = c0; c < c0 + w; c++) {
        testBoard[r][c] = CELL_TYPE.OCCUPIED_BY_PLAYER_2;
      }
    }

    try {
      const closedLoops = findClosedLoops({ rows: testBoard });
      if (closedLoops && closedLoops.length > 0) {
        let capturedMy = 0;
        for (let i = 0; i < closedLoops.length; i++) {
          const loop = closedLoops[i];
          if (loop && loop.player === PLAYER_2) {
            capturedMy += loop.cells ? loop.cells.length : 0;
          }
        }
        score += capturedMy * weights.WEIGHT_CAPTURE;
      }
    } catch (e) {
      // Loop detection fallback
    }

    return score;
  }

  guessCellToDropRectangle = ({
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows,
    emptyCellsCount
  }) => {
    const isP1 = currentPlayer === PLAYER_1;

    // Symmetrical 180° rotation: evaluate strictly from P2 perspective
    const evalBoard = isP1 ? this.rotateBoard180(rows) : rows;
    const evalPlayer = PLAYER_2;

    const rects = findPotentiallyOccupiedRectangles({
      currentPlayer: evalPlayer,
      rectangleHeight,
      rectangleWidth,
      rows: evalBoard
    }).concat(
      rectangleHeight === rectangleWidth
        ? []
        : findPotentiallyOccupiedRectangles({
            currentPlayer: evalPlayer,
            rectangleHeight: rectangleWidth,
            rectangleWidth: rectangleHeight,
            rows: evalBoard
          })
    );

    if (!rects || rects.length === 0) {
      return {
        rowIndex: -1,
        columnIndex: -1,
        rectangleHeight,
        rectangleWidth
      };
    }

    let bestRect = rects[0];
    let bestScore = -Infinity;

    for (let i = 0; i < rects.length; i++) {
      const rect = rects[i];
      const moveScore = this.evaluateMoveAsP2(evalBoard, rect, emptyCellsCount || 550);

      if (moveScore > bestScore) {
        bestScore = moveScore;
        bestRect = rect;
      }
    }

    // Inverse mapping if evaluating for Player 1
    if (isP1) {
      const realRow = 15 - (bestRect.rowIndex + bestRect.rectangleHeight);
      const realCol = 40 - (bestRect.columnIndex + bestRect.rectangleWidth);
      return {
        rowIndex: realRow,
        columnIndex: realCol,
        rectangleHeight: bestRect.rectangleHeight,
        rectangleWidth: bestRect.rectangleWidth
      };
    }

    return {
      rowIndex: bestRect.rowIndex,
      columnIndex: bestRect.columnIndex,
      rectangleHeight: bestRect.rectangleHeight,
      rectangleWidth: bestRect.rectangleWidth
    };
  };
}
