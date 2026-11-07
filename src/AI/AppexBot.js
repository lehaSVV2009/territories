import SpatialSlayerBot from "./SpatialSlayerBot";
import weightsData from "./weights.json";
import appexData from "./appex_weights.json";
import { CELL_TYPE, PLAYER_1, PLAYER_2 } from "territories-core";

const BOARD_ROWS = 15;
const BOARD_COLS = 40;
const LN_4 = Math.log(4.0);
const MAX_SLEW_RATE_DELTA = 0.15;

const CLUSTER_NAMES = [
  "Anti-Rush",
  "Anti-Wall/Loop",
  "Anti-Turtle/Greedy",
  "Anti-Brawler"
];

// Calibrated projection matrix (4x7) and bias vector (4) from appex/profiler.rs
const DEFAULT_PROJECTION_WEIGHTS = [
  [-2.6419, 2.2591, -3.7912, -13.6581, -11.2493, 16.4809, -0.2096],
  [4.4524, -3.2022, 4.7247, 1.7072, 7.8826, 7.4465, 0.3584],
  [-4.5547, -2.0751, -0.2437, 1.8510, -1.0513, -19.9653, 10.8025],
  [2.7443, 3.0183, -0.6897, 10.1000, 4.4181, -3.9621, -25.8262]
];

const DEFAULT_BIAS_VECTOR = [5.6783, -7.3144, -5.9623, 2.1000];

export default class AppexBot extends SpatialSlayerBot {
  constructor({ playerID, name, weights, matrix, counterWeights } = {}) {
    // Immutable Argon-4 base anchor
    const argonWeights = Object.assign(
      {},
      weightsData.params || weightsData.weights,
      weights
    );
    const spatialMatrix = matrix || weightsData.matrix || weightsData.spatial_matrix;

    super({
      playerID,
      name: name || "Appex (Адаптивная модель)",
      weights: argonWeights,
      matrix: spatialMatrix
    });

    this.argonWeights = Object.assign({}, argonWeights);
    this.counterWeights = counterWeights || appexData.counter_weights;
    this.projectionWeights = appexData.projection_weights || DEFAULT_PROJECTION_WEIGHTS;
    this.biasVector = appexData.bias_vector || DEFAULT_BIAS_VECTOR;

    // Profiler State
    this.smoothedFeatures = new Array(7).fill(0.0);
    this.emaAlpha = 0.3; // N ~ 5.67 turns
    this.historyLen = 0;
    this.probabilities = [0.25, 0.25, 0.25, 0.25];
    this.confirmedDominant = 0;
    this.pendingDominant = 0;
    this.pendingStreak = 0;

    // Board history tracking
    this.lastBoard = null;
    this.lastTurn = 0;

    // Live Telemetry log for UI introspection (AI Lab)
    this.telemetry = {
      turn: 0,
      probabilities: [0.25, 0.25, 0.25, 0.25],
      entropy: 1.0,
      alpha: 0.0,
      dominantCluster: CLUSTER_NAMES[0],
      activeWeights: Object.assign({}, this.argonWeights),
      smoothedFeatures: new Array(7).fill(0.0)
    };
  }

