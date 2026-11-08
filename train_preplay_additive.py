#!/usr/bin/env python3
"""Low-variance charging control: ridge-fit each charge's contextual money value.

The sum of four context-dependent charge values is exported exactly as a TPQ1
network with eight active ReLU units. It deliberately omits interactions among
simultaneously selected charges. Whole-deal splits and monetary targets match
the nonlinear preplay trainer.
"""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from train_preplay import FEATURES, HIDDEN, ROW, group_rows, metrics, save, validation_mask

CONTEXT = np.r_[0:52, 156:220, 252:256]
CHARGES = 104 + np.array([8, 13 + 9, 26 + 12, 39 + 10])


def export_parameters(coefficients):
    w, b, v, c = (np.zeros((FEATURES, HIDDEN), 'f4'), np.zeros(HIDDEN, 'f4'),
                  np.zeros(HIDDEN, 'f4'), np.zeros(1, 'f4'))
    for suit in range(4):
        # Context inputs are bounded by [0, 1]. Gating makes both units zero
        # for an unselected charge, and relu(value)-relu(-value) otherwise.
        gate = float(np.abs(coefficients[suit]).sum() + 1)
        for offset, sign in [(0, 1), (1, -1)]:
            unit = 2 * suit + offset
            w[CONTEXT, unit] = sign * coefficients[suit]
            w[CHARGES[suit], unit] = gate
            b[unit] = -gate
            v[unit] = sign
    return [w, b, v, c]


def normal_equations(data):
    x, y, legal, _, _ = data
    context = x[:, 0, CONTEXT]
    assert np.all((context >= 0) & (context <= 1))
    n = legal.sum(1, keepdims=True)
    actions = x[:, :, CHARGES]
    actions = actions - (actions * legal[:, :, None]).sum(1, keepdims=True) / n[:, :, None]
    design = (actions[:, :, :, None] * context[:, None, None, :]).reshape(*legal.shape, -1)
    design = (design / np.sqrt(n[:, :, None]))[legal]
    target = (y / np.sqrt(n))[legal]
    return design.T @ design / len(x), design.T @ target / len(x)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('data', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    all_rows = np.memmap(args.data, dtype='<f4', mode='r').reshape(-1, ROW)
    rows = all_rows[all_rows[:, 2] == 1]
    valid = validation_mask(rows[:, 0])
    train, validation = group_rows(rows[~valid]), group_rows(rows[valid])
    gram, rhs = normal_equations(train)
    history, selected, best = [], None, float('inf')
    for penalty in [0.0001, 0.001, 0.01, 0.1]:
        coefficients = np.linalg.solve(gram + penalty * np.eye(len(rhs)), rhs).reshape(4, -1)
        params = export_parameters(coefficients)
        metric = metrics(params, validation)
        history.append({'ridge': penalty, **metric})
        print(json.dumps(history[-1]), flush=True)
        if metric['regret_money'] < best:
            best, selected, chosen = metric['regret_money'], params, penalty
    save(args.output, selected, 1)
    report = {'format': 'TPQ1', 'kind': 'charge', 'architecture': 'additive contextual ridge',
              'context_features_per_charge': len(CONTEXT), 'active_hidden_units': 8,
              'ridge': chosen, 'loss': 'decision-centered squared money / 100',
              'validation_split': 'splitmix64(deal_id) % 10 == 0',
              'train': metrics(selected, train), 'validation': metrics(selected, validation),
              'selection': history, 'data_sha256': hashlib.sha256(args.data.read_bytes()).hexdigest(),
              'checkpoint_sha256': hashlib.sha256(args.output.read_bytes()).hexdigest()}
    args.output.with_suffix('.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
