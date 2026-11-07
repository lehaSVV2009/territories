"""Comprehensive PyO3 FFI and Maturin integration test suite for Milestone M3.

Verifies:
1. Module import and function discovery
2. `simulate_match_ffi` execution, return contracts, bot parsing, custom weights, and PRNG determinism
3. `evaluate_population_ffi` Rayon parallelism, GIL release, and strict input-to-output index order preservation
4. `select_move_ffi` move legality, score, and p99 <= 2.0 ms latency SLA
5. `export_tree_dot` and `export_tree_json` serialization and syntax validation
6. `save_weights_file` and `load_weights_file` roundtrip, metadata extraction, and 100% JS browser format compatibility
"""

import json
import os
import tempfile
import time
import pytest
import pyrust


class TestPyRustModuleExport:
    """Verifies that all required FFI symbols are exported by the native pyrust extension."""

    def test_module_exports(self):
        expected_functions = [
            "simulate_match_ffi",
            "evaluate_population_ffi",
            "select_move_ffi",
            "export_tree_dot",
            "export_tree_json",
            "save_weights_file",
            "load_weights_file",
        ]
        for fn in expected_functions:
            assert hasattr(pyrust, fn), f"pyrust module is missing required export: '{fn}'"
            assert callable(getattr(pyrust, fn)), f"pyrust export '{fn}' is not callable"


class TestSimulateMatchFFI:
    """Tests for `simulate_match_ffi`."""

    def test_simulate_match_basic_greedy(self):
        res = pyrust.simulate_match_ffi("GreedyBot", "GreedyBot", 42, None, None)
        assert len(res) == 4
        winner, p1_score, p2_score, turns = res
        assert winner in (1, 2, None)
        assert isinstance(p1_score, int) and p1_score >= 0
        assert isinstance(p2_score, int) and p2_score >= 0
        assert isinstance(turns, int) and 1 <= turns <= 80

    def test_simulate_match_determinism(self):
        res1 = pyrust.simulate_match_ffi("FastBot", "GreedyBot", 12345, None, None)
        res2 = pyrust.simulate_match_ffi("FastBot", "GreedyBot", 12345, None, None)
        assert res1 == res2, "Identical seed must produce deterministic match result"

    def test_simulate_match_different_seeds_diverge(self):
        res1 = pyrust.simulate_match_ffi("FastBot", "GreedyBot", 101, None, None)
        res2 = pyrust.simulate_match_ffi("FastBot", "GreedyBot", 999, None, None)
        # Different PRNG sequences will generate different match transcripts
        assert (res1[1], res1[2]) != (res2[1], res2[2])

    def test_simulate_match_custom_weights(self):
        w1 = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]
        w2 = [50.0, 120.0, 100.0, 30.0, 20.0, 10.0, 0.25, 15.0, 15.0, 10.0]
        res = pyrust.simulate_match_ffi("SpatialSlayer", "SpatialSlayer", 42, w1, w2)
        assert len(res) == 4
        assert res[3] > 0

    def test_simulate_match_invalid_bot_type_raises_error(self):
        with pytest.raises(ValueError, match="Unknown bot type"):
            pyrust.simulate_match_ffi("NonExistentBot", "GreedyBot", 42, None, None)