  /**
   * Extracts the 7 normalized telemetry features by comparing board before & after opponent move.
   * Enforces 180° rotation invariance when opponent is Player 2.
   */
  extractTelemetry(prevBoard, currBoard, oppPlayer, oppRect) {
    const isOppP2 = oppPlayer === PLAYER_2;

    // Map to canonical frame where opponent starts at (0, 0)
    let r0 = oppRect.rowIndex;
    let r1 = oppRect.rowIndex + oppRect.rectangleHeight - 1;
    let c0 = oppRect.columnIndex;
    let c1 = oppRect.columnIndex + oppRect.rectangleWidth - 1;
    let h = oppRect.rectangleHeight;
    let w = oppRect.rectangleWidth;

    if (isOppP2) {
      const origR0 = r0;
      const origR1 = r1;
      const origC0 = c0;
      const origC1 = c1;
      r0 = BOARD_ROWS - 1 - origR1;
      r1 = BOARD_ROWS - 1 - origR0;
      c0 = BOARD_COLS - 1 - origC1;
      c1 = BOARD_COLS - 1 - origC0;
    }

    const oppCellVal = isOppP2
      ? CELL_TYPE.OCCUPIED_BY_PLAYER_2
      : CELL_TYPE.OCCUPIED_BY_PLAYER_1;
    const myCellVal = isOppP2
      ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
      : CELL_TYPE.OCCUPIED_BY_PLAYER_2;

    // Features 1 & 2: Shift of opponent center-of-mass (delta_r, delta_c)
    let prevSumR = 0;
    let prevSumC = 0;
    let prevCount = 0;
    let currSumR = 0;
    let currSumC = 0;
    let currCount = 0;

    for (let r = 0; r < BOARD_ROWS; r++) {
      for (let c = 0; c < BOARD_COLS; c++) {
        const canR = isOppP2 ? BOARD_ROWS - 1 - r : r;
        const canC = isOppP2 ? BOARD_COLS - 1 - c : c;

        if (prevBoard && prevBoard[r][c] === oppCellVal) {
          prevSumR += canR;
          prevSumC += canC;
          prevCount++;
        }
        if (currBoard[r][c] === oppCellVal) {
          currSumR += canR;
          currSumC += canC;
          currCount++;
        }
      }
    }

    const prevMeanR = prevCount > 0 ? prevSumR / prevCount : 0.0;
    const prevMeanC = prevCount > 0 ? prevSumC / prevCount : 0.0;
    const currMeanR = currCount > 0 ? currSumR / currCount : (r0 + r1) / 2.0;
    const currMeanC = currCount > 0 ? currSumC / currCount : (c0 + c1) / 2.0;

    const deltaR = Math.max(-1.0, Math.min(1.0, (currMeanR - prevMeanR) / 15.0));
    const deltaC = Math.max(-1.0, Math.min(1.0, (currMeanC - prevMeanC) / 40.0));

    // Feature 3: Placed rectangle area normalized by 36.0
    const area = h * w;
    const areaNorm = Math.max(1.0 / 36.0, Math.min(1.0, area / 36.0));

    // Feature 4: Normalized Euclidean distance to board center (7.0, 19.5)
    const rectCenterR = (r0 + r1) / 2.0;
    const rectCenterC = (c0 + c1) / 2.0;
    const distCenter = Math.sqrt(
      Math.pow(rectCenterR - 7.0, 2) + Math.pow(rectCenterC - 19.5, 2)
    );
    const maxCenterDist = Math.sqrt(Math.pow(7.0, 2) + Math.pow(19.5, 2));
    const centerDistNorm = Math.max(0.0, Math.min(1.0, distCenter / maxCenterDist));

    // Feature 5: Min distance to board borders
    const borderDist = Math.min(r0, BOARD_ROWS - 1 - r1, c0, BOARD_COLS - 1 - c1);
    const borderDistNorm = Math.max(0.0, Math.min(1.0, borderDist / 7.0));

    // Feature 6: Fraction of cells on boundary rows 0 or 14
    let perimCells = 0;
    if (r0 === 0) perimCells += w;
    if (r1 === 14 && r0 !== 14) perimCells += w;
    const perimRatio = Math.max(0.0, Math.min(1.0, perimCells / area));

    // Feature 7: Min Manhattan distance to friendly cells
    let minContactDist = 53.0;
    for (let r = 0; r < BOARD_ROWS; r++) {
      for (let c = 0; c < BOARD_COLS; c++) {
        if (currBoard[r][c] === myCellVal) {
          const canR = isOppP2 ? BOARD_ROWS - 1 - r : r;
          const canC = isOppP2 ? BOARD_COLS - 1 - c : c;
          // Distance from cell (canR, canC) to rectangle [r0..r1, c0..c1]
          const dr = canR < r0 ? r0 - canR : canR > r1 ? canR - r1 : 0;
          const dc = canC < c0 ? c0 - canC : canC > c1 ? canC - c1 : 0;
          const d = dr + dc;
          if (d < minContactDist) {
            minContactDist = d;
          }
        }
      }
    }
    const contactDistNorm = Math.max(0.0, Math.min(1.0, minContactDist / 53.0));

    return [
      deltaR,
      deltaC,
      areaNorm,
      centerDistNorm,
      borderDistNorm,
      perimRatio,
      contactDistNorm
    ];
  }

