#!/usr/bin/env python3
"""Fit small conditional neural policies or counterfactual action advantages.

THH1 stores input/hidden/output dimensions and a role tag; inference is Rust-only.
Whole-game splits and actor-only features are supplied by the engine exporter.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
import numpy as np

INPUTS, FEATURES, OUTPUTS, ROW = 363, 28, 80, 746


def unpack(rows, inputs):
    x = rows[:, 6:6+inputs].copy()
    a = rows[:, 6+INPUTS:].reshape(-1, 13, 29)
    ids = a[:, :, 0].astype(int)
    features = a[:, :, 1:]
    legal = np.arange(13)[None, :] < rows[:, 5:6]
    return x, ids, features, legal


def forward(params, x, ids, features, legal):
    w1, b1, w2, b2 = params
    h = np.maximum(x@w1+b1, 0)
    c = h@w2+b2
    scores = np.einsum('bla,ba->bl', features, c[:, :FEATURES])
    scores += np.take_along_axis(c[:, FEATURES:], ids, axis=1)
    scores = np.where(legal, scores, -1e9)
    return h, scores


def loss_gradient(params, x, ids, features, legal, chosen, reg=0, targets=None):
    h, scores = forward(params, x, ids, features, legal)
    n = len(x)
    if targets is None:
        p = np.exp(scores-scores.max(axis=1, keepdims=True))
        p /= p.sum(axis=1, keepdims=True)
        loss = -np.log(np.maximum(p[np.arange(n), chosen], 1e-30)).mean()
        ds = p.copy(); ds[np.arange(n), chosen] -= 1; ds /= n
    else:
        # Center each comparison; unknown position difficulty is not action credit.
        count = legal.sum(axis=1, keepdims=True)
        centered = np.where(legal, scores, 0)
        centered -= centered.sum(axis=1, keepdims=True)/count
        error = np.where(legal, centered-targets, 0)
        absolute = np.abs(error)
        loss = (np.where(absolute <= 1, .5*error**2, absolute-.5)/count).sum()/n
        ds = np.clip(error, -1, 1)/count/n
        ds = np.where(legal, ds-ds.sum(axis=1, keepdims=True)/count, 0)
    dc = np.zeros((n, OUTPUTS), dtype=x.dtype)
    dc[:, :FEATURES] = np.einsum('bl,bla->ba', ds, features)
    np.add.at(dc, (np.repeat(np.arange(n), 13), (FEATURES+ids).ravel()), ds.ravel())
    w1, b1, w2, b2 = params
    dh = (dc@w2.T)*(h>0)
    grads = [x.T@dh, dh.sum(axis=0), h.T@dc, dc.sum(axis=0)]
    for i in [0, 2]:
        loss += reg*(params[i]**2).sum()/2
        grads[i] += reg*params[i]
    return float(loss), grads


def evaluate(params, rows, inputs, targets=None, mask=False):
    total = 0.0
    for start in range(0, len(rows), 1024):
        b = rows[start:start+1024]
        x, ids, features, legal = unpack(b, inputs)
        if mask and inputs == INPUTS: x[:, 174:330] = 0; x[:, 362] = 0
        t = None if targets is None else targets[start:start+len(b)]
        total += len(b)*loss_gradient(params, x, ids, features, legal, b[:, 4].astype(int), targets=t)[0]
    return total/len(rows)


def export(path, params, inputs, hidden, advantage):
    w1, b1, w2, b2 = params
    values = np.concatenate([w1.T.ravel(), b1, w2.ravel(), b2]).astype('<f4')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b'THH1'+struct.pack('<4I', inputs, hidden, OUTPUTS, int(advantage))+values.tobytes())


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('data', type=Path); p.add_argument('output', type=Path)
    p.add_argument('--inputs', type=int, choices=[18, INPUTS], default=INPUTS)
    p.add_argument('--hidden', type=int, choices=[32, 64], default=32)
    p.add_argument('--epochs', type=int, default=80)
    p.add_argument('--legacy-actions', action='store_true', help='Capacity control using only the old 24 action features')
    p.add_argument('--seed', type=int, default=81)
    p.add_argument('--targets', type=Path, help='13 centered money/100 targets per decision')
    p.add_argument('--initialize', type=Path, help='Reuse and freeze the hidden features of a policy checkpoint')
    args = p.parse_args()
    rows = np.memmap(args.data, dtype='<f4', mode='r').reshape(-1, ROW)
    splits = [np.flatnonzero(rows[:, 1]==s) for s in range(3)]
    assert all(len(s) for s in splits)
    assert all(not set(rows[a,0])&set(rows[b,0]) for a,b in [(splits[0],splits[1]),(splits[0],splits[2]),(splits[1],splits[2])])
    targets = None if args.targets is None else np.fromfile(args.targets, dtype='<f4').reshape(-1,13)
    if targets is not None: assert len(targets)==len(rows)
    rng = np.random.default_rng(args.seed)
    params = [rng.normal(0, .08, (args.inputs,args.hidden)).astype('f4'), np.zeros(args.hidden,'f4'),
              rng.normal(0, .03, (args.hidden,OUTPUTS)).astype('f4'), np.zeros(OUTPUTS,'f4')]
    if args.legacy_actions: params[2][:,24:]=0;params[3][24:]=0
    if args.initialize:
        assert targets is not None
        raw=args.initialize.read_bytes()
        assert raw[:4]==b'THH1' and struct.unpack('<4I',raw[4:20])==(args.inputs,args.hidden,OUTPUTS,0)
        weights=np.frombuffer(raw[20:],dtype='<f4')
        n=args.inputs*args.hidden
        params[0]=weights[:n].reshape(args.hidden,args.inputs).T.copy()
        params[1]=weights[n:n+args.hidden].copy()
        params[2][:]=0;params[3][:]=0
    m, v = [np.zeros_like(a) for a in params], [np.zeros_like(a) for a in params]
    train, validation = rows[splits[0]], rows[splits[1]]
    train_targets = None if targets is None else targets[splits[0]]
    val_targets = None if targets is None else targets[splits[1]]
    best, best_epoch, step, lr, history = float('inf'), 0, 0, .003, []
    for epoch in range(1, args.epochs+1):
        order = rng.permutation(len(train))
        for offset in range(0, len(order), 512):
            idx = order[offset:offset+512]; batch = train[idx]
            x, ids, features, legal = unpack(batch, args.inputs)
            if args.inputs == INPUTS:
                mask = rng.random(len(x))<.5
                x[mask, 174:330] = 0; x[mask, 362] = 0
            t = None if targets is None else train_targets[idx]
            _, grads = loss_gradient(params, x, ids, features, legal, batch[:,4].astype(int), 1e-4, t)
            if args.legacy_actions: grads[2][:,24:]=0;grads[3][24:]=0
            step += 1
            for i, g in enumerate(grads):
                if args.initialize and i<2: continue
                m[i] = .9*m[i]+.1*g; v[i] = .999*v[i]+.001*g*g
                params[i] -= lr*(m[i]/(1-.9**step))/(np.sqrt(v[i]/(1-.999**step))+1e-8)
        full = evaluate(params, validation, args.inputs, val_targets)
        masked = evaluate(params, validation, args.inputs, val_targets, mask=True)
        val = (full+masked)/2
        history.append({'epoch':epoch, 'validation_full':full, 'validation_masked':masked, 'lr':lr})
        if val < best-1e-5:
            best, best_epoch, selected = val, epoch, [a.copy() for a in params]
        if epoch%5==0: print(json.dumps(history[-1]), flush=True)
        if epoch-best_epoch in [5,10]: lr *= .5
        if epoch-best_epoch>=15: break
    metrics = []
    for s, indices in enumerate(splits):
        t = None if targets is None else targets[indices]
        metrics.append({'split':s,'decisions':len(indices),
                        'full':evaluate(selected, rows[indices], args.inputs,t),
                        'masked':evaluate(selected, rows[indices],args.inputs,t,mask=True)})
    export(args.output, selected, args.inputs, args.hidden, targets is not None)
    report = {'role':'counterfactual money advantage / 100' if targets is not None else 'human move logits',
              'inputs':args.inputs,'hidden':args.hidden,'parameters':sum(a.size for a in selected),
              'active_parameters':args.inputs*args.hidden+args.hidden+(args.hidden+1)*(24 if args.legacy_actions else OUTPUTS),
              'legacy_actions':args.legacy_actions,
              'best_epoch':best_epoch,'epochs':epoch,'epochs_requested':args.epochs,'seed':args.seed,'regularization':1e-4,
              'private_pass_mask_probability':.5 if args.inputs == INPUTS else 0.0,
              'initialize':str(args.initialize) if args.initialize else None,
              'metrics':metrics,'history':history,
              'checkpoint_sha256':hashlib.sha256(args.output.read_bytes()).hexdigest(),
              'data_sha256':hashlib.sha256(args.data.read_bytes()).hexdigest(),
              'targets_sha256':hashlib.sha256(args.targets.read_bytes()).hexdigest() if args.targets else None,
              'initialize_sha256':hashlib.sha256(args.initialize.read_bytes()).hexdigest() if args.initialize else None}
    args.output.with_suffix('.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='history'}), flush=True)


if __name__=='__main__': main()
