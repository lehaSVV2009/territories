#!/usr/bin/env python3
"""PyRust Evolutionary Learning Engine for Territories AI (Milestone M4).

Orchestrates evolutionary parameter optimization and spatial prior refinement:
1. Fixed Numeraire: WEIGHT_AREA = 50.0 is invariant and never mutated.
2. Clamping Corridors: Strict bounds enforced across all 9 heuristic parameters.
3. Symmetric Radial Gaussian Bumps: 2-3 local 2D Gaussian domes (sigma ~ 2.5..4.0, amplitude +-Delta)
   with strict 180° point-inversion / horizontal & vertical mirroring (14 - r, 39 - c).
   Independent white noise per cell is strictly prohibited. Matrix clamped to [0.0, 50.0].
4. Fitness Function: Fitness = WinRate_% + 0.02 * AvgMargin (win rate primary, margin tie-breaker).
5. Anchor Set Validation: Fixed 100 CRN seed pairs (1000..1099, 200 matches) with multi-archetype
   validation against Champion, GreedyBot, and LegacyHardBot to prevent over-fitting/rock-paper-scissors cycles.
6. Top-2 Elitism: If Top-1 mutant scores >60% in generation but narrowly misses Anchor Set validation,
   Top-2 mutant of the same generation is also tested.
7. Batch Rayon Parallel Evaluation via pyrust.evaluate_population_ffi (lambda=16 default).
8. Browser JS compatibility: weights.json persistence via pyrust.save_weights_file.
9. Weight Evolution Delta Report: Formatted delta table (Gen 0 vs Gen N) for params and spatial matrix.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import json
import math
import os
import random
import resource
import sys
import time
from typing import Any, Dict, List, Optional, Sequence, Tuple

try:
    import pyrust
except ImportError:
    # If pyrust is not in standard path, try importing from current directory
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
    import pyrust  # type: ignore

# Board dimensions
BOARD_HEIGHT: int = 15
BOARD_WIDTH: int = 40
TOTAL_CELLS: int = BOARD_HEIGHT * BOARD_WIDTH  # 600

# Canonical 10-parameter representation
PARAM_NAMES: List[str] = [
    "WEIGHT_AREA",
    "WEIGHT_CAPTURE",
    "WEIGHT_DENY_CAPTURE",
    "WEIGHT_VORONOI_DELTA",
    "WEIGHT_SAFE_TERRITORY",
    "WEIGHT_COMPACTNESS",
    "WEIGHT_SPATIAL_MATRIX",
    "WEIGHT_FRONTIER",
    "WEIGHT_OPP_CHOKE",
    "WEIGHT_BOTTLENECK",
]

# Clamping corridors per PROJECT.md, DISPATCH.md, and src/selector.rs
CLAMPING_CORRIDORS: Dict[str, Tuple[float, float]] = {
    "WEIGHT_AREA": (50.0, 50.0),  # Invariant Numeraire
    "WEIGHT_CAPTURE": (70.0, 150.0),
    "WEIGHT_DENY_CAPTURE": (60.0, 150.0),
    "WEIGHT_VORONOI_DELTA": (15.0, 45.0),
    "WEIGHT_SAFE_TERRITORY": (15.0, 35.0),
    "WEIGHT_COMPACTNESS": (5.0, 25.0),
    "WEIGHT_SPATIAL_MATRIX": (0.10, 0.50),
    "WEIGHT_FRONTIER": (10.0, 30.0),
    "WEIGHT_OPP_CHOKE": (5.0, 25.0),
    "WEIGHT_BOTTLENECK": (5.0, 25.0),
}

# Baseline reference parameters
DEFAULT_WEIGHTS: Dict[str, float] = {
    "WEIGHT_AREA": 50.0,
    "WEIGHT_CAPTURE": 95.0,
    "WEIGHT_DENY_CAPTURE": 85.0,
    "WEIGHT_VORONOI_DELTA": 26.0,
    "WEIGHT_SAFE_TERRITORY": 23.088,
    "WEIGHT_COMPACTNESS": 12.973,
    "WEIGHT_SPATIAL_MATRIX": 0.322,
    "WEIGHT_FRONTIER": 17.307,
    "WEIGHT_OPP_CHOKE": 18.0,
    "WEIGHT_BOTTLENECK": 12.0,
}

# Spatial matrix clamping boundaries
SPATIAL_MIN: float = 0.0
SPATIAL_MAX: float = 50.0

# Authoritative Anchor Set: 100 fixed seed pairs (1000..1099, 200 matches)
ANCHOR_SEEDS: List[int] = list(range(1000, 1100))


# ==============================================================================
# Clamping & Parameter Helpers
# ==============================================================================

def clamp_param_value(name: str, value: float) -> float:
    """Clamps a single named parameter to its designated corridor."""
    if name == "WEIGHT_AREA":
        return 50.0
    low, high = CLAMPING_CORRIDORS.get(name, (0.0, 1000.0))
    return max(low, min(high, float(value)))


def clamp_params_list(params: Sequence[float]) -> List[float]:
    """Clamps a 10-element parameter list to canonical corridors, locking WEIGHT_AREA = 50.0."""
    clamped: List[float] = []
    for i, name in enumerate(PARAM_NAMES):
        raw_val = params[i] if i < len(params) else DEFAULT_WEIGHTS[name]
        clamped.append(clamp_param_value(name, raw_val))
    return clamped


def clamp_params_dict(params: Dict[str, float]) -> Dict[str, float]:
    """Clamps a parameter dictionary to canonical corridors."""
    clamped: Dict[str, float] = {}
    for name in PARAM_NAMES:
        raw_val = params.get(name, DEFAULT_WEIGHTS[name])
        clamped[name] = clamp_param_value(name, raw_val)
    return clamped


def is_params_clamped(params: Sequence[float] | Dict[str, float], tol: float = 1e-6) -> bool:
    """Checks whether all parameters strictly satisfy clamping corridor invariants."""
    if isinstance(params, dict):
        for name, (low, high) in CLAMPING_CORRIDORS.items():
            val = params.get(name, None)
            if val is None or val < low - tol or val > high + tol:
                return False
        return True

    if len(params) != len(PARAM_NAMES):
        return False

    for i, name in enumerate(PARAM_NAMES):
        low, high = CLAMPING_CORRIDORS[name]
        val = params[i]
        if val < low - tol or val > high + tol:
            return False
    return True


# ==============================================================================
# Parameter Mutation
# ==============================================================================

def mutate_params(
    params: Sequence[float],
    mutation_scale: float = 0.1,
    mutation_rate: float = 0.7,
    rng: Optional[random.Random] = None,
) -> List[float]:
    """Mutates heuristic parameters with Gaussian perturbations, strictly locking WEIGHT_AREA = 50.0.

    Args:
        params: 10-element parameter list in canonical order.
        mutation_scale: Standard deviation of perturbation relative to corridor span.
        mutation_rate: Probability of mutating each mutable parameter.
        rng: Random number generator instance (optional).

    Returns:
        New 10-element clamped parameter list.
    """
    if rng is None:
        rng = random.Random()

    mutated = list(params)
    # 1. WEIGHT_AREA (index 0) is strictly locked to 50.0 (Invariant Numeraire)
    mutated[0] = 50.0

    # 2. Mutate remaining 9 parameters within their clamping corridors
    for i in range(1, len(PARAM_NAMES)):
        name = PARAM_NAMES[i]
        low, high = CLAMPING_CORRIDORS[name]
        span = high - low

        if rng.random() < mutation_rate:
            sigma = mutation_scale * span
            delta = rng.gauss(0.0, sigma)
            mutated[i] += delta

        # Strict clamping corridor enforcement
        mutated[i] = max(low, min(high, mutated[i]))

    return mutated


# ==============================================================================
# Spatial Matrix Operations & Radial Gaussian Bumps
# ==============================================================================

def symmetrize_spatial_matrix(matrix: Sequence[float]) -> List[float]:
    """Enforces exact bit-level 180° point-inversion symmetry on a 600-element spatial matrix.

    For every cell (r, c), its mirrored twin is (14 - r, 39 - c).
    """
    if len(matrix) != TOTAL_CELLS:
        raise ValueError(f"Expected {TOTAL_CELLS} cells, got {len(matrix)}")

    sym = list(matrix)
    for r in range(BOARD_HEIGHT):
        r_mirror = BOARD_HEIGHT - 1 - r
        for c in range(BOARD_WIDTH):
            c_mirror = BOARD_WIDTH - 1 - c
            idx1 = r * BOARD_WIDTH + c
            idx2 = r_mirror * BOARD_WIDTH + c_mirror
            avg_val = (sym[idx1] + sym[idx2]) * 0.5
            sym[idx1] = avg_val
            sym[idx2] = avg_val
    return sym


def verify_matrix_symmetry(matrix: Sequence[float], tol: float = 1e-9) -> bool:
    """Verifies that a spatial matrix satisfies 180° point-inversion symmetry: M[r, c] == M[14 - r, 39 - c]."""
    if len(matrix) != TOTAL_CELLS:
        return False
    for r in range(BOARD_HEIGHT):
        r_mirror = BOARD_HEIGHT - 1 - r
        for c in range(BOARD_WIDTH):
            c_mirror = BOARD_WIDTH - 1 - c
            v1 = matrix[r * BOARD_WIDTH + c]
            v2 = matrix[r_mirror * BOARD_WIDTH + c_mirror]
            if abs(v1 - v2) > tol:
                return False
    return True


def apply_gaussian_domes(
    matrix: Sequence[float],
    num_domes: Optional[int] = None,
    sigma_range: Tuple[float, float] = (2.5, 4.0),
    delta_range: Tuple[float, float] = (0.5, 2.5),
    rng: Optional[random.Random] = None,
) -> List[float]:
    """Applies 2-3 local 2D Gaussian domes with paired 180° point-inversion mirroring.

    Independent white noise per cell is strictly prohibited.
    Values are clamped to [0.0, 50.0].

    Args:
        matrix: 600-element spatial prior matrix.
        num_domes: Number of dome pairs (default: random 2 or 3).
        sigma_range: Radius bounds (sigma_r ~ 2.5..4.0 cells).
        delta_range: Amplitude bounds (|Delta| ~ 0.5..2.5).
        rng: Random generator instance.

    Returns:
        Mutated, symmetric, and clamped 600-element spatial matrix.
    """
    if len(matrix) != TOTAL_CELLS:
        raise ValueError(f"Expected {TOTAL_CELLS} cells, got {len(matrix)}")

    if rng is None:
        rng = random.Random()

    if num_domes is None:
        num_domes = rng.randint(2, 3)

    out = list(matrix)

    for _ in range(num_domes):
        # Choose random dome center (r_c, c_c)
        r_c = rng.uniform(0.0, float(BOARD_HEIGHT - 1))
        c_c = rng.uniform(0.0, float(BOARD_WIDTH - 1))
        sigma = rng.uniform(sigma_range[0], sigma_range[1])
        amplitude = rng.uniform(delta_range[0], delta_range[1]) * rng.choice([-1.0, 1.0])

        # Mirrored center under 180° point inversion
        r_sym = float(BOARD_HEIGHT - 1) - r_c
        c_sym = float(BOARD_WIDTH - 1) - c_c
        two_sigma_sq = 2.0 * sigma * sigma

        # Radial 2D Gaussian perturbation added symmetrically to both centers
        for r in range(BOARD_HEIGHT):
            for c in range(BOARD_WIDTH):
                idx = r * BOARD_WIDTH + c
                d1_sq = (float(r) - r_c) ** 2 + (float(c) - c_c) ** 2
                d2_sq = (float(r) - r_sym) ** 2 + (float(c) - c_sym) ** 2
                bump = amplitude * (math.exp(-d1_sq / two_sigma_sq) + math.exp(-d2_sq / two_sigma_sq))
                out[idx] += bump

    # Enforce exact bit-level symmetry and clamp to [0.0, 50.0]
    out = symmetrize_spatial_matrix(out)
    for i in range(TOTAL_CELLS):
        out[i] = max(SPATIAL_MIN, min(SPATIAL_MAX, out[i]))

    # Final symmetry pass to guarantee bit-exact floating point equality after clamping
    return symmetrize_spatial_matrix(out)


# ==============================================================================
# Fitness Calculation
# ==============================================================================

def calculate_fitness(winrate_pct: float, avg_margin: float) -> float:
    """Exact fitness function per requirement: Fitness = WinRate_% + 0.02 * AvgMargin.

    Args:
        winrate_pct: Win rate in percentage [0.0, 100.0].
        avg_margin: Average score margin in [-600.0, +600.0].

    Returns:
        Computed fitness score where win rate is primary and +50 margin = +1.0% fitness.
    """
    return float(winrate_pct) + 0.02 * float(avg_margin)


def fitness_from_stats(stats: Dict[str, Any]) -> float:
    """Computes fitness from FFI candidate statistics dictionary."""
    winrate = float(stats.get("winrate", 0.0))
    winrate_pct = winrate * 100.0

    total_matches = stats.get("total_matches", 0)
    if total_matches > 0 and "net_margin" in stats:
        avg_margin = float(stats["net_margin"]) / float(total_matches)
    else:
        avg_margin = float(stats.get("avg_margin", 0.0))

    return calculate_fitness(winrate_pct, avg_margin)


# ==============================================================================
# Candidate Representation & Procedural Matrix
# ==============================================================================

def generate_procedural_matrix() -> List[float]:
    """Generates the standard 600-element spatial prior matrix with strict 180° symmetry."""
    matrix = [10.0] * TOTAL_CELLS
    for r in range(BOARD_HEIGHT):
        for c in range(BOARD_WIDTH):
            idx = r * BOARD_WIDTH + c
            weight = 10.0
            dist_center_col = abs(float(c) - 19.5)
            dist_center_row = abs(float(r) - 7.0)
            if dist_center_col <= 3.0:
                weight += (4.0 - dist_center_col) * 8.0
            if dist_center_row <= 3.0:
                weight += (4.0 - dist_center_row) * 4.0
            if r == 0 or r == BOARD_HEIGHT - 1:
                weight += 6.0
            matrix[idx] = weight
    matrix = symmetrize_spatial_matrix(matrix)
    for i in range(TOTAL_CELLS):
        matrix[i] = max(SPATIAL_MIN, min(SPATIAL_MAX, matrix[i]))
    return symmetrize_spatial_matrix(matrix)


@dataclass
class Candidate:
    """Encapsulates a model candidate with heuristic parameters, spatial matrix, and evaluation metrics."""
    params: List[float]
    matrix: List[float]
    metadata: Dict[str, Any] = field(default_factory=dict)
    fitness: float = 0.0
    winrate: float = 0.0
    avg_margin: float = 0.0

    def __post_init__(self):
        if len(self.params) != len(PARAM_NAMES):
            self.params = clamp_params_list(self.params)
        if len(self.matrix) != TOTAL_CELLS:
            self.matrix = symmetrize_spatial_matrix(self.matrix)

    def clone(self) -> Candidate:
        """Deep copy of candidate."""
        return Candidate(
            params=list(self.params),
            matrix=list(self.matrix),
            metadata=dict(self.metadata),
            fitness=self.fitness,
            winrate=self.winrate,
            avg_margin=self.avg_margin,
        )

    def clamp(self) -> None:
        """Enforces clamping corridors on parameters and spatial matrix."""
        self.params = clamp_params_list(self.params)
        self.matrix = [max(SPATIAL_MIN, min(SPATIAL_MAX, x)) for x in self.matrix]
        self.matrix = symmetrize_spatial_matrix(self.matrix)

    def is_clamped(self) -> bool:
        """Validates that parameters and matrix are within legal corridors."""
        if not is_params_clamped(self.params):
            return False
        for x in self.matrix:
            if x < SPATIAL_MIN or x > SPATIAL_MAX:
                return False
        return True

    def is_symmetric(self, tol: float = 1e-9) -> bool:
        """Validates that spatial matrix satisfies 180° point-inversion symmetry."""
        return verify_matrix_symmetry(self.matrix, tol)

    def params_dict(self) -> Dict[str, float]:
        """Returns parameters as a dictionary mapped to canonical names."""
        return {PARAM_NAMES[i]: self.params[i] for i in range(len(PARAM_NAMES))}

    @classmethod
    def from_default(cls) -> Candidate:
        """Constructs a candidate with default baseline parameters and procedural matrix."""
        params = [DEFAULT_WEIGHTS[name] for name in PARAM_NAMES]
        matrix = generate_procedural_matrix()
        return cls(params=params, matrix=matrix, metadata={"epoch": 0})

    @classmethod
    def from_weights_json(cls, path: str) -> Candidate:
        """Constructs a candidate by loading a weights.json file via pyrust FFI or json parsing."""
        if hasattr(pyrust, "load_weights_file"):
            params, matrix, meta = pyrust.load_weights_file(path)
            return cls(params=params, matrix=matrix, metadata=meta)
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        weights_dict = data.get("weights", {})
        params = [float(weights_dict.get(name, DEFAULT_WEIGHTS[name])) for name in PARAM_NAMES]
        matrix = [float(v) for v in data.get("matrix", [])]
        return cls(params=params, matrix=matrix, metadata=data.get("metadata", {}))

    def save_to_json(self, path: str) -> None:
        """Saves candidate weights, spatial matrix, and metadata to JSON via pyrust.save_weights_file."""
        if hasattr(pyrust, "save_weights_file"):
            pyrust.save_weights_file(path, self.params, self.matrix, self.metadata)
        else:
            payload = {
                "architecture": "Argon-4 / Phase-Adaptive + Loop Denial",
                "version": "2.0.0",
                "weights": self.params_dict(),
                "matrix": self.matrix,
                "metadata": self.metadata,
            }
            with open(path, "w", encoding="utf-8") as f:
                json.dump(payload, f, indent=2)


# ==============================================================================
# Anchor Set Validation & Multi-Archetype Sentinel
# ==============================================================================

@dataclass
class ValidationResult:
    """Outcome of Anchor Set multi-archetype validation."""
    passed: bool
    cand_fitness: float
    champ_fitness: float
    cand_stats: Dict[str, Any]
    champ_stats: Dict[str, Any]
    reason: str


class AnchorValidator:
    """Performs multi-archetype Anchor Set examinations.

    Protects against self-overfitting and rock-paper-scissors cycles by evaluating
    the candidate and champion against:
    1. The target training opponent across 100 CRN seed pairs (1000..1099, 200 matches).
    2. Fixed archetype slicing vs GreedyBot and LegacyHardBot.
    Candidate is crowned only if overall Anchor fitness strictly improves with no regression.
    """

    def __init__(self, anchor_seeds: Optional[List[int]] = None, num_workers: int = 4):
        self.anchor_seeds = anchor_seeds if anchor_seeds is not None else list(ANCHOR_SEEDS)
        self.num_workers = num_workers

    def validate(
        self,
        candidate: Candidate,
        champion: Candidate,
        target_opponent: str = "GreedyBot",
        min_greedy_winrate: float = 0.95,
        min_hard_winrate: float = 0.65,
        min_baseline_winrate: float = 0.50,
    ) -> ValidationResult:
        """Executes the Anchor Set examination on candidate vs champion.

        Returns:
            ValidationResult indicating whether candidate successfully replaces champion.
        """
        # Batch evaluation: evaluate champion (idx 0) and candidate (idx 1) in parallel
        cands_params = [champion.params, candidate.params]
        cands_mats = [champion.matrix, candidate.matrix]

        # 1. Target opponent evaluation on Anchor Set
        target_results = pyrust.evaluate_population_ffi(
            cands_params, cands_mats, target_opponent, self.anchor_seeds, self.num_workers
        )
        champ_target = target_results[0]
        cand_target = target_results[1]

        champ_fit_target = fitness_from_stats(champ_target)
        cand_fit_target = fitness_from_stats(cand_target)

        # 2. Archetype benchmark: GreedyBot evaluation
        if target_opponent.lower() == "greedybot" or "greedy" in target_opponent.lower():
            champ_greedy = champ_target
            cand_greedy = cand_target
        else:
            greedy_results = pyrust.evaluate_population_ffi(
                cands_params, cands_mats, "GreedyBot", self.anchor_seeds, self.num_workers
            )
            champ_greedy = greedy_results[0]
            cand_greedy = greedy_results[1]

        # 3. Archetype benchmark: LegacyHardBot evaluation
        if target_opponent.lower() in ("hardbot", "legacyhardbot", "hard"):
            champ_hard = champ_target
            cand_hard = cand_target
        else:
            hard_results = pyrust.evaluate_population_ffi(
                cands_params, cands_mats, "LegacyHardBot", self.anchor_seeds, self.num_workers
            )
            champ_hard = hard_results[0]
            cand_hard = hard_results[1]

        # 4. Archetype benchmark: Baseline V2 (SpatialSlayer with default unmutated weights)
        if target_opponent.lower() in ("slayer", "spatial", "spatialsplayer", "v2", "baselinev2"):
            champ_v2 = champ_target
            cand_v2 = cand_target
        else:
            try:
                v2_results = pyrust.evaluate_population_ffi(
                    cands_params, cands_mats, "SpatialSlayer", self.anchor_seeds, self.num_workers
                )
                champ_v2 = v2_results[0]
                cand_v2 = v2_results[1]
            except (StopIteration, IndexError):
                champ_v2 = champ_target
                cand_v2 = cand_target

        champ_greedy_wr = float(champ_greedy["winrate"])
        cand_greedy_wr = float(cand_greedy["winrate"])
        champ_hard_wr = float(champ_hard["winrate"])
        cand_hard_wr = float(cand_hard["winrate"])
        champ_v2_wr = float(champ_v2["winrate"])
        cand_v2_wr = float(cand_v2["winrate"])

        champ_fit_v2 = fitness_from_stats(champ_v2)
        cand_fit_v2 = fitness_from_stats(cand_v2)
        champ_fit_hard = fitness_from_stats(champ_hard)
        cand_fit_hard = fitness_from_stats(cand_hard)
        champ_fit_greedy = fitness_from_stats(champ_greedy)
        cand_fit_greedy = fitness_from_stats(cand_greedy)

        # Composite multi-archetype anchor score including Baseline V2
        champ_composite_fit = (
            0.40 * champ_fit_target
            + 0.25 * champ_fit_v2
            + 0.20 * champ_fit_hard
            + 0.15 * champ_fit_greedy
        )
        cand_composite_fit = (
            0.40 * cand_fit_target
            + 0.25 * cand_fit_v2
            + 0.20 * cand_fit_hard
            + 0.15 * cand_fit_greedy
        )

        # Regression gate checks:
        # A) Must strictly outperform champion on composite fitness
        if cand_composite_fit <= champ_composite_fit:
            return ValidationResult(
                passed=False,
                cand_fitness=cand_composite_fit,
                champ_fitness=champ_composite_fit,
                cand_stats=cand_target,
                champ_stats=champ_target,
                reason=f"Failed Anchor Set: Cand fitness ({cand_composite_fit:.2f}) <= Champ ({champ_composite_fit:.2f})",
            )

        # B) Must not regress against GreedyBot (floor 95% or champ - 2%)
        greedy_floor = min(champ_greedy_wr - 0.02, min_greedy_winrate)
        if cand_greedy_wr < greedy_floor:
            return ValidationResult(
                passed=False,
                cand_fitness=cand_composite_fit,
                champ_fitness=champ_composite_fit,
                cand_stats=cand_target,
                champ_stats=champ_target,
                reason=f"GreedyBot regression: cand {cand_greedy_wr*100:.1f}% < floor {greedy_floor*100:.1f}%",
            )

        # C) Must not severely regress against HardBot
        hard_floor = min(champ_hard_wr - 0.05, min_hard_winrate)
        if cand_hard_wr < hard_floor:
            return ValidationResult(
                passed=False,
                cand_fitness=cand_composite_fit,
                champ_fitness=champ_composite_fit,
                cand_stats=cand_target,
                champ_stats=champ_target,
                reason=f"LegacyHardBot regression: cand {cand_hard_wr*100:.1f}% < floor {hard_floor*100:.1f}%",
            )

        # D) Must not regress against Baseline V2 (SpatialSlayer default)
        v2_floor = min(champ_v2_wr - 0.05, min_baseline_winrate)
        if cand_v2_wr < v2_floor:
            return ValidationResult(
                passed=False,
                cand_fitness=cand_composite_fit,
                champ_fitness=champ_composite_fit,
                cand_stats=cand_target,
                champ_stats=champ_target,
                reason=f"Baseline V2 regression: cand {cand_v2_wr*100:.1f}% < floor {v2_floor*100:.1f}%",
            )

        return ValidationResult(
            passed=True,
            cand_fitness=cand_composite_fit,
            champ_fitness=champ_composite_fit,
            cand_stats={
                "target": cand_target,
                "greedy": cand_greedy,
                "hard": cand_hard,
                "v2": cand_v2,
                "composite_fitness": cand_composite_fit,
            },
            champ_stats={
                "target": champ_target,
                "greedy": champ_greedy,
                "hard": champ_hard,
                "v2": champ_v2,
                "composite_fitness": champ_composite_fit,
            },
            reason="Approved: Strictly exceeded champion on Anchor Set without archetype regression",
        )


# ==============================================================================
# Weight Evolution Delta Reporting
# ==============================================================================

def generate_weight_delta_table(
    initial_params: Sequence[float],
    final_params: Sequence[float],
    initial_matrix: Sequence[float],
    final_matrix: Sequence[float],
    gen_from: int = 0,
    gen_to: int = 40,
    extra_metrics: Optional[Dict[str, Any]] = None,
) -> str:
    """Generates an ASCII delta report table comparing initial vs evolved weights and spatial matrix."""
    lines: List[str] = []
    lines.append("=" * 105)
    lines.append(f"WEIGHT EVOLUTION DELTA REPORT: Gen {gen_from} -> Gen {gen_to}")
    lines.append("=" * 105)
    lines.append(
        f"{'Parameter':<24} {'Gen ' + str(gen_from):>10} {'Gen ' + str(gen_to):>10} "
        f"{'Delta':>10} {'% Change':>10} {'Corridor Min':>14} {'Corridor Max':>14}"
    )
    lines.append("-" * 105)

    for i, name in enumerate(PARAM_NAMES):
        v0 = initial_params[i]
        v1 = final_params[i]
        delta = v1 - v0
        pct = (delta / v0 * 100.0) if abs(v0) > 1e-9 else 0.0
        low, high = CLAMPING_CORRIDORS[name]
        lines.append(
            f"{name:<24} {v0:>10.3f} {v1:>10.3f} {delta:>+10.3f} {pct:>+9.1f}% {low:>14.3f} {high:>14.3f}"
        )

    lines.append("-" * 105)

    # Matrix analysis
    m0_min, m0_max = min(initial_matrix), max(initial_matrix)
    m1_min, m1_max = min(final_matrix), max(final_matrix)
    m0_mean = sum(initial_matrix) / float(len(initial_matrix))
    m1_mean = sum(final_matrix) / float(len(final_matrix))

    # Center zone relief (rows 6..8, cols 18..21)
    center_indices = [r * BOARD_WIDTH + c for r in range(6, 9) for c in range(18, 22)]
    m0_center = sum(initial_matrix[i] for i in center_indices) / len(center_indices)
    m1_center = sum(final_matrix[i] for i in center_indices) / len(center_indices)

    # Bottleneck columns (c in (10, 20, 30))
    bn_indices = [r * BOARD_WIDTH + c for r in range(BOARD_HEIGHT) for c in (10, 20, 30)]
    m0_bn = sum(initial_matrix[i] for i in bn_indices) / len(bn_indices)
    m1_bn = sum(final_matrix[i] for i in bn_indices) / len(bn_indices)

    # Ceiling saturation (cells >= 50.0 - 1e-6)
    m0_saturated = sum(1 for v in initial_matrix if v >= SPATIAL_MAX - 1e-6)
    m1_saturated = sum(1 for v in final_matrix if v >= SPATIAL_MAX - 1e-6)

    l2_delta = math.sqrt(sum((b - a) ** 2 for a, b in zip(initial_matrix, final_matrix)))
    max_cell_delta = max(abs(b - a) for a, b in zip(initial_matrix, final_matrix))

    sym0 = verify_matrix_symmetry(initial_matrix)
    sym1 = verify_matrix_symmetry(final_matrix)

    lines.append("SPATIAL MATRIX RELIEF PROFILE:")
    lines.append(
        f"  Gen {gen_from}: min={m0_min:.3f}, max={m0_max:.3f}, mean={m0_mean:.3f}, "
        f"center={m0_center:.3f}, bottlenecks(c=10,20,30)={m0_bn:.3f}, "
        f"saturated={m0_saturated}/600 ({m0_saturated/6.0:.1f}%), 180° symmetry: {'STRICT' if sym0 else 'VIOLATED'}"
    )
    lines.append(
        f"  Gen {gen_to}: min={m1_min:.3f}, max={m1_max:.3f}, mean={m1_mean:.3f}, "
        f"center={m1_center:.3f}, bottlenecks(c=10,20,30)={m1_bn:.3f}, "
        f"saturated={m1_saturated}/600 ({m1_saturated/6.0:.1f}%), 180° symmetry: {'STRICT' if sym1 else 'VIOLATED'}"
    )
    lines.append(f"  Matrix L2 norm delta: {l2_delta:.4f} | Peak cell delta: {max_cell_delta:.4f}")

    if extra_metrics:
        lines.append("-" * 105)
        lines.append("EVOLUTIONARY METRICS & TOP-2 ELITISM:")
        for k, v in extra_metrics.items():
            lines.append(f"  {k}: {v}")

    lines.append("=" * 105)

    return "\n".join(lines)


# ==============================================================================
# Evolutionary Training Engine
# ==============================================================================

class EvolutionaryEngine:
    """Evolutionary training engine coordinating generation loops, population evaluation,

    Anchor Set multi-archetype exams, Top-2 elitism, and weights.json persistence.
    """

    def __init__(
        self,
        lambda_size: int = 16,
        generations: int = 30,
        seeds_per_eval: int = 20,
        opponent: str = "GreedyBot",
        weights_path: str = "weights.json",
        output_path: str = "weights.json",
        curriculum: Optional[str] = None,
        curriculum_switch: Optional[int] = None,
        mutation_scale: float = 0.1,
        spatial_mutation_prob: float = 0.5,
        seed: int = 42,
        num_workers: int = 4,
        max_turns: int = 80,
    ):
        self.lambda_size = lambda_size
        self.generations = generations
        self.seeds_per_eval = seeds_per_eval
        self.opponent = opponent
        self.weights_path = weights_path
        self.output_path = output_path
        self.curriculum = curriculum
        self.curriculum_switch = curriculum_switch
        self.mutation_scale = mutation_scale
        self.spatial_mutation_prob = spatial_mutation_prob
        self.seed = seed
        self.num_workers = num_workers
        self.max_turns = max_turns

        self.rng = random.Random(seed)
        self.validator = AnchorValidator(num_workers=num_workers)

        # Champion model
        self.champion = self._load_initial_champion()
        self.gen0_snapshot: Optional[Candidate] = None
        self.history: List[Dict[str, Any]] = []
        self.top2_fallback_activations: int = 0
        self.top2_promotions_count: int = 0

    def _load_initial_champion(self) -> Candidate:
        """Loads champion from weights.json or initializes default baseline."""
        if os.path.exists(self.weights_path):
            try:
                params_list, mat_list, meta = pyrust.load_weights_file(self.weights_path)
                cand = Candidate(params=params_list, matrix=mat_list, metadata=dict(meta))
                cand.clamp()
                return cand
            except Exception as e:
                print(f"[Warning] Failed to load '{self.weights_path}': {e}. Using default baseline.")

        return Candidate.from_default()

    def _get_active_opponent(self, gen_idx: int) -> str:
        """Determines active sparring opponent based on generation and curriculum settings."""
        if self.curriculum_switch is not None:
            return self.opponent if gen_idx < self.curriculum_switch else "LegacyHardBot"

        if not self.curriculum:
            return self.opponent

        curr_lower = self.curriculum.lower().strip()
        if curr_lower in ("true", "standard", "greedy_then_hard"):
            # Phase 1 (first 50% generations): GreedyBot; Phase 2 (second 50%): LegacyHardBot
            half = self.generations // 2
            return "GreedyBot" if gen_idx < half else "LegacyHardBot"

        if "," in self.curriculum:
            # Custom comma-separated curriculum stages
            stages = [s.strip() for s in self.curriculum.split(",") if s.strip()]
            stage_len = max(1, self.generations // len(stages))
            stage_idx = min(len(stages) - 1, gen_idx // stage_len)
            return stages[stage_idx]

        return self.curriculum

    def _evaluate_generation_promotions(
        self,
        champ_gen_fit: float,
        scored_mutants: List[Tuple[Candidate, Dict[str, Any], float]],
        active_opp: str,
    ) -> Tuple[Optional[Candidate], Optional[ValidationResult], str]:
        """Evaluates Top-1 candidate and applies Top-2 elitism fallback if eligible.

        Args:
            champ_gen_fit: Generation fitness of the current champion on gen_seeds.
            scored_mutants: Ranked list of (mutant_candidate, raw_eval_stats, gen_fitness).
            active_opp: Active opponent bot type for the current generation.

        Returns:
            Tuple of (promoted_candidate_or_None, validation_result_or_None, reason_str).
        """
        if not scored_mutants:
            return None, None, "No mutants to evaluate"

        top1_mutant, top1_res, top1_fit = scored_mutants[0]
        top2_mutant, top2_res, top2_fit = (
            scored_mutants[1] if len(scored_mutants) > 1 else (None, None, -f64_inf())
        )

        if top1_fit > champ_gen_fit:
            # Test Top-1 mutant on Anchor Set
            v1 = self.validator.validate(top1_mutant, self.champion, target_opponent=active_opp)
            if v1.passed:
                return top1_mutant, v1, f"Top-1 promoted: {v1.reason}"
            else:
                # Top-1 failed Anchor Set.
                # Check Top-2 Elitism: if Top-1 had high winrate (>60%) and Top-2 beat champion
                if (
                    top1_mutant.winrate > 0.60
                    and top2_mutant is not None
                    and top2_fit > champ_gen_fit
                ):
                    self.top2_fallback_activations += 1
                    v2 = self.validator.validate(
                        top2_mutant, self.champion, target_opponent=active_opp
                    )
                    if v2.passed:
                        self.top2_promotions_count += 1
                        return (
                            top2_mutant,
                            v2,
                            f"Top-2 promoted (Top-1 had {top1_mutant.winrate*100:.1f}% gen WR): {v2.reason}",
                        )

        return None, None, "No candidate qualified for promotion"

    def save_champion(self, path: Optional[str] = None) -> None:
        """Saves current champion to weights.json conforming 100% to JS browser build."""
        target_path = path or self.output_path
        self.champion.clamp()

        meta_dict = {
            "epoch": self.champion.metadata.get("epoch", 0),
            "winrate_vs_hard": float(self.champion.metadata.get("winrate_vs_hard", 0.0)),
            "winrate_vs_greedy": float(self.champion.metadata.get("winrate_vs_greedy", 0.0)),
            "winrate_vs_champ": float(self.champion.metadata.get("winrate_vs_champ", 0.0)),
            "avg_margin": float(self.champion.avg_margin),
            "fitness": float(self.champion.fitness),
        }

        pyrust.save_weights_file(
            target_path,
            self.champion.params,
            self.champion.matrix,
            meta_dict,
        )

    def run(self) -> Tuple[Candidate, List[Dict[str, Any]]]:
        """Executes the full evolutionary training loop."""
        print("=" * 80)
        print("STARTING PYRUST EVOLUTIONARY ENGINE")
        print(f"Generations: {self.generations} | Population: lambda={self.lambda_size} | "
              f"Seeds/Gen: {self.seeds_per_eval} | Opponent: {self.opponent}")
        print(f"Numeraire: WEIGHT_AREA = 50.0 (locked) | Workers: {self.num_workers}")
        print("=" * 80)

        # Record Gen 0 snapshot for delta report
        self.gen0_snapshot = self.champion.clone()

        # Initial baseline evaluation of champion
        init_seeds = [self.seed + s for s in range(self.seeds_per_eval)]
        active_opp = self._get_active_opponent(0)
        init_res = pyrust.evaluate_population_ffi(
            [self.champion.params],
            [self.champion.matrix],
            active_opp,
            init_seeds,
            self.num_workers,
            self.max_turns,
        )[0]

        self.champion.winrate = float(init_res["winrate"])
        total_m = init_res["total_matches"]
        self.champion.avg_margin = float(init_res["net_margin"]) / float(total_m) if total_m > 0 else 0.0

        # Anchor Set baseline fitness standardization
        init_val = self.validator.validate(self.champion, self.champion, target_opponent=active_opp)
        self.champion.fitness = init_val.champ_fitness
        self.champion.metadata["epoch"] = 0

        print(f"[Gen 0 Baseline] Opponent: {active_opp} | WinRate: {self.champion.winrate*100:.1f}% | "
              f"AvgMargin: {self.champion.avg_margin:+.1f} | Fitness: {self.champion.fitness:.2f}")

        # Training Generation Loop
        for gen in range(1, self.generations + 1):
            t_gen_start = time.time()
            active_opp = self._get_active_opponent(gen - 1)

            # 1. Generate lambda mutants from champion
            mutants: List[Candidate] = []
            for _ in range(self.lambda_size):
                m_params = mutate_params(
                    self.champion.params,
                    mutation_scale=self.mutation_scale,
                    mutation_rate=0.7,
                    rng=self.rng,
                )

                if self.rng.random() < self.spatial_mutation_prob:
                    m_matrix = apply_gaussian_domes(
                        self.champion.matrix,
                        num_domes=None,
                        rng=self.rng,
                    )
                else:
                    m_matrix = list(self.champion.matrix)

                cand = Candidate(params=m_params, matrix=m_matrix)
                cand.clamp()
                mutants.append(cand)

            # 2. Assemble evaluation batch: [Champion] + Mutants
            batch_candidates = [self.champion] + mutants
            batch_params = [c.params for c in batch_candidates]
            batch_matrices = [c.matrix for c in batch_candidates]

            # Generate unique CRN seeds for this generation
            gen_seeds = [self.seed + gen * 1000 + s for s in range(self.seeds_per_eval)]

            # 3. Parallel Rayon batch evaluation (GIL released)
            batch_results = pyrust.evaluate_population_ffi(
                batch_params,
                batch_matrices,
                active_opp,
                gen_seeds,
                self.num_workers,
                self.max_turns,
            )

            # 4. Extract metrics and compute exact fitness
            champ_eval = batch_results[0]
            champ_gen_fit = fitness_from_stats(champ_eval)
            champ_gen_wr = float(champ_eval["winrate"])

            scored_mutants: List[Tuple[Candidate, Dict[str, Any], float]] = []
            for idx, mutant in enumerate(mutants):
                res = batch_results[idx + 1]
                mut_fit = fitness_from_stats(res)
                mutant.fitness = mut_fit
                mutant.winrate = float(res["winrate"])
                mut_matches = res["total_matches"]
                mutant.avg_margin = float(res["net_margin"]) / float(mut_matches) if mut_matches > 0 else 0.0
                scored_mutants.append((mutant, res, mut_fit))

            # Rank mutants descending by generation fitness
            scored_mutants.sort(key=lambda item: item[2], reverse=True)

            top1_mutant, top1_res, top1_fit = scored_mutants[0]
            top2_mutant, top2_res, top2_fit = (
                scored_mutants[1] if len(scored_mutants) > 1 else (None, None, -f64_inf())
            )

            # 5. Anchor Set Validation with Top-2 Elitism
            new_champ_candidate, val_result, promotion_reason = self._evaluate_generation_promotions(
                champ_gen_fit, scored_mutants, active_opp
            )
            promoted = (
                new_champ_candidate is not None
                and val_result is not None
                and val_result.passed
            )

            # 6. Apply Promotion if validated
            if promoted and new_champ_candidate is not None and val_result is not None:
                old_fit = self.champion.fitness
                # BUG-M4-01: Standardize candidate fitness to Anchor Set fitness before promotion
                new_champ_candidate.fitness = val_result.cand_fitness
                self.champion = new_champ_candidate.clone()
                self.champion.metadata["epoch"] = gen
                self.champion.metadata["winrate_vs_target"] = self.champion.winrate
                self.save_champion()
                print(
                    f"  >>> [Gen {gen:02d} PROMOTION] Fitness: {old_fit:.2f} -> {self.champion.fitness:.2f} | "
                    f"WinRate: {self.champion.winrate*100:.1f}% | {promotion_reason}"
                )

            gen_elapsed = time.time() - t_gen_start
            rss_mb = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / (1024.0 * 1024.0)

            self.history.append({
                "generation": gen,
                "opponent": active_opp,
                "champ_fitness": self.champion.fitness,
                "champ_winrate": self.champion.winrate,
                "top1_fit": top1_fit,
                "top1_winrate": top1_mutant.winrate,
                "promoted": promoted,
                "elapsed": gen_elapsed,
                "ram_mb": rss_mb,
            })

            if gen % 5 == 0 or gen == 1 or promoted:
                print(
                    f"[Gen {gen:02d}/{self.generations}] Opp: {active_opp:<10} | "
                    f"Champ Fit: {self.champion.fitness:6.2f} (WR: {self.champion.winrate*100:5.1f}%) | "
                    f"Top-1 Fit: {top1_fit:6.2f} (WR: {top1_mutant.winrate*100:5.1f}%) | "
                    f"Time: {gen_elapsed:4.1f}s | RAM: {rss_mb:.1f}MB"
                )

        # 7. Post-Training Reporting: Delta Table (Gen 0 vs Final)
        if self.gen0_snapshot is not None:
            final_rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / (1024.0 * 1024.0)
            extra_metrics = {
                "Top-2 Fallback Activations": self.top2_fallback_activations,
                "Top-2 Candidate Promotions": self.top2_promotions_count,
                "Memory Usage (RAM RSS)": f"{final_rss:.2f} MB",
            }
            delta_report = generate_weight_delta_table(
                self.gen0_snapshot.params,
                self.champion.params,
                self.gen0_snapshot.matrix,
                self.champion.matrix,
                gen_from=0,
                gen_to=self.generations,
                extra_metrics=extra_metrics,
            )
            print("\n" + delta_report)

        # Final persistence check
        self.save_champion()
        print("\nTraining complete. Model weights saved to:", self.output_path)
        return self.champion, self.history


def f64_inf() -> float:
    return float("inf")


# ==============================================================================
# CLI Entry Point
# ==============================================================================

def main():
    parser = argparse.ArgumentParser(description="PyRust Territories AI Evolutionary Training Engine")
    parser.add_argument("--generations", type=int, default=30, help="Number of evolutionary generations")
    parser.add_argument("--lambda-size", "--pop-size", dest="lambda_size", type=int, default=16, help="Population mutants per generation")
    parser.add_argument("--seeds-per-eval", "--games-per-eval", dest="seeds_per_eval", type=int, default=20, help="Seed pairs per generation evaluation")
    parser.add_argument("--opponent", type=str, default="GreedyBot", help="Opponent: GreedyBot, FastBot, LegacyHardBot, SpatialSlayer")
    parser.add_argument("--weights-path", type=str, default="weights.json", help="Path to input weights.json")
    parser.add_argument("--output-path", "--export-weights", dest="output_path", type=str, default="weights.json", help="Path to output weights.json")
    parser.add_argument("--curriculum", type=str, default=None, help="Curriculum: standard, greedy_then_hard, or comma-separated list")
    parser.add_argument("--curriculum-switch", type=int, default=None, help="Generation to switch opponent in curriculum")
    parser.add_argument("--mutation-scale", type=float, default=0.1, help="Scale of parameter mutations")
    parser.add_argument("--spatial-prob", type=float, default=0.5, help="Probability of spatial matrix mutation")
    parser.add_argument("--seed", "--seeds-start", dest="seed", type=int, default=42, help="Master PRNG seed for reproducible evolution")
    parser.add_argument("--num-workers", type=int, default=4, help="Parallel worker threads")
    parser.add_argument("--pilot", action="store_true", help="Run a fast 5-generation pilot check")

    args = parser.parse_args()

    generations = 5 if args.pilot else args.generations
    seeds_per_eval = 10 if args.pilot else args.seeds_per_eval

    engine = EvolutionaryEngine(
        lambda_size=args.lambda_size,
        generations=generations,
        seeds_per_eval=seeds_per_eval,
        opponent=args.opponent,
        weights_path=args.weights_path,
        output_path=args.output_path,
        curriculum=args.curriculum,
        curriculum_switch=args.curriculum_switch,
        mutation_scale=args.mutation_scale,
        spatial_mutation_prob=args.spatial_prob,
        seed=args.seed,
        num_workers=args.num_workers,
    )

    engine.run()


if __name__ == "__main__":
    main()
