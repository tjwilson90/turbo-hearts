#!/usr/bin/env python3
"""Summarize paired human/bot branches, clustering uncertainty by original game."""
import argparse
import json
from collections import defaultdict
from pathlib import Path

import numpy as np


def estimate(values):
    """Equal weight per sampled decision, with a game-clustered standard error."""
    groups = defaultdict(list)
    for game, value in values:
        groups[game].append(value)
    n = len(values)
    if n == 0:
        return {"decisions": 0, "games": 0, "mean": None, "ci95": None}
    mean = sum(v for _, v in values) / n
    clusters = len(groups)
    if clusters == 1:
        return {"decisions": n, "games": 1, "mean": mean, "ci95": None}
    residuals = [sum(v - mean for v in group) for group in groups.values()]
    se = np.sqrt(clusters / (clusters - 1) * sum(v * v for v in residuals)) / n
    return {"decisions": n, "games": clusters, "mean": mean,
            "ci95": [mean - 1.96 * se, mean + 1.96 * se]}


def read(path):
    rows = [json.loads(line) for line in path.open()]
    return {(r["game"], r["event"], r["seat"]): r for r in rows}


def gap(row, responder):
    if row["equivalent"]:
        return 0.0
    return next(c["human_minus_bot"] for c in row["continuations"]
                if c["responder"] == responder)


def summarize(baseline, candidate):
    assert baseline, "No sampled decisions"
    assert baseline.keys() == candidate.keys(), "Compare the same sampled positions"
    for key, row in baseline.items():
        other = candidate[key]
        assert row["human"] == other["human"] and row["seeds"] == other["seeds"]
        if not row["equivalent"] and not other["equivalent"]:
            assert len(row["continuations"]) == len(other["continuations"])
        for a, b in zip(row["continuations"], other["continuations"]):
            assert a["responder"] == b["responder"]
            assert a["human_money"] == b["human_money"], "Human branch must be shared"
    responders = sorted({c["responder"] for rows in [baseline, candidate]
                         for r in rows.values() for c in r["continuations"]})
    results = []
    for mode in ["all", "lead", "follow", "slough"]:
        keys = [k for k, r in baseline.items() if mode == "all" or r["mode"] == mode]
        for responder in responders:
            before = [(k[0], gap(baseline[k], responder)) for k in keys]
            after = [(k[0], gap(candidate[k], responder)) for k in keys]
            paired = [(g, a - b) for (g, a), (_, b) in zip(before, after)]
            results.append({"mode": mode, "responder": responder,
                "baseline_disagreements": sum(not baseline[k]["equivalent"] for k in keys),
                "candidate_disagreements": sum(not candidate[k]["equivalent"] for k in keys),
                "human_minus_baseline": estimate(before),
                "human_minus_candidate": estimate(after),
                "candidate_minus_baseline": estimate(paired)})
    return {"baseline": next(iter(baseline.values()))["profile"],
            "candidate": next(iter(candidate.values()))["profile"], "results": results,
            "interpretation": "Actual hands with responsive simulated continuations; equivalent moves have zero gap. "
                              "Intervals cluster by game, not move or continuation seed. "
                              "Recorded final scores are not causal action labels."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    report = summarize(read(args.baseline), read(args.candidate))
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
