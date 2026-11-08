#!/usr/bin/env python3
"""Freeze a successful-player cohort from training-only opponent-adjusted payoffs."""
import argparse
import json
from pathlib import Path
import numpy as np


def rank(rows, minimum=30, ridge=30):
    n = 1+max(max(r["players"]) for r in rows)
    x, y = [], []
    counts = np.zeros(n, dtype=int)
    for row in rows:
        if row["split"] != 0:
            continue
        money = np.asarray(row["hand_money"]).mean(axis=0)
        for seat, player in enumerate(row["players"]):
            features = np.zeros(n)
            for other, opponent in enumerate(row["players"]):
                features[opponent] += 1 if seat == other else -1/3
            x.append(features)
            y.append(money[seat])
            counts[player] += int(row["humans"][seat])
    x, y = np.asarray(x), np.asarray(y)
    ratings = np.linalg.solve(x.T@x+ridge*np.eye(n), x.T@y)
    eligible = sorted(np.flatnonzero(counts >= minimum), key=lambda p: -ratings[p])
    selected = set(eligible[:max(1, (len(eligible)+2)//3)])
    return selected, [{"key": int(p), "training_games": int(counts[p]),
                       "rating": float(ratings[p])} for p in eligible]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("report", type=Path)
    args = parser.parse_args()
    rows = [json.loads(line) for line in args.input.open()]
    selected, ratings = rank(rows)
    summary = []
    for split in range(4):
        groups = {"selected": [], "other_humans": []}
        games = set()
        for row in rows:
            if row["split"] != split: continue
            money = np.asarray(row["hand_money"]).mean(axis=0)
            for seat, player in enumerate(row["players"]):
                if not row["humans"][seat]: continue
                group = "selected" if player in selected else "other_humans"
                groups[group].append(float(money[seat]))
                if group == "selected": games.add(row["id"])
        summary.append({"split": split, "games_with_selected": len(games),
            **{group: {"player_games": len(values),
                       "mean_money_per_hand": float(np.mean(values)) if values else None}
               for group, values in groups.items()}})
    with args.output.open("w") as out:
        for row in rows:
            row["skilled"] = [h and p in selected for p,h in zip(row["players"],row["humans"])]
            out.write(json.dumps(row,separators=(",", ":"))+"\n")
    report = {"selection": "top third among humans with >=30 training games; ridge 30; training only",
        "rating_target": "money/hand = own strength minus mean opposing strength",
        "selected_keys": sorted(int(p) for p in selected), "eligible": ratings,
        "performance_by_split": summary}
    args.report.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report,indent=2))


if __name__ == "__main__": main()