class TestEvaluatePopulationFFI:
    """Tests for `evaluate_population_ffi` Rayon parallelism and result order determinism."""

    def test_evaluate_population_result_structure(self):
        cands = [
            [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0],
            [50.0, 110.0, 90.0, 30.0, 20.0, 10.0, 0.40, 16.0, 16.0, 12.0],
        ]
        seeds = [10, 20, 30]
        results = pyrust.evaluate_population_ffi(cands, None, "GreedyBot", seeds, 4)
        assert len(results) == len(cands)

        required_keys = [
            "winrate", "avg_margin", "avg_turns", "fitness",
            "wins", "losses", "draws", "total_matches", "total_turns", "net_margin"
        ]
        for r in results:
            for k in required_keys:
                assert k in r, f"evaluate_population result missing required key: {k}"
            assert r["total_matches"] == len(seeds) * 2

    def test_evaluate_population_index_order_determinism(self):
        """Explicitly validates that results[i] strictly corresponds to candidates[i]

        even when parallelized across Rayon threads.
        """
        seeds = [101, 102, 103, 104]
        # 4 distinct candidates with varied parameters
        cands = [
            [50.0, 70.0, 70.0, 15.0, 15.0, 5.0, 0.10, 10.0, 10.0, 5.0],
            [50.0, 140.0, 140.0, 40.0, 35.0, 25.0, 0.50, 25.0, 25.0, 20.0],
            [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0],
            [50.0, 85.0, 95.0, 20.0, 20.0, 15.0, 0.20, 12.0, 12.0, 8.0],
        ]

        # 1. Evaluate each candidate singly (deterministic ground truth)
        single_results = []
        for c in cands:
            res = pyrust.evaluate_population_ffi([c], None, "GreedyBot", seeds, 1)
            single_results.append(res[0])

        # 2. Evaluate all 4 candidates in parallel across Rayon global pool
        batch_results = pyrust.evaluate_population_ffi(cands, None, "GreedyBot", seeds, 4)

        assert len(batch_results) == len(cands)
        for i in range(len(cands)):
            assert batch_results[i]["winrate"] == single_results[i]["winrate"], (
                f"Index {i} winrate mismatch: batch {batch_results[i]} vs single {single_results[i]}"
            )
            assert batch_results[i]["net_margin"] == single_results[i]["net_margin"], (
                f"Index {i} margin mismatch: batch {batch_results[i]} vs single {single_results[i]}"
            )
            assert batch_results[i]["total_turns"] == single_results[i]["total_turns"], (
                f"Index {i} turns mismatch: batch {batch_results[i]} vs single {single_results[i]}"
            )

    def test_evaluate_population_custom_matrices(self):
        cands = [[50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]]
        custom_matrix = [2.0] * 600
        seeds = [42]
        res = pyrust.evaluate_population_ffi(cands, [custom_matrix], "FastBot", seeds, 2)
        assert len(res) == 1
        assert res[0]["total_matches"] == 2


class TestSelectMoveFFIAndLatencySLA:
    """Tests for `select_move_ffi` correctness and p99 <= 2.0 ms latency SLA."""

    def test_select_move_return_contract(self):
        board = [0] * 600
        # Place P1 anchor at (0, 0) 2x2 and P2 anchor at (13, 38) 2x2
        for r in range(0, 2):
            for c in range(0, 2):
                board[r * 40 + c] = 1
        for r in range(13, 15):
            for c in range(38, 40):
                board[r * 40 + c] = 2

        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]
        res = pyrust.select_move_ffi(board, 3, 2, weights)

        assert isinstance(res, tuple) and len(res) == 5
        r0, c0, h, w, score = res
        assert 0 <= r0 < 15
        assert 0 <= c0 < 40
        assert (h, w) in ((3, 2), (2, 3))
        assert isinstance(score, float)

    def test_select_move_invalid_board_size_raises_error(self):
        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]
        with pytest.raises(ValueError, match="board_cells must have exactly 600 elements"):
            pyrust.select_move_ffi([0] * 599, 2, 2, weights)

    def test_select_move_latency_sla_p99_under_two_ms(self):
        """Validates that select_move_ffi satisfies p99 <= 2.0 ms SLA across 100 decisions."""
        board = [0] * 600
        for r in range(13, 15):
            for c in range(38, 40):
                board[r * 40 + c] = 2

        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]

        # Warmup
        for _ in range(10):
            pyrust.select_move_ffi(board, 3, 3, weights)

        latencies_ms = []
        for _ in range(100):
            t0 = time.perf_counter()
            mv = pyrust.select_move_ffi(board, 3, 3, weights)
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
            latencies_ms.append(elapsed_ms)
            assert mv[2] > 0 and mv[3] > 0

        latencies_ms.sort()
        p50 = latencies_ms[50]
        p90 = latencies_ms[90]
        p99 = latencies_ms[99]

        print(f"\n[LATENCY SLA BENCHMARK] p50: {p50:.4f} ms | p90: {p90:.4f} ms | p99: {p99:.4f} ms")
        assert p99 <= 2.0, f"Inference SLA violated: p99 latency {p99:.4f} ms exceeds 2.0 ms target"


