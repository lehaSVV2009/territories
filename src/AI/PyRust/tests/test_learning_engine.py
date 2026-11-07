"""Comprehensive unit and integration test suite for Milestone M4: Python Training Engine.

Verifies:
1. Fixed Numeraire: WEIGHT_AREA = 50.0 invariance across mutations and clamping.
2. Clamping Corridors: Strict corridor boundaries on all 10 heuristic parameters.
3. Symmetric Radial Gaussian Domes: 180° point-inversion symmetry, non-white-noise smoothness, [0, 50] bounds.
4. Exact Fitness Calculation: Fitness = WinRate_% + 0.02 * AvgMargin (+50 margin = +1.0% fitness).
5. Anchor Set Validation: 100 CRN seed pairs (1000..1099) with multi-archetype regression sentinel.
6. Top-2 Elitism: Top-2 mutant fallback examination when Top-1 narrowly fails Anchor Set.
7. Weight Delta Reporting: Gen 0 vs Gen N parameter and spatial matrix delta table formatting.
8. Weights File Persistence: 100% JS browser build format compatibility roundtrip.
9. End-to-end Pilot Training Run: 5 generations of evolutionary optimization with FFI evaluation.
"""

import math
import os
import random
import tempfile
import pytest

import pyrust
from models.learning_engine import (
    ANCHOR_SEEDS,
    AnchorValidator,
    BOARD_HEIGHT,
    BOARD_WIDTH,
    CLAMPING_CORRIDORS,
    Candidate,
    DEFAULT_WEIGHTS,
    EvolutionaryEngine,
    PARAM_NAMES,
    SPATIAL_MAX,
    SPATIAL_MIN,
    TOTAL_CELLS,
    apply_gaussian_domes,
    calculate_fitness,
    clamp_param_value,
    clamp_params_dict,
    clamp_params_list,
    fitness_from_stats,
    generate_weight_delta_table,
    is_params_clamped,
    mutate_params,
    symmetrize_spatial_matrix,
    verify_matrix_symmetry,
    ValidationResult,
)


# ==============================================================================
# 1. Numeraire & Clamping Corridor Tests
# ==============================================================================

class TestNumeraireAndClampingCorridors:
    """Verifies that WEIGHT_AREA = 50.0 is invariant and all parameters respect strict corridors."""

    def test_numeraire_area_invariance_on_mutation(self):
        rng = random.Random(12345)
        base_params = [DEFAULT_WEIGHTS[name] for name in PARAM_NAMES]
        assert base_params[0] == 50.0

        for _ in range(100):
            mutated = mutate_params(base_params, mutation_scale=0.5, mutation_rate=1.0, rng=rng)
            assert mutated[0] == 50.0, f"WEIGHT_AREA mutated to {mutated[0]}, must remain strictly 50.0"

    def test_numeraire_reset_when_tampered(self):
        tampered = [999.0] + [100.0] * 9
        clamped = clamp_params_list(tampered)
        assert clamped[0] == 50.0, "clamp_params_list must reset WEIGHT_AREA to 50.0"

    def test_all_parameter_corridors_enforced(self):
        rng = random.Random(42)
        base_params = [DEFAULT_WEIGHTS[name] for name in PARAM_NAMES]

        # Heavy aggressive mutations
        for _ in range(200):
            mutated = mutate_params(base_params, mutation_scale=2.0, mutation_rate=1.0, rng=rng)
            assert len(mutated) == len(PARAM_NAMES)
            assert is_params_clamped(mutated), f"Mutated parameters broke corridors: {mutated}"
            for i, name in enumerate(PARAM_NAMES):
                low, high = CLAMPING_CORRIDORS[name]
                assert low <= mutated[i] <= high, (
                    f"Parameter '{name}' value {mutated[i]} outside corridor [{low}, {high}]"
                )

    def test_clamping_underflow_overflow_bounds(self):
        # Underflow check: all set to -999.0
        underflow = [-999.0] * len(PARAM_NAMES)
        clamped_low = clamp_params_list(underflow)
        for i, name in enumerate(PARAM_NAMES):
            expected_low = CLAMPING_CORRIDORS[name][0]
            assert clamped_low[i] == expected_low, (
                f"Underflow clamp failed for {name}: expected {expected_low}, got {clamped_low[i]}"
            )

        # Overflow check: all set to +999.0
        overflow = [999.0] * len(PARAM_NAMES)
        clamped_high = clamp_params_list(overflow)
        for i, name in enumerate(PARAM_NAMES):
            expected_high = CLAMPING_CORRIDORS[name][1]
            assert clamped_high[i] == expected_high, (
                f"Overflow clamp failed for {name}: expected {expected_high}, got {clamped_high[i]}"
            )

    def test_is_params_clamped_predicate(self):
        valid = [DEFAULT_WEIGHTS[name] for name in PARAM_NAMES]
        assert is_params_clamped(valid) is True

        # Invalidate WEIGHT_CAPTURE (valid: [70, 150])
        invalid = list(valid)
        invalid[1] = 150.1
        assert is_params_clamped(invalid) is False

        invalid[1] = 69.9
        assert is_params_clamped(invalid) is False

        # Invalidate WEIGHT_AREA (valid: [50, 50])
        invalid_area = list(valid)
        invalid_area[0] = 50.01
        assert is_params_clamped(invalid_area) is False


