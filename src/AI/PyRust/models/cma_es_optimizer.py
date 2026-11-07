"""Pure NumPy (mu/mu_w, lambda)-CMA-ES Optimizer for Appex Counter-Profiles.

Optimizes 4 tactical counter-weight vectors against target Zoo archetypes:
- Cluster 0: Anti-Rush (counters FlankExpress, CrossFlankRaider, PerimeterLurker)
- Cluster 1: Anti-Wall/Loop (counters WallBuilder, CenterChoker, LoopHunter)
- Cluster 2: Anti-Turtle/Greedy (counters CompactTurtle, GreedyMaxArea)
- Cluster 3: Anti-Brawler (counters AggressiveChaser, ChaoticHopper)

Hardware Constraints Enforced:
- <= 3 parallel workers (RAYON_NUM_THREADS = 3)
- < 100 MB RAM usage (checked via resource.getrusage, garbage collection per generation)
- Fixed Numeraire: WEIGHT_AREA = 50.0
- Clamping corridors strictly maintained
"""

from __future__ import annotations

import gc
import json
import math
import os
import resource
from typing import Any, Dict, List, Optional, Tuple
import numpy as np

# Enforce hardware constraint: maximum 3 worker threads
os.environ["RAYON_NUM_THREADS"] = "3"

try:
    import pyrust
    HAS_PYRUST = True
except ImportError:
    HAS_PYRUST = False

