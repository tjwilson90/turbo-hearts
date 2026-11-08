#!/usr/bin/env python3
"""Fit passing/charging advantages from paired responsive terminal money.

MSE targets the conditional mean money, including rare charged moons. Loss and
selection weight decisions equally; all candidates of a deal share its split.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
import numpy as np

FEATURES, HIDDEN, ROW = 256, 32, 262


def validation_mask(ids):
    # Mix bits before modulo: validation must not select only two hero seats.
    x = ids.astype(np.uint64) + np.uint64(0x9E3779B97F4A7C15)
    x = (x ^ (x >> 30)) * np.uint64(0xBF58476D1CE4E5B9)
    x = (x ^ (x >> 27)) * np.uint64(0x94D049BB133111EB)
    return ((x ^ (x >> 31)) % 10) == 0


def group_rows(rows):
    starts = np.r_[0, np.flatnonzero(np.any(rows[1:, :2] != rows[:-1, :2], axis=1)) + 1]
    lengths = np.diff(np.r_[starts, len(rows)])
    width = lengths.max()
    x = np.zeros((len(starts), width, FEATURES), 'f4')
    y = np.zeros((len(starts), width), 'f4')
    legal = np.arange(width)[None, :] < lengths[:, None]
    baseline = np.zeros(len(starts), int)
    for i, (start, n) in enumerate(zip(starts, lengths)):
        x[i, :n] = rows[start:start+n, 6:]
        y[i, :n] = rows[start:start+n, 5]
        selected = np.flatnonzero(rows[start:start+n, 4])
        assert len(selected) == 1 and abs(y[i, :n].sum()) < 1e-4
        baseline[i] = selected[0]
    return x, y, legal, baseline, rows[starts, :4].copy()


def forward(params, x):
    w, b, v, c = params
    h = np.maximum(x @ w + b, 0)
    return h, h @ v + c


def loss_gradient(params, x, y, legal, reg=0.0):
    h, prediction = forward(params, x)
    n = len(x)
    count = legal.sum(1, keepdims=True).astype(x.dtype)
    prediction -= (prediction * legal).sum(1, keepdims=True) / count
    error = (prediction - y) * legal
    loss = np.sum(error**2 / count) / n
    delta = 2 * error / count / n
    delta = (delta - delta.sum(1, keepdims=True) / count) * legal
    w, b, v, c = params
    dh = delta[:, :, None] * v * (h > 0)
    grads = [x.reshape(-1, FEATURES).T @ dh.reshape(-1, HIDDEN), dh.sum((0, 1)),
             (h * delta[:, :, None]).sum((0, 1)), np.array([delta.sum()], dtype=x.dtype)]
    for i in [0, 2]:
        loss += reg * np.sum(params[i]**2) / 2
        grads[i] += reg * params[i]
    return float(loss), grads


def metrics(params, data):
    x, y, legal, baseline, meta = data
    scores = np.concatenate([forward(params, x[i:i+256])[1] for i in range(0, len(x), 256)])
    chosen = np.argmax(np.where(legal, scores, -np.inf), axis=1)
    target = y[np.arange(len(x)), chosen]
    reference = y[np.arange(len(x)), baseline]
    regret = 100 * (np.max(np.where(legal, y, -np.inf), axis=1) - target)
    return {'decisions': len(x), 'deals': len(np.unique(meta[:, 0])),
            'regret_money': float(regret.mean()),
            'target_gain_over_parent_money': float(100 * (target-reference).mean()),
            'gain_by_direction': [float(100 * (target-reference)[meta[:, 3] == d].mean())
                                  if np.any(meta[:, 3] == d) else None for d in range(4)]}


def save(path, params, kind):
    w, b, v, c = params
    values = np.concatenate([w.ravel(), b, v, c]).astype('<f4')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b'TPQ1' + struct.pack('<III', FEATURES, HIDDEN, kind) + values.tobytes())


def load(path, kind):
    raw = path.read_bytes()
    assert raw[:16] == b'TPQ1' + struct.pack('<III', FEATURES, HIDDEN, kind)
    a = np.frombuffer(raw[16:], dtype='<f4').copy()
    assert len(a) == FEATURES * HIDDEN + 2 * HIDDEN + 1 and np.isfinite(a).all()
    n = FEATURES * HIDDEN
    return [a[:n].reshape(FEATURES, HIDDEN), a[n:n+HIDDEN], a[n+HIDDEN:-1], a[-1:]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('data', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('kind', choices=['pass', 'charge'])
    parser.add_argument('--initial', type=Path)
    parser.add_argument('--epochs', type=int, default=80)
    parser.add_argument('--seed', type=int, default=91)
    args = parser.parse_args()
    kind = int(args.kind == 'charge')
    all_rows = np.memmap(args.data, dtype='<f4', mode='r').reshape(-1, ROW)
    rows = all_rows[all_rows[:, 2] == kind]
    assert len(rows) and np.isfinite(rows).all()
    valid = validation_mask(rows[:, 0])
    train, validation = group_rows(rows[~valid]), group_rows(rows[valid])
    assert not set(train[4][:, 0]) & set(validation[4][:, 0])
    rng = np.random.default_rng(args.seed)
    params = load(args.initial, kind) if args.initial else [
        rng.normal(0, .06, (FEATURES, HIDDEN)).astype('f4'), np.zeros(HIDDEN, 'f4'),
        rng.normal(0, .01, HIDDEN).astype('f4'), np.zeros(1, 'f4')]
    m, v = [np.zeros_like(p) for p in params], [np.zeros_like(p) for p in params]
    best, best_epoch, selected, step, history, lr = float('inf'), 0, None, 0, [], .002
    for epoch in range(1, args.epochs+1):
        order = rng.permutation(len(train[0]))
        for start in range(0, len(order), 128):
            idx = order[start:start+128]
            _, grads = loss_gradient(params, train[0][idx], train[1][idx], train[2][idx], 3e-4)
            step += 1
            for i, g in enumerate(grads):
                m[i] = .9*m[i] + .1*g
                v[i] = .999*v[i] + .001*g*g
                params[i] -= lr * (m[i]/(1-.9**step)) / (np.sqrt(v[i]/(1-.999**step)) + 1e-8)
        metric = metrics(params, validation)
        history.append({'epoch': epoch, 'learning_rate': lr, **metric})
        if metric['regret_money'] < best-1e-5:
            best, best_epoch, selected = metric['regret_money'], epoch, [p.copy() for p in params]
        if epoch % 5 == 0: print(json.dumps(history[-1]), flush=True)
        if epoch-best_epoch in [7, 14]: lr *= .5
        if epoch-best_epoch >= 21: break
    save(args.output, selected, kind)
    report = {'format': 'TPQ1', 'kind': args.kind, 'features': FEATURES, 'hidden': HIDDEN,
              'loss': 'decision-centered mean squared money / 100', 'regularization': 3e-4,
              'best_epoch': best_epoch, 'epochs': epoch, 'seed': args.seed,
              'initial': str(args.initial) if args.initial else None,
              'validation_split': 'splitmix64(deal_id) % 10 == 0',
              'train': metrics(selected, train), 'validation': metrics(selected, validation),
              'history': history, 'data_sha256': hashlib.sha256(args.data.read_bytes()).hexdigest(),
              'checkpoint_sha256': hashlib.sha256(args.output.read_bytes()).hexdigest()}
    args.output.with_suffix('.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k != 'history'}), flush=True)


if __name__ == '__main__':
    main()