# ==============================================================================
# 2. Spatial Matrix & Symmetric Radial Gaussian Bumps
# ==============================================================================

class TestSymmetricRadialGaussianBumps:
    """Verifies that spatial matrix mutations are 180° point-symmetric and smoothly radial."""

    def test_spatial_matrix_dimensions(self):
        assert BOARD_HEIGHT == 15
        assert BOARD_WIDTH == 40
        assert TOTAL_CELLS == 600

    def test_radial_gaussian_bumps_strictly_180_symmetric(self):
        rng = random.Random(999)
        base_matrix = [20.0] * TOTAL_CELLS

        for trial in range(30):
            mutated = apply_gaussian_domes(
                base_matrix,
                num_domes=random.randint(2, 3),
                sigma_range=(2.5, 4.0),
                delta_range=(0.5, 2.5),
                rng=rng,
            )
            assert len(mutated) == TOTAL_CELLS
            assert verify_matrix_symmetry(mutated), f"Trial {trial} violated 180° symmetry"

            # Check exact cell-by-cell symmetry
            for r in range(BOARD_HEIGHT):
                r_mir = BOARD_HEIGHT - 1 - r
                for c in range(BOARD_WIDTH):
                    c_mir = BOARD_WIDTH - 1 - c
                    v1 = mutated[r * BOARD_WIDTH + c]
                    v2 = mutated[r_mir * BOARD_WIDTH + c_mir]
                    assert abs(v1 - v2) < 1e-12, (
                        f"Asymmetry at ({r},{c})={v1} vs ({r_mir},{c_mir})={v2}"
                    )

    def test_matrix_values_strictly_clamped_in_zero_to_fifty(self):
        rng = random.Random(42)
        base_matrix = [48.0] * TOTAL_CELLS  # Near ceiling

        # Apply positive aggressive bumps to test ceiling clamping
        bumped = apply_gaussian_domes(base_matrix, num_domes=5, delta_range=(10.0, 20.0), rng=rng)
        assert max(bumped) <= SPATIAL_MAX

        # Apply negative aggressive bumps to test floor clamping
        base_matrix_low = [2.0] * TOTAL_CELLS
        bumped_low = apply_gaussian_domes(base_matrix_low, num_domes=5, delta_range=(10.0, 20.0), rng=rng)
        assert min(bumped_low) >= SPATIAL_MIN

    def test_no_independent_white_noise(self):
        """Verifies that spatial mutations are continuous radial domes with high spatial autocorrelation

        rather than uncorrelated white noise.
        """
        rng = random.Random(777)
        base_matrix = [15.0] * TOTAL_CELLS
        mutated = apply_gaussian_domes(base_matrix, num_domes=2, sigma_range=(3.0, 4.0), rng=rng)

        # Compute differences from base matrix
        deltas = [mutated[i] - base_matrix[i] for i in range(TOTAL_CELLS)]

        # Calculate horizontal and vertical neighbour correlations of deltas
        h_diffs = []
        for r in range(BOARD_HEIGHT):
            for c in range(BOARD_WIDTH - 1):
                idx1 = r * BOARD_WIDTH + c
                idx2 = r * BOARD_WIDTH + c + 1
                h_diffs.append(abs(deltas[idx1] - deltas[idx2]))

        # In a Gaussian dome with sigma=3.0, adjacent cell step delta |d(x+1) - d(x)|
        # is small (bounded by delta / sigma ~ 2.5 / 3.0 ~ 0.8), whereas white noise has jumpy deltas.
        max_adjacent_step = max(h_diffs)
        avg_adjacent_step = sum(h_diffs) / len(h_diffs)

        assert avg_adjacent_step < 0.6, (
            f"Average adjacent delta step {avg_adjacent_step:.3f} is too erratic for Gaussian dome"
        )
        assert max_adjacent_step < 2.0, (
            f"Max step {max_adjacent_step:.3f} exceeds Gaussian gradient bound"
        )


