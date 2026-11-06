#!/usr/bin/env python3
"""Population self-play: clipped policy-gradient updates from terminal money.

This is policy learning, not search imitation. Checkpoints remain THP1 scalar
networks but their outputs are policy scores, NOT calibrated monetary values.
"""
import argparse
import json
import subprocess
from pathlib import Path
import numpy as np
from neural_net import FEATURES, forward, export, load


def policy_loss_gradient(logits, chosen, old, mask, advantage, entropy_weight=0.01):
    """PPO surrogate loss and its gradient with respect to masked logits."""
    logits = np.where(mask, logits, -1e9)
    logits = logits - logits.max(axis=1, keepdims=True)
    probs = np.exp(np.maximum(logits, -60)) * mask
    probs /= probs.sum(axis=1, keepdims=True)
    probability = (probs * chosen).sum(axis=1)
    old_probability = (old * chosen).sum(axis=1)
    if np.any(old_probability <= 0):
        raise ValueError("chosen actions must have positive behavior probability")
    ratio = probability / old_probability
    active = np.where(advantage >= 0, ratio < 1.2, ratio > 0.8)
    grad = (probs - chosen) * (ratio * advantage * active)[:, None]
    logp = np.log(np.maximum(probs, 1e-30))
    entropy = -(probs * logp).sum(axis=1, keepdims=True)
    grad += entropy_weight * probs * (logp + entropy)
    surrogate = np.minimum(ratio * advantage, np.clip(ratio, 0.8, 1.2) * advantage)
    loss = -np.mean(surrogate + entropy_weight * entropy[:, 0])
    return float(loss), grad / len(logits)


def update(params, path, temperature, rng, epochs=4, learning_rate=0.0001):
    if not np.isfinite(temperature) or temperature <= 0:
        raise ValueError("training temperature must be finite and positive")
    rows = np.fromfile(path, dtype="<f4").reshape(-1, FEATURES + 5)
    if not len(rows) or not np.isfinite(rows).all():
        raise ValueError("empty or nonfinite trajectories")
    starts = np.r_[0, np.flatnonzero(np.any(rows[1:, :2] != rows[:-1, :2], axis=1)) + 1]
    lengths = np.diff(np.r_[starts, len(rows)])
    width = int(lengths.max())
    x = np.zeros((len(starts), width, FEATURES), np.float32)
    chosen = np.zeros((len(starts), width), np.float32)
    old = np.zeros((len(starts), width), np.float32)
    mask = np.zeros((len(starts), width), bool)
    advantage = rows[starts, 4].copy()
    # Scaling changes the optimization step size, not the money objective.
    advantage /= max(float(advantage.std()), 1.0)
    for i, (s, n) in enumerate(zip(starts, lengths)):
        x[i, :n] = rows[s:s+n, 5:]
        chosen[i, :n] = rows[s:s+n, 2]
        old[i, :n] = rows[s:s+n, 3]
        mask[i, :n] = True
    assert np.all(chosen.sum(axis=1) == 1)
    assert np.allclose(old.sum(axis=1), 1, atol=1e-5)
    del rows
    moments = [np.zeros_like(p) for p in params]
    velocities = [np.zeros_like(p) for p in params]
    step = 0
    for epoch in range(epochs):
        for idx in np.array_split(rng.permutation(len(x)), max(1, (len(x) + 255) // 256)):
            xb = x[idx].reshape(-1, FEATURES)
            h, out = forward(xb, params)
            logits = out.reshape(len(idx), width) * (100 / temperature)
            _, grad = policy_loss_gradient(logits, chosen[idx], old[idx], mask[idx], advantage[idx])
            delta = grad.reshape(-1, 1) * (100 / temperature)
            dh = (delta @ params[2].T) * (h > 0)
            grads = [xb.T @ dh, dh.sum(axis=0), h.T @ delta, delta.sum(axis=0)]
            norm = np.sqrt(sum(float((g * g).sum()) for g in grads))
            step += 1
            for p, m, v, g in zip(params, moments, velocities, grads):
                g = g / max(1, norm)
                m *= 0.9
                m += 0.1 * g
                v *= 0.999
                v += 0.001 * g * g
                p -= learning_rate * (m / (1 - 0.9**step)) / (np.sqrt(v / (1 - 0.999**step)) + 1e-8)
        print(json.dumps({"epoch": epoch + 1, "decisions": len(x),
                          "mean_paired_advantage": float(advantage.mean())}), flush=True)
    return params


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--initial", type=Path, default=Path("assets/models/policy-moon.thp"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--collector", default="target/release/examples/self_play")
    parser.add_argument("--hands", type=int, default=4000)
    parser.add_argument("--generations", type=int, default=3)
    parser.add_argument("--seed", type=int, default=100000)
    parser.add_argument("--temperature", type=float, default=20)
    args = parser.parse_args()
    if args.hands <= 0 or args.generations <= 0 or args.seed < 0:
        parser.error("hands and generations must be positive, and seed nonnegative")
    if not np.isfinite(args.temperature) or args.temperature <= 0:
        parser.error("temperature must be finite and positive")
    args.output.mkdir(parents=True, exist_ok=True)
    current = args.initial
    archives = [str(args.initial)]
    for generation in range(1, args.generations + 1):
        data = args.output / f"trajectories-{generation}.bin"
        seed = args.seed + (generation - 1) * args.hands
        command = [args.collector, str(data), str(current), str(args.hands),
                   str(seed), str(args.temperature), *archives]
        run = subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE)
        metrics = json.loads(run.stdout)
        print(json.dumps(metrics), flush=True)
        params = update(load(current), data, args.temperature, np.random.default_rng(seed))
        destination = args.output / f"policy-rl-{generation}.thp"
        checksum = export(destination, params)
        report = {"objective": "clipped policy gradient on paired terminal money", "role": "policy_scores",
                  "generation": generation, "initial": str(current), "seed": seed, "hands": args.hands,
                  "temperature": args.temperature, "epochs": 4, "learning_rate": 0.0001, "sha256": checksum,
                  "collection": metrics, "command": command}
        destination.with_suffix(".json").write_text(json.dumps(report, indent=2) + "\n")
        archives.append(str(destination))
        current = destination
        print("Saved " + str(destination), flush=True)


if __name__ == "__main__":
    main()