  /**
   * Updates EMA smoothing, Softmax clustering, hysteresis, and slew-rate clamping.
   */
  updateProfiler(rawFeatures) {
    // 1. EMA update
    if (this.historyLen === 0) {
      this.smoothedFeatures = [...rawFeatures];
    } else {
      for (let i = 0; i < 7; i++) {
        this.smoothedFeatures[i] =
          this.emaAlpha * rawFeatures[i] + (1.0 - this.emaAlpha) * this.smoothedFeatures[i];
      }
    }
    this.historyLen++;

    // 2. Affine projection logits: z_k = sum_j (W_{kj} * F_j) + b_k
    const logits = [0, 0, 0, 0];
    for (let k = 0; k < 4; k++) {
      let dot = 0.0;
      for (let j = 0; j < 7; j++) {
        dot += this.projectionWeights[k][j] * this.smoothedFeatures[j];
      }
      logits[k] = dot + this.biasVector[k];
    }

    // 3. Numerically stable Softmax
    let maxLogit = -Infinity;
    for (let k = 0; k < 4; k++) {
      if (logits[k] > maxLogit) maxLogit = logits[k];
    }

    let expSum = 0.0;
    const rawProbs = [0, 0, 0, 0];
    for (let k = 0; k < 4; k++) {
      const e = Math.exp(logits[k] - maxLogit);
      rawProbs[k] = e;
      expSum += e;
    }
    if (expSum > 0.0) {
      for (let k = 0; k < 4; k++) {
        rawProbs[k] /= expSum;
      }
    } else {
      rawProbs.fill(0.25);
    }

    // 4. Hysteresis confirmation (2-turn persistence)
    let rawDominant = 0;
    let maxP = -1;
    for (let k = 0; k < 4; k++) {
      if (rawProbs[k] > maxP) {
        maxP = rawProbs[k];
        rawDominant = k;
      }
    }

    if (this.historyLen === 1) {
      this.confirmedDominant = rawDominant;
      this.pendingDominant = rawDominant;
      this.pendingStreak = 1;
    } else if (rawDominant === this.confirmedDominant) {
      this.pendingDominant = rawDominant;
      this.pendingStreak = 0;
    } else {
      if (rawDominant === this.pendingDominant) {
        this.pendingStreak++;
        if (this.pendingStreak >= 2) {
          this.confirmedDominant = rawDominant;
        }
      } else {
        this.pendingDominant = rawDominant;
        this.pendingStreak = 1;
      }
    }

    // 5. Geodesic simplex slew-rate clamping: delta p <= 0.15
    let maxDiff = 0.0;
    for (let k = 0; k < 4; k++) {
      const diff = Math.abs(rawProbs[k] - this.probabilities[k]);
      if (diff > maxDiff) maxDiff = diff;
    }

    const lambda = maxDiff > MAX_SLEW_RATE_DELTA ? MAX_SLEW_RATE_DELTA / maxDiff : 1.0;
    for (let k = 0; k < 4; k++) {
      this.probabilities[k] =
        (1.0 - lambda) * this.probabilities[k] + lambda * rawProbs[k];
    }
  }

  /**
   * Computes normalized Shannon entropy H(p) in [0.0, 1.0].
   */
  computeNormalizedEntropy() {
    let sumPLnP = 0.0;
    for (let k = 0; k < 4; k++) {
      const p = this.probabilities[k];
      if (p > 1e-12) {
        sumPLnP += p * Math.log(p);
      }
    }
    const h = -sumPLnP / LN_4;
    return Math.max(0.0, Math.min(1.0, h));
  }