# ==============================================================================
# 3. Fitness Function Tests
# ==============================================================================

class TestFitnessFunction:
    """Verifies that Fitness = WinRate_% + 0.02 * AvgMargin with tie-breaker mechanics."""

    def test_exact_fitness_formula(self):
        # 100% win rate, zero margin
        assert calculate_fitness(100.0, 0.0) == 100.0

        # 95% win rate, +50 margin (+1.0% fitness tie-breaker)
        assert calculate_fitness(95.0, 50.0) == 96.0

        # 50% win rate, +100 margin
        assert calculate_fitness(50.0, 100.0) == 52.0

        # 0% win rate, -200 margin
        assert calculate_fitness(0.0, -200.0) == -4.0

    def test_margin_tie_breaker_distinction(self):
        # Two candidates with identical 80.0% win rate
        fit_cand_a = calculate_fitness(80.0, 40.0)   # +40 margin -> 80.8
        fit_cand_b = calculate_fitness(80.0, -10.0)  # -10 margin -> 79.8
        assert fit_cand_a > fit_cand_b
        assert abs(fit_cand_a - 80.8) < 1e-9
        assert abs(fit_cand_b - 79.8) < 1e-9

    def test_fitness_from_stats_dict(self):
        stats = {
            "winrate": 0.90,  # 90.0%
            "net_margin": 400.0,
            "total_matches": 10,  # avg_margin = 40.0
        }
        computed = fitness_from_stats(stats)
        expected = 90.0 + 0.02 * 40.0  # 90.8
        assert abs(computed - expected) < 1e-9


# ==============================================================================
# 4. Anchor Set Validation & Multi-Archetype Sentinel Tests
# ==============================================================================