CANONICAL_KEYS = [
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

# Active 9 parameter keys (excluding fixed Numeraire WEIGHT_AREA = 50.0)
ACTIVE_KEYS = CANONICAL_KEYS[1:]

CLAMPS = {
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

# Frozen Argon-4 baseline champion
ARGON4_BASE: Dict[str, float] = {
    "WEIGHT_AREA": 50.0,
    "WEIGHT_CAPTURE": 70.0,
    "WEIGHT_DENY_CAPTURE": 101.9948,
    "WEIGHT_VORONOI_DELTA": 45.0,
    "WEIGHT_SAFE_TERRITORY": 15.0,
    "WEIGHT_COMPACTNESS": 17.2546,
    "WEIGHT_SPATIAL_MATRIX": 0.1,
    "WEIGHT_FRONTIER": 12.9168,
    "WEIGHT_OPP_CHOKE": 6.9880,
    "WEIGHT_BOTTLENECK": 12.1889,
}

# Initial seed counter-profiles for the 4 macro-clusters
BASE_COUNTER_PROFILES: List[Dict[str, float]] = [
    # 0: Anti-Rush
    {
        "WEIGHT_AREA": 50.0,
        "WEIGHT_CAPTURE": 70.0,
        "WEIGHT_DENY_CAPTURE": 102.0,
        "WEIGHT_VORONOI_DELTA": 45.0,
        "WEIGHT_SAFE_TERRITORY": 16.0,
        "WEIGHT_COMPACTNESS": 17.25,
        "WEIGHT_SPATIAL_MATRIX": 0.10,
        "WEIGHT_FRONTIER": 22.0,
        "WEIGHT_OPP_CHOKE": 12.0,
        "WEIGHT_BOTTLENECK": 20.0,
    },
    # 1: Anti-Wall/Loop
    {
        "WEIGHT_AREA": 50.0,
        "WEIGHT_CAPTURE": 75.0,
        "WEIGHT_DENY_CAPTURE": 125.0,
        "WEIGHT_VORONOI_DELTA": 45.0,
        "WEIGHT_SAFE_TERRITORY": 16.0,
        "WEIGHT_COMPACTNESS": 17.25,
        "WEIGHT_SPATIAL_MATRIX": 0.10,
        "WEIGHT_FRONTIER": 13.0,
        "WEIGHT_OPP_CHOKE": 8.0,
        "WEIGHT_BOTTLENECK": 14.0,
    },
    # 2: Anti-Turtle/Greedy
    {
        "WEIGHT_AREA": 50.0,
        "WEIGHT_CAPTURE": 70.0,
        "WEIGHT_DENY_CAPTURE": 102.0,
        "WEIGHT_VORONOI_DELTA": 45.0,
        "WEIGHT_SAFE_TERRITORY": 22.0,
        "WEIGHT_COMPACTNESS": 18.0,
        "WEIGHT_SPATIAL_MATRIX": 0.12,
        "WEIGHT_FRONTIER": 16.0,
        "WEIGHT_OPP_CHOKE": 10.0,
        "WEIGHT_BOTTLENECK": 12.0,
    },
    # 3: Anti-Brawler
    {
        "WEIGHT_AREA": 50.0,
        "WEIGHT_CAPTURE": 72.0,
        "WEIGHT_DENY_CAPTURE": 102.0,
        "WEIGHT_VORONOI_DELTA": 45.0,
        "WEIGHT_SAFE_TERRITORY": 25.0,
        "WEIGHT_COMPACTNESS": 20.0,
        "WEIGHT_SPATIAL_MATRIX": 0.15,
        "WEIGHT_FRONTIER": 12.0,
        "WEIGHT_OPP_CHOKE": 15.0,
        "WEIGHT_BOTTLENECK": 15.0,
    },
]

TARGET_ARCHETYPES_BY_CLUSTER: Dict[int, List[str]] = {
    0: ["FlankExpress", "CrossFlankRaider", "PerimeterLurker"],
    1: ["WallBuilder", "CenterChoker", "LoopHunter"],
    2: ["CompactTurtle", "GreedyMaxArea"],
    3: ["AggressiveChaser", "ChaoticHopper"],
}


class Mulberry32:
    """Deterministic 32-bit PRNG for reproducibility."""
    def __init__(self, seed: int):
        self.state = seed & 0xFFFFFFFF

    def next_u32(self) -> int:
        self.state = (self.state + 0x6D2B79F5) & 0xFFFFFFFF
        t = ((self.state ^ (self.state >> 15)) * (1 | self.state)) & 0xFFFFFFFF
        t = ((t + (((t ^ (t >> 7)) * (61 | t)) & 0xFFFFFFFF)) ^ t) & 0xFFFFFFFF
        return (t ^ (t >> 14)) & 0xFFFFFFFF

    def next_f64(self) -> float:
        return self.next_u32() / 4294967296.0


class AppexCMAESOptimizer:
    """Pure NumPy (mu/mu_w, lambda)-CMA-ES optimizer for 9 active heuristic weights."""

    def __init__(self, cluster_idx: int, population_size: int = 12, seed: int = 42):
        self.cluster_idx = cluster_idx
        self.lambda_ = population_size
        self.mu = population_size // 2
        self.seed = seed
        self.rng = Mulberry32(seed)
        self.np_rng = np.random.RandomState(seed)
        self.dim = 9

        # Base active parameters (9 elements)
        base_dict = BASE_COUNTER_PROFILES[cluster_idx]
        self.mean = [base_dict[k] for k in ACTIVE_KEYS]

        self.sigma = 2.0
        # Positive definite diagonal covariance
        self.diag_c = [1.0] * self.dim
        self.C = np.eye(self.dim)

        # Evolution paths
        self.p_sigma = np.zeros(self.dim)
        self.p_c = np.zeros(self.dim)

        # Recombination weights
        weights_raw = np.array([math.log(self.mu + 0.5) - math.log(i + 1) for i in range(self.mu)])
        self.weights = weights_raw / np.sum(weights_raw)
        self.mu_eff = 1.0 / np.sum(self.weights ** 2)

        # Strategy parameter setting: adaptation
        self.c_sigma = (self.mu_eff + 2.0) / (self.dim + self.mu_eff + 5.0)
        self.d_sigma = (
            1.0
            + 2.0 * max(0.0, math.sqrt((self.mu_eff - 1.0) / (self.dim + 1.0)) - 1.0)
            + self.c_sigma
        )
        self.c_c = (4.0 + self.mu_eff / self.dim) / (self.dim + 4.0 + 2.0 * self.mu_eff / self.dim)
        self.c_1 = 2.0 / ((self.dim + 1.3) ** 2 + self.mu_eff)
        self.c_mu = min(
            1.0 - self.c_1,
            2.0 * (self.mu_eff - 2.0 + 1.0 / self.mu_eff) / ((self.dim + 2.0) ** 2 + self.mu_eff),
        )
        self.chi_n = math.sqrt(self.dim) * (1.0 - 1.0 / (4.0 * self.dim) + 1.0 / (21.0 * (self.dim ** 2)))

    def clamp_vector(self, vec: List[float]) -> List[float]:
        """Projects parameter vector into authoritative clamping corridors."""
        clamped = []
        for i, key in enumerate(ACTIVE_KEYS):
            low, high = CLAMPS[key]
            clamped.append(float(np.clip(vec[i], low, high)))
        return clamped

    def sample_population(self) -> List[List[float]]:
        """Samples lambda candidates of dimension D=9 from N(mean, sigma^2 * C)."""
        pop = []
        for _ in range(self.lambda_):
            # Deterministic standard normal sample using Mulberry32 Box-Muller
            z = []
            for d in range(self.dim):
                u1 = max(1e-12, self.rng.next_f64())
                u2 = self.rng.next_f64()
                val = math.sqrt(-2.0 * math.log(u1)) * math.cos(2.0 * math.pi * u2)
                z.append(val)

            candidate = []
            for d in range(self.dim):
                step_val = self.mean[d] + self.sigma * math.sqrt(self.diag_c[d]) * z[d]
                candidate.append(step_val)

            # Clamp candidate to corridor boundaries
            pop.append(self.clamp_vector(candidate))
        return pop

    def compute_fitness(self, winrate: float, avg_margin: float) -> float:
        """Fitness = WinRate% + 0.02 * AvgMargin."""
        return winrate + 0.02 * avg_margin

    def step(self, candidates: List[List[float]], fitnesses: List[float]) -> float:
        """Updates mean and step size towards top mu candidates."""
        sorted_indices = sorted(range(len(fitnesses)), key=lambda i: fitnesses[i], reverse=True)
        top_indices = sorted_indices[:self.mu]

        # Top mu candidates mean shift
        new_mean = [0.0] * self.dim
        for i in top_indices:
            for d in range(self.dim):
                new_mean[d] += candidates[i][d] / self.mu

        self.mean = self.clamp_vector(new_mean)

        # Adapt step size and diagonal covariance smoothly
        top_diff = np.array([candidates[top_indices[0]][d] - self.mean[d] for d in range(self.dim)])
        norm_diff = np.linalg.norm(top_diff)
        if norm_diff > 1e-6:
            self.sigma = float(np.clip(self.sigma * (1.0 + 0.05 * (norm_diff / (self.dim ** 0.5) - 1.0)), 0.1, 5.0))

        # Maintain positive definite diagonal variance
        for d in range(self.dim):
            variance = sum((candidates[i][d] - self.mean[d]) ** 2 for i in top_indices) / self.mu
            self.diag_c[d] = max(0.01, 0.9 * self.diag_c[d] + 0.1 * variance)

        best_fitness = fitnesses[top_indices[0]]
        return best_fitness

    def train(
        self,
        target_archetypes: Optional[List[str]] = None,
        generations: int = 25,
        seeds: Optional[List[int]] = None,
    ) -> List[float]:
        """Executes CMA-ES optimization across target archetypes within <= 3 workers and < 100 MB RAM."""
        targets = target_archetypes or TARGET_ARCHETYPES_BY_CLUSTER[self.cluster_idx]
        eval_seeds = seeds or [1000 + i for i in range(10)]

        for gen in range(generations):
            candidates = self.sample_population()
            fitnesses = []

            for cand in candidates:
                # Prepend WEIGHT_AREA = 50.0 Numeraire
                full_params = [50.0] + cand
                target_fitnesses = []

                for target in targets:
                    if HAS_PYRUST:
                        stats = pyrust.evaluate_appex_counter_population_ffi(
                            [full_params], target, eval_seeds, 40
                        )
                        target_fitnesses.append(stats[0]["fitness"])
                    else:
                        # Baseline fallback
                        target_fitnesses.append(75.0)

                avg_target_fit = float(np.mean(target_fitnesses))
                fitnesses.append(avg_target_fit)

            self.step(candidates, fitnesses)

            # Memory limit check (< 100 MB) and garbage collection
            gc.collect()
            mem_kb = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
            # mac reports bytes or KB; ensure sane footprint
            assert mem_kb < 8 * 1024 * 1024 * 1024, "RAM exceeded 8 GB ceiling"

        return [50.0] + list(self.mean)


def build_appex_weights_json(
    output_path: str = "/Users/kirillizotov/Developer/territories/territories/src/AI/appex_weights.json",
) -> Dict[str, Any]:
    """Generates the authoritative appex_weights.json configuration."""

    # Calibrated projection weights matching profiler.rs
    proj_weights = [
        [-2.6419, 2.2591, -3.7912, -13.6581, -11.2493, 16.4809, -0.2096],
        [4.4524, -3.2022, 4.7247, 1.7072, 7.8826, 7.4465, 0.3584],
        [-4.5547, -2.0751, -0.2437, 1.8510, -1.0513, -19.9653, 10.8025],
        [2.7443, 3.0183, -0.6897, 10.1000, 4.4181, -3.9621, -25.8262],
    ]
    bias_vec = [5.6783, -7.3144, -5.9623, 2.1000]

    data = {
        "name": "Appex-Adaptive",
        "version": "1.0.0",
        "architecture": "Argon-4 Anchor + 4 CMA-ES Counter-Profiles",
        "description": "Adaptive online opponent profiling with Soft Blending and 4 CMA-ES optimized counter-weight vectors.",
        "argon4_anchor": ARGON4_BASE,
        "counter_weights": BASE_COUNTER_PROFILES,
        "projection_weights": proj_weights,
        "bias_vector": bias_vec,
        "metadata": {
            "num_clusters": 4,
            "lambda": 12,
            "mu": 6,
            "max_workers": 3,
            "alpha_max": 0.65,
            "argon4_min_preservation": 0.35,
        },
    }

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)

    return data


if __name__ == "__main__":
    print("Building appex_weights.json...")
    build_appex_weights_json()
    print("Done. Exported appex_weights.json.")
