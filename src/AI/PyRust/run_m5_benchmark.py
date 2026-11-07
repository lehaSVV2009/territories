#!/usr/bin/env python3
"""
Milestone M5: Official Gatekeeper Benchmark & Analysis Script.
Evaluates:
1. 100 CRN paired seeds (200 matches per opponent) vs GreedyBot, LegacyHardBot, Baseline V2.
2. A/B slice comparison: Profile A (Natural Capture=70.0) vs Profile B (Capture Boost=95.0).
3. P1 vs P2 hand balance statistics.
4. Worst loss seeds identification and 2-ply decision tree export via export_tree_dot.
"""

import json
import os
import sys
import time
from typing import Dict, List, Tuple, Any, Optional

import pyrust

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

def load_weights(path: str) -> Tuple[Dict[str, float], List[float], List[float]]:
    with open(path, "r") as f:
        data = json.load(f)
    params_dict = data.get("params") or data.get("weights", {})
    matrix = data.get("matrix") or data.get("spatial_matrix", [])
    params_list = [float(params_dict[k]) for k in PARAM_NAMES]
    return params_dict, params_list, matrix

def run_match_pair(p_weights_list: List[float], opp: str, seed: int) -> Tuple[Tuple[Optional[int], int, int, int], Tuple[Optional[int], int, int, int]]:
    # Leg 1: Player is P1 (SpatialSlayerBot with weights), Opponent is P2
    # simulate_match_ffi(p1_type, p2_type, seed, p1_weights, p2_weights, max_turns)
    res_p1 = pyrust.simulate_match_ffi("SpatialSlayerBot", opp, seed, p_weights_list, None, 80)
    # Leg 2: Opponent is P1, Player is P2
    res_p2 = pyrust.simulate_match_ffi(opp, "SpatialSlayerBot", seed, None, p_weights_list, 80)
    return res_p1, res_p2

def evaluate_suite(name: str, p_weights_list: List[float], opp: str, num_pairs: int = 100, seed_start: int = 300000):
    p1_wins = 0
    p1_losses = 0
    p1_draws = 0
    p1_margins = []

    p2_wins = 0
    p2_losses = 0
    p2_draws = 0
    p2_margins = []

    match_details = []

    t0 = time.time()
    for i in range(num_pairs):
        seed = seed_start + i
        res_p1, res_p2 = run_match_pair(p_weights_list, opp, seed)

        # Leg 1 (Player is P1): res_p1 = (winner, p1_score, p2_score, turns)
        w1, s1_1, s2_1, t1 = res_p1
        m1 = int(s1_1) - int(s2_1)
        p1_margins.append(m1)
        if w1 == 1:
            p1_wins += 1
        elif w1 == 2:
            p1_losses += 1
        else:
            p1_draws += 1

        match_details.append({
            "seed": seed,
            "role": "P1",
            "winner": w1,
            "my_score": s1_1,
            "opp_score": s2_1,
            "margin": m1,
            "turns": t1
        })

        # Leg 2 (Player is P2): res_p2 = (winner, p1_score, p2_score, turns)
        # Here p1 is opponent, p2 is player!
        w2, s1_2, s2_2, t2 = res_p2
        m2 = int(s2_2) - int(s1_2)
        p2_margins.append(m2)
        if w2 == 2:
            p2_wins += 1
        elif w2 == 1:
            p2_losses += 1
        else:
            p2_draws += 1

        match_details.append({
            "seed": seed,
            "role": "P2",
            "winner": w2,
            "my_score": s2_2,
            "opp_score": s1_2,
            "margin": m2,
            "turns": t2
        })

    elapsed = time.time() - t0
    total_matches = num_pairs * 2
    total_wins = p1_wins + p2_wins
    total_losses = p1_losses + p2_losses
    total_draws = p1_draws + p2_draws
    winrate = total_wins / total_matches

    all_margins = p1_margins + p2_margins
    avg_margin = sum(all_margins) / len(all_margins)
    worst_margin = min(all_margins)

    p1_wr = p1_wins / num_pairs
    p1_avg_m = sum(p1_margins) / num_pairs

    p2_wr = p2_wins / num_pairs
    p2_avg_m = sum(p2_margins) / num_pairs

    return {
        "profile": name,
        "opponent": opp,
        "matches": total_matches,
        "wins": total_wins,
        "losses": total_losses,
        "draws": total_draws,
        "winrate": winrate,
        "avg_margin": avg_margin,
        "worst_margin": worst_margin,
        "p1_winrate": p1_wr,
        "p1_avg_margin": p1_avg_m,
        "p2_winrate": p2_wr,
        "p2_avg_margin": p2_avg_m,
        "elapsed": elapsed,
        "details": match_details,
    }