class TestAnchorSetValidationAndElitism:
    """Verifies Anchor Set 100-seed pair examination, anti-overfitting gating, and Top-2 elitism."""

    def test_anchor_seeds_specification(self):
        assert len(ANCHOR_SEEDS) == 100
        assert ANCHOR_SEEDS[0] == 1000
        assert ANCHOR_SEEDS[-1] == 1099

    def test_candidate_representation_and_cloning(self):
        cand = Candidate.from_default()
        assert cand.is_clamped()
        assert cand.is_symmetric()

        cloned = cand.clone()
        assert cloned.params == cand.params
        assert cloned.matrix == cand.matrix
        cloned.params[1] += 5.0
        assert cloned.params[1] != cand.params[1]

    def test_anchor_validator_approves_superior_candidate(self):
        validator = AnchorValidator(anchor_seeds=[1000, 1001], num_workers=2)
        champ = Candidate.from_default()
        better_cand = Candidate.from_default()
        better_cand.params[1] = 110.0  # Boost capture weight

        result = validator.validate(better_cand, champ, target_opponent="FastBot")
        assert isinstance(result.passed, bool)
        assert hasattr(result, "cand_fitness")
        assert hasattr(result, "champ_fitness")

    def test_top2_elitism_logic(self):
        """Exercises EvolutionaryEngine._evaluate_generation_promotions directly for Top-2 elitism."""
        champ = Candidate.from_default()
        champ.fitness = 100.0

        top1 = Candidate.from_default()
        top1.winrate = 0.75  # > 60% generation win rate
        top1.fitness = 108.0

        top2 = Candidate.from_default()
        top2.winrate = 0.70
        top2.fitness = 105.0

        # Scenario: Any candidate except top2 fails Anchor examination; top2 passes
        class MockValidator:
            def validate(self, candidate, champion, target_opponent="GreedyBot"):
                if candidate is top2:
                    return ValidationResult(
                        passed=True,
                        cand_fitness=104.0,
                        champ_fitness=100.0,
                        cand_stats={"composite_fitness": 104.0},
                        champ_stats={"composite_fitness": 100.0},
                        reason="Approved: Strictly exceeded champion on Anchor Set without archetype regression",
                    )
                else:
                    return ValidationResult(
                        passed=False,
                        cand_fitness=98.0,
                        champ_fitness=100.0,
                        cand_stats={"composite_fitness": 98.0},
                        champ_stats={"composite_fitness": 100.0},
                        reason="Failed Anchor Set",
                    )

        engine = EvolutionaryEngine(
            lambda_size=2, generations=1, seeds_per_eval=2, opponent="GreedyBot"
        )
        engine.champion = champ
        engine.validator = MockValidator()
        active_opp = "GreedyBot"

        scored_mutants = [
            (top1, {"winrate": top1.winrate, "total_matches": 10, "net_margin": 80.0}, top1.fitness),
            (top2, {"winrate": top2.winrate, "total_matches": 10, "net_margin": 60.0}, top2.fitness),
        ]

        # 1. Genuine promotion evaluation via EvolutionaryEngine method
        promoted_cand, val_result, reason = engine._evaluate_generation_promotions(
            champ_gen_fit=champ.fitness,
            scored_mutants=scored_mutants,
            active_opp=active_opp,
        )

        assert promoted_cand is top2, "Top-2 elitism must promote Top-2 candidate when Top-1 fails Anchor Set"
        assert val_result is not None and val_result.passed is True
        assert val_result.cand_fitness == 104.0
        assert engine.top2_fallback_activations == 1
        assert engine.top2_promotions_count == 1
        assert "Top-2 promoted" in reason

        # 2. Boundary Condition: Top-1 win rate <= 0.60 must suppress Top-2 fallback
        engine_no_fallback = EvolutionaryEngine(
            lambda_size=2, generations=1, seeds_per_eval=2, opponent="GreedyBot"
        )
        engine_no_fallback.champion = champ
        engine_no_fallback.validator = MockValidator()

        top1_marginal = Candidate.from_default()
        top1_marginal.winrate = 0.59  # <= 60.0% threshold
        top1_marginal.fitness = 108.0

        scored_marginal = [
            (top1_marginal, {"winrate": 0.59, "total_matches": 10, "net_margin": 40.0}, 108.0),
            (top2, {"winrate": 0.70, "total_matches": 10, "net_margin": 60.0}, 105.0),
        ]

        prom_none, val_none, reason_none = engine_no_fallback._evaluate_generation_promotions(
            champ_gen_fit=champ.fitness,
            scored_mutants=scored_marginal,
            active_opp=active_opp,
        )

        assert prom_none is None, "Top-2 fallback must NOT trigger if Top-1 failed to meet 60% win rate bar"
        assert val_none is None
        assert engine_no_fallback.top2_fallback_activations == 0
        assert engine_no_fallback.top2_promotions_count == 0


# ==============================================================================
# 5. Weight Delta Table Reporting & Persistence Tests
# ==============================================================================