  /**
   * Computes active weights blending Argon-4 anchor and the 4 counter profiles.
   */
  computeActiveWeights(turn) {
    const H = this.computeNormalizedEntropy();
    const currentTurn = turn || this.lastTurn + 1;
    const rawAlpha = (currentTurn / 8.0) * (1.0 - H);
    const alpha = Math.max(0.0, Math.min(0.65, rawAlpha));

    const activeWeights = {};
    for (const key of Object.keys(this.argonWeights)) {
      if (key === "WEIGHT_AREA") {
        activeWeights[key] = 50.0; // Invariant Numeraire
        continue;
      }
      let counterSum = 0.0;
      for (let k = 0; k < 4; k++) {
        const cVal =
          this.counterWeights[k][key] !== undefined
            ? this.counterWeights[k][key]
            : this.argonWeights[key];
        counterSum += this.probabilities[k] * cVal;
      }
      activeWeights[key] = (1.0 - alpha) * this.argonWeights[key] + alpha * counterSum;
    }

    return { activeWeights, alpha, entropy: H };
  }

  /**
   * Autonomous move selection with telemetry tracking and soft blending.
   */
  guessCellToDropRectangle({
    turn,
    currentPlayer,
    rectangleHeight,
    rectangleWidth,
    rows,
    emptyCellsCount
  }) {
    const oppPlayer =
      currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

    // Detect opponent's newly placed rectangle from board diff
    if (this.lastBoard) {
      const oppCellVal =
        oppPlayer === PLAYER_1
          ? CELL_TYPE.OCCUPIED_BY_PLAYER_1
          : CELL_TYPE.OCCUPIED_BY_PLAYER_2;

      let minR = BOARD_ROWS;
      let maxR = -1;
      let minC = BOARD_COLS;
      let maxC = -1;
      let diffFound = false;

      for (let r = 0; r < BOARD_ROWS; r++) {
        for (let c = 0; c < BOARD_COLS; c++) {
          if (rows[r][c] === oppCellVal && this.lastBoard[r][c] !== oppCellVal) {
            if (r < minR) minR = r;
            if (r > maxR) maxR = r;
            if (c < minC) minC = c;
            if (c > maxC) maxC = c;
            diffFound = true;
          }
        }
      }

      if (diffFound && maxR >= minR && maxC >= minC) {
        const oppRect = {
          rowIndex: minR,
          columnIndex: minC,
          rectangleHeight: maxR - minR + 1,
          rectangleWidth: maxC - minC + 1
        };
        const rawTelemetry = this.extractTelemetry(
          this.lastBoard,
          rows,
          oppPlayer,
          oppRect
        );
        this.updateProfiler(rawTelemetry);
      }
    }

    // Blend active heuristic weights
    const { activeWeights, alpha, entropy } = this.computeActiveWeights(turn);
    this.weights = activeWeights;

    // Update telemetry state for UI / debugging
    this.telemetry = {
      turn: turn || this.lastTurn + 1,
      probabilities: [...this.probabilities],
      entropy,
      alpha,
      dominantCluster: CLUSTER_NAMES[this.confirmedDominant],
      activeWeights: Object.assign({}, activeWeights),
      smoothedFeatures: [...this.smoothedFeatures]
    };

    // Delegate move selection to SpatialSlayer with active weights
    const result = super.guessCellToDropRectangle({
      turn,
      currentPlayer,
      rectangleHeight,
      rectangleWidth,
      rows,
      emptyCellsCount
    });

    // Save board snapshot for next turn's diff
    this.lastBoard = rows.map(row => [...row]);
    this.lastTurn = turn;

    return result;
  }

  /**
   * Exposes real-time telemetry data for AI Lab visualization.
   */
  getTelemetry() {
    return {
      ...this.telemetry,
      clusterNames: CLUSTER_NAMES,
      argonShare: (1.0 - this.telemetry.alpha) * 100,
      counterShare: this.telemetry.alpha * 100
    };
  }

  getTelemetrySummary() {
    return this.getTelemetry();
  }
}
