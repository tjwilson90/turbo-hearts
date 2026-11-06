#!/usr/bin/env python3
"""Train a compact action-value prior from `policy_data` using only NumPy.

The Rust generator owns feature encoding. Entire deals are held out so that
candidate actions and later positions cannot leak across train/validation.
The best validation *decision regret*, rather than training loss, selects weights.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path

import numpy as np

FEATURES = 256
HIDDEN = 64


def forward(x, params):
    w, b, v, c = params
    h = np.maximum(x @ w + b, 0)
    return h, h @ v + c


def export(path, params):
    w, b, v, c = params
    data = b"THP1" + struct.pack("<II", FEATURES, HIDDEN)
    for array in (w.T, b, v, c):
        data += np.asarray(array, dtype="<f4").tobytes()
    path.write_bytes(data)
    return hashlib.sha256(data).hexdigest()


def load(path):
    data = Path(path).read_bytes()
    expected = 12 + 4 * (FEATURES * HIDDEN + 2 * HIDDEN + 1)
    if len(data) != expected or data[:12] != b"THP1" + struct.pack("<II", FEATURES, HIDDEN):
        raise ValueError("incompatible network")
    a = np.frombuffer(data[12:], dtype="<f4").copy()
    if not np.isfinite(a).all():
        raise ValueError("nonfinite network weights")
    n = FEATURES * HIDDEN
    return [a[:n].reshape(HIDDEN, FEATURES).T.copy(), a[n:n+HIDDEN].copy(),
            a[n+HIDDEN:-1].reshape(HIDDEN, 1).copy(), a[-1:].copy()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("data", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--epochs", type=int, default=60)
    parser.add_argument("--batch-size", type=int, default=1024)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--learning-rate", type=float, default=0.001)
    parser.add_argument("--initial", type=Path, help="fine-tune an existing THP1 checkpoint")
    args = parser.parse_args()
    rows = np.fromfile(args.data, dtype="<f4").reshape(-1, FEATURES + 3)
    if not len(rows) or not np.isfinite(rows).all():
        raise ValueError("empty data or nonfinite data")
    valid = rows[:, 0].astype(np.int64) % 10 == 0
    if not valid.any() or valid.all():
        raise ValueError("need both training and validation deals")
    x_train = rows[~valid, 2:-1].copy()
    y_train = rows[~valid, -1:].copy()
    x_valid = rows[valid, 2:-1].copy()
    y_valid = rows[valid, -1].copy()
    ids = rows[valid, :2]
    starts = np.r_[0, np.flatnonzero(np.any(ids[1:] != ids[:-1], axis=1)) + 1]
    lengths = np.diff(np.r_[starts, len(ids)])
    groups = np.repeat(np.arange(len(starts)), lengths)
    best_targets = np.maximum.reduceat(y_valid, starts)
    first_deal, last_deal = int(rows[:, 0].min()), int(rows[:, 0].max())
    train_deals = len(np.unique(rows[~valid, 0]))
    valid_deals = len(np.unique(rows[valid, 0]))
    del rows
    rng = np.random.default_rng(args.seed)
    params = load(args.initial) if args.initial else [
        (rng.standard_normal((FEATURES, HIDDEN)) * np.sqrt(2 / FEATURES)).astype(np.float32),
        np.zeros(HIDDEN, np.float32),
        (rng.standard_normal((HIDDEN, 1)) * 0.01).astype(np.float32),
        np.zeros(1, np.float32),
    ]
    moments = [np.zeros_like(p) for p in params]
    velocities = [np.zeros_like(p) for p in params]
    best_regret = float("inf")
    step = 0
    history = []
    print(json.dumps({"train_rows": len(x_train), "validation_rows": len(x_valid),
                      "train_deals": train_deals, "validation_deals": valid_deals}), flush=True)
    for epoch in range(1, args.epochs + 1):
        order = rng.permutation(len(x_train))
        for begin in range(0, len(order), args.batch_size):
            idx = order[begin:begin + args.batch_size]
            x, y = x_train[idx], y_train[idx]
            h, prediction = forward(x, params)
            # Huber loss limits the influence of rare, very large charged moons.
            delta = np.clip(prediction - y, -1, 1) / len(idx)
            dh = (delta @ params[2].T) * (h > 0)
            grads = [x.T @ dh + 1e-5 * params[0], dh.sum(axis=0),
                     h.T @ delta + 1e-5 * params[2], delta.sum(axis=0)]
            step += 1
            for p, m, v, grad in zip(params, moments, velocities, grads):
                m *= 0.9
                m += 0.1 * grad
                v *= 0.999
                v += 0.001 * grad * grad
                p -= args.learning_rate * (m / (1 - 0.9**step)) / (np.sqrt(v / (1 - 0.999**step)) + 1e-8)
        predictions = np.concatenate([forward(x_valid[i:i+4096], params)[1][:, 0]
                                      for i in range(0, len(x_valid), 4096)])
        maxima = np.maximum.reduceat(predictions, starts)
        chosen = np.minimum.reduceat(np.where(predictions == maxima[groups], np.arange(len(predictions)), len(predictions)), starts)
        regret = float(np.mean(best_targets - y_valid[chosen]) * 100)
        mse = float(np.mean((predictions - y_valid)**2))
        metric = {"epoch": epoch, "validation_mse": mse, "validation_regret_money": regret}
        history.append(metric)
        print(json.dumps(metric), flush=True)
        if regret < best_regret:
            best_regret = regret
            checksum = export(args.output, params)
            report = {"format": "THP1", "features": FEATURES, "hidden": HIDDEN,
                      "seed": args.seed, "batch_size": args.batch_size,
                      "first_deal": first_deal, "last_deal": last_deal,
                      "learning_rate": args.learning_rate,
                      "initial": str(args.initial) if args.initial else None,
                      "train_deals": train_deals, "validation_deals": valid_deals,
                      "train_rows": len(x_train), "validation_rows": len(x_valid),
                      "validation_split": "deal_id % 10 == 0", "best": metric,
                      "sha256": checksum}
            args.output.with_suffix(".json").write_text(json.dumps(report, indent=2) + "\n")
    args.output.with_suffix(".history.json").write_text(json.dumps(history, indent=2) + "\n")


if __name__ == "__main__":
    main()