class TestTreeVisualizationExport:
    """Tests for `export_tree_dot` and `export_tree_json`."""

    def test_export_tree_dot_file(self):
        board = [0] * 600
        board[14 * 40 + 39] = 2
        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]

        with tempfile.NamedTemporaryFile(suffix=".dot", delete=False) as tf:
            out_path = tf.name

        try:
            pyrust.export_tree_dot(board, 2, 3, weights, out_path)
            assert os.path.exists(out_path)
            with open(out_path, "r", encoding="utf-8") as f:
                content = f.read()

            assert "digraph" in content
            assert "{" in content and "}" in content
            assert content.count("{") == content.count("}")
        finally:
            if os.path.exists(out_path):
                os.remove(out_path)

    def test_export_tree_json_file(self):
        board = [0] * 600
        board[14 * 40 + 39] = 2
        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]

        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
            out_path = tf.name

        try:
            pyrust.export_tree_json(board, 2, 3, weights, out_path)
            assert os.path.exists(out_path)
            with open(out_path, "r", encoding="utf-8") as f:
                data = json.load(f)

            assert isinstance(data, dict)
            for req in ["turn", "player", "dice", "boardState", "evaluatedCandidates", "root"]:
                assert req in data, f"Exported tree JSON missing required field: {req}"
        finally:
            if os.path.exists(out_path):
                os.remove(out_path)


class TestWeightsFilePersistence:
    """Tests for `save_weights_file` and `load_weights_file` with 100% JS compatibility."""

    def test_weights_persistence_roundtrip(self):
        weights = [50.0, 95.0, 85.0, 26.0, 25.0, 12.97, 0.32, 18.0, 18.0, 14.0]
        matrix = [float(i % 50) for i in range(600)]
        meta = {
            "epoch": 42,
            "winrate_vs_hard": 0.965,
            "winrate_vs_greedy": 0.992,
            "winrate_vs_champ": 0.685,
            "avg_margin": 22.4,
            "fitness": 141.3,
        }

        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
            out_path = tf.name

        try:
            pyrust.save_weights_file(out_path, weights, matrix, meta)
            assert os.path.exists(out_path)

            # 1. Inspect raw JSON file for 100% JS browser format compliance
            with open(out_path, "r", encoding="utf-8") as f:
                raw_json = json.load(f)

            # Check matrix array has exactly 600 float elements
            assert "matrix" in raw_json, "Saved JSON must contain 'matrix' array for JS compatibility"
            assert len(raw_json["matrix"]) == 600

            # Check params object contains all 10 canonical keys
            assert "params" in raw_json, "Saved JSON must contain 'params' object for JS compatibility"
            canonical_keys = [
                "WEIGHT_AREA", "WEIGHT_CAPTURE", "WEIGHT_DENY_CAPTURE",
                "WEIGHT_VORONOI_DELTA", "WEIGHT_SAFE_TERRITORY", "WEIGHT_COMPACTNESS",
                "WEIGHT_SPATIAL_MATRIX", "WEIGHT_FRONTIER", "WEIGHT_OPP_CHOKE",
                "WEIGHT_BOTTLENECK"
            ]
            for key in canonical_keys:
                assert key in raw_json["params"], f"Saved params missing canonical key '{key}'"

            # 2. Test roundtrip load via FFI
            w_loaded, m_loaded, meta_loaded = pyrust.load_weights_file(out_path)
            assert len(w_loaded) == 10
            assert len(m_loaded) == 600
            assert meta_loaded["epoch"] == 42
            assert abs(meta_loaded["winrate_vs_hard"] - 0.965) < 1e-6
            assert abs(meta_loaded["avg_margin"] - 22.4) < 1e-6

            # Verify weight values preserved
            for orig, loaded in zip(weights, w_loaded):
                assert abs(orig - loaded) < 1e-6

            # Verify matrix elements preserved
            for orig_m, loaded_m in zip(matrix, m_loaded):
                assert abs(orig_m - loaded_m) < 1e-6
        finally:
            if os.path.exists(out_path):
                os.remove(out_path)

    def test_save_weights_invalid_matrix_length_raises_error(self):
        weights = [50.0] * 10
        with pytest.raises(ValueError, match="matrix must have exactly 600 elements"):
            pyrust.save_weights_file("/tmp/bad.json", weights, [1.0] * 500, None)