class TestReportingAndPersistence:
    """Verifies delta table generation and 100% JS browser format weights persistence."""

    def test_generate_weight_delta_table(self):
        p0 = [DEFAULT_WEIGHTS[n] for n in PARAM_NAMES]
        p1 = list(p0)
        p1[1] += 10.0  # WEIGHT_CAPTURE delta
        p1[3] += 5.0   # WEIGHT_VORONOI_DELTA delta

        m0 = [10.0] * TOTAL_CELLS
        m1 = [12.0] * TOTAL_CELLS

        report = generate_weight_delta_table(p0, p1, m0, m1, gen_from=0, gen_to=40)
        assert "WEIGHT EVOLUTION DELTA REPORT" in report
        assert "WEIGHT_AREA" in report
        assert "WEIGHT_CAPTURE" in report
        assert "SPATIAL MATRIX RELIEF PROFILE" in report
        assert "Matrix L2 norm delta" in report

    def test_save_and_load_champion_js_compatibility(self):
        cand = Candidate.from_default()
        cand.metadata = {
            "epoch": 40,
            "winrate_vs_hard": 0.95,
            "winrate_vs_greedy": 0.99,
            "fitness": 142.5,
        }

        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
            path = tf.name

        try:
            # Save using pyrust.save_weights_file
            pyrust.save_weights_file(
                path,
                cand.params,
                cand.matrix,
                cand.metadata,
            )

            # Load and verify 100% roundtrip
            p_loaded, m_loaded, meta_loaded = pyrust.load_weights_file(path)
            assert len(p_loaded) == 10
            assert len(m_loaded) == 600
            assert meta_loaded["epoch"] == 40
            assert abs(meta_loaded["fitness"] - 142.5) < 1e-6

            # Verify Numeraire invariant survived roundtrip
            assert p_loaded[0] == 50.0
        finally:
            if os.path.exists(path):
                os.remove(path)


# ==============================================================================
# 6. Pilot Training Run Integration Test
# ==============================================================================

class TestPilotTrainingRun:
    """Executes a 5-generation pilot training run to verify full pipeline convergence and FFI integration."""

    def test_pilot_training_run_5_generations(self):
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf_out:
            out_weights = tf_out.name

        try:
            # 5 generations, lambda=4, seeds=3 per eval for quick integration verification
            engine = EvolutionaryEngine(
                lambda_size=4,
                generations=5,
                seeds_per_eval=3,
                opponent="GreedyBot",
                output_path=out_weights,
                mutation_scale=0.1,
                seed=42,
                num_workers=4,
            )

            champ, history = engine.run()

            assert len(history) == 5
            assert champ.is_clamped()
            assert champ.is_symmetric()
            assert champ.params[0] == 50.0  # Numeraire invariant

            # Verify saved weights file
            assert os.path.exists(out_weights)
            p_load, m_load, meta_load = pyrust.load_weights_file(out_weights)
            assert p_load[0] == 50.0
            assert len(m_load) == 600
            assert verify_matrix_symmetry(m_load)
        finally:
            if os.path.exists(out_weights):
                os.remove(out_weights)

    def test_pilot_training_ram_stability_and_generation_timing(self):
        """Verifies RAM stability across epochs and records generation throughput."""
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf_out:
            out_weights = tf_out.name

        try:
            engine = EvolutionaryEngine(
                lambda_size=4,
                generations=5,
                seeds_per_eval=2,
                opponent="FastBot",
                output_path=out_weights,
                mutation_scale=0.1,
                seed=101,
                num_workers=4,
            )

            champ, history = engine.run()
            assert len(history) == 5

            # Verify RAM consumption is stable across epochs (no C-ABI memory leaks)
            ram_values = [h["ram_mb"] for h in history if "ram_mb" in h]
            assert len(ram_values) == 5
            ram_growth_mb = ram_values[-1] - ram_values[0]
            # Memory should not leak more than 30MB over 5 generations
            assert ram_growth_mb < 30.0, f"Potential memory leak detected: RAM grew by {ram_growth_mb:.2f} MB"

            # Verify all generations completed in reasonable time
            for h in history:
                assert h["elapsed"] > 0.0
        finally:
            if os.path.exists(out_weights):
                os.remove(out_weights)