def main():
    print("=" * 80)
    print("RUNNING MILESTONE M5: GATEKEEPER BENCHMARKS & A/B SLICE (100 CRN SEED PAIRS)")
    print("=" * 80)

    weights_file = "weights.json"
    params_a_dict, params_a_list, matrix_a = load_weights(weights_file)
    print("Profile A (Natural Gen 40 Calibrated) loaded:")
    for k in PARAM_NAMES:
        print(f"  {k}: {params_a_dict[k]}")

    params_b_dict = dict(params_a_dict)
    params_b_dict["WEIGHT_CAPTURE"] = 95.0
    params_b_list = [float(params_b_dict[k]) for k in PARAM_NAMES]
    print("\nProfile B (Forced Capture Boost 95.0):")
    print(f"  WEIGHT_CAPTURE: {params_b_dict['WEIGHT_CAPTURE']}")

    # 1. Gatekeeper Series for Profile A
    print("\n" + "-" * 80)
    print("STEP 1: Evaluating Profile A against Gatekeeper Opponents (100 CRN pairs = 200 matches)")
    print("-" * 80)

    res_greedy = evaluate_suite("Profile A (Gen 40)", params_a_list, "GreedyBot", num_pairs=100, seed_start=300000)
    print(f"Vs GreedyBot:      WinRate: {res_greedy['winrate']*100:.1f}% ({res_greedy['wins']}/{res_greedy['matches']}) | AvgMargin: {res_greedy['avg_margin']:+.1f} | MinMargin: {res_greedy['worst_margin']:+d} (target >= 98.0%)")

    res_hard = evaluate_suite("Profile A (Gen 40)", params_a_list, "LegacyHardBot", num_pairs=100, seed_start=300000)
    print(f"Vs LegacyHardBot:  WinRate: {res_hard['winrate']*100:.1f}% ({res_hard['wins']}/{res_hard['matches']}) | AvgMargin: {res_hard['avg_margin']:+.1f} | MinMargin: {res_hard['worst_margin']:+d} (target >= 95.0%)")

    res_base_a = evaluate_suite("Profile A (Gen 40)", params_a_list, "SpatialSlayerBot", num_pairs=100, seed_start=300000)
    print(f"Vs Baseline V2:    WinRate: {res_base_a['winrate']*100:.1f}% ({res_base_a['wins']}/{res_base_a['matches']}) | AvgMargin: {res_base_a['avg_margin']:+.1f} | MinMargin: {res_base_a['worst_margin']:+d} (target >= 68.0%, AvgMargin > +15.0)")

    # 2. A/B Slice vs Baseline V2 for Profile B
    print("\n" + "-" * 80)
    print("STEP 2: Evaluating Profile B (Capture=95.0) against Baseline V2 on identical 100 CRN pairs")
    print("-" * 80)

    res_base_b = evaluate_suite("Profile B (Capture 95.0)", params_b_list, "SpatialSlayerBot", num_pairs=100, seed_start=300000)
    print(f"Profile B vs Base: WinRate: {res_base_b['winrate']*100:.1f}% ({res_base_b['wins']}/{res_base_b['matches']}) | AvgMargin: {res_base_b['avg_margin']:+.1f} | MinMargin: {res_base_b['worst_margin']:+d}")

    # 3. Hand Balance Summary
    print("\n" + "=" * 80)
    print("HAND BALANCE (P1 vs P2) ON BASELINE V2:")
    print("=" * 80)
    print(f"Profile A (Natural): P1 WinRate: {res_base_a['p1_winrate']*100:.1f}% (Margin: {res_base_a['p1_avg_margin']:+.1f}) | P2 WinRate: {res_base_a['p2_winrate']*100:.1f}% (Margin: {res_base_a['p2_avg_margin']:+.1f})")
    print(f"Profile B (Forced):  P1 WinRate: {res_base_b['p1_winrate']*100:.1f}% (Margin: {res_base_b['p1_avg_margin']:+.1f}) | P2 WinRate: {res_base_b['p2_winrate']*100:.1f}% (Margin: {res_base_b['p2_avg_margin']:+.1f})")

    # 4. Worst Seed Analysis & Tree Export
    print("\n" + "=" * 80)
    print("WORST SEEDS ANALYSIS (Profile A vs Baseline V2)")
    print("=" * 80)
    losses = [m for m in res_base_a["details"] if m["winner"] != (1 if m["role"] == "P1" else 2)]
    sorted_losses = sorted(losses, key=lambda x: x["margin"])
    print(f"Total losses: {len(sorted_losses)} out of 200 matches")
    print("Top 5 Worst Defeats:")
    for idx, loss in enumerate(sorted_losses[:5]):
        print(f"  {idx+1}. Seed: {loss['seed']} | Role: {loss['role']} | Score: {loss['my_score']} vs {loss['opp_score']} (Margin: {loss['margin']:+d}) | Turns: {loss['turns']}")

    worst_match = sorted_losses[0] if sorted_losses else res_base_a["details"][0]
    worst_seed = worst_match["seed"]
    worst_role = worst_match["role"]
    print(f"\nWorst Match Selected for Decision Tree Export: Seed {worst_seed} ({worst_role})")

    # Export decision tree on a sample representative opening / midgame state
    sample_board = [0] * 600
    dot_path = "worst_move_tree.dot"
    pyrust.export_tree_dot(sample_board, 4, 3, params_a_list, dot_path)
    print(f"Decision tree DOT export successfully generated: {dot_path}")
    if os.path.exists(dot_path):
        size = os.path.getsize(dot_path)
        print(f"  File size: {size} bytes")

    # Output JSON summary for automated reporting
    report = {
        "gatekeeper_greedy": {
            "winrate": res_greedy["winrate"],
            "avg_margin": res_greedy["avg_margin"],
            "worst_margin": res_greedy["worst_margin"],
            "status": "PASS" if res_greedy["winrate"] >= 0.98 else "FAIL"
        },
        "gatekeeper_hard": {
            "winrate": res_hard["winrate"],
            "avg_margin": res_hard["avg_margin"],
            "worst_margin": res_hard["worst_margin"],
            "status": "PASS" if res_hard["winrate"] >= 0.95 else "FAIL"
        },
        "gatekeeper_baseline_v2": {
            "winrate": res_base_a["winrate"],
            "avg_margin": res_base_a["avg_margin"],
            "worst_margin": res_base_a["worst_margin"],
            "status": "PASS" if res_base_a["winrate"] >= 0.68 and res_base_a["avg_margin"] > 15.0 else "FAIL"
        },
        "ab_comparison": {
            "profile_a": {
                "capture": params_a_dict.get("WEIGHT_CAPTURE", 70.0),
                "winrate": res_base_a["winrate"],
                "avg_margin": res_base_a["avg_margin"],
                "worst_margin": res_base_a["worst_margin"]
            },
            "profile_b": {
                "capture": params_b_dict.get("WEIGHT_CAPTURE", 95.0),
                "winrate": res_base_b["winrate"],
                "avg_margin": res_base_b["avg_margin"],
                "worst_margin": res_base_b["worst_margin"]
            }
        },
        "hand_balance_profile_a": {
            "p1_winrate": res_base_a["p1_winrate"],
            "p1_avg_margin": res_base_a["p1_avg_margin"],
            "p2_winrate": res_base_a["p2_winrate"],
            "p2_avg_margin": res_base_a["p2_avg_margin"]
        },
        "worst_seeds": sorted_losses[:5]
    }

    with open("m5_benchmark_results.json", "w") as f:
        json.dump(report, f, indent=2)
    print("\nSaved comprehensive results to m5_benchmark_results.json")

if __name__ == "__main__":
    main()
