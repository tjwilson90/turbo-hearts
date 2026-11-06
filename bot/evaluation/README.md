# Evaluation, November 2026

The original four ONNX weight files were absent, so it was impossible to measure
the historical neural bot's playing strength. All comparisons below are against
working, reproducible baselines. The current `default` is `defend:128:12`:
128 shared determinizations, point-avoidance and shooting continuations, a
mixture model for opposing shooters, and exact 12-card endgames. Passing and
charging use the existing heuristic. Neural priors are experimental.

## Held-out matches

Configuration was selected on development seeds 1000–1999. The JSON files here
record fresh seeds starting at 50000, with four seat replays per original deal.
Each comparison runs the baseline on the same deals and opponent configuration.
95% intervals are paired and clustered by deal. Money per hand is the actual
zero-sum game payoff, including moons, charges, JD, and TC; higher is better.

| Opponents | Baseline | Hands per version | Default money | Baseline money | Paired gain (95% CI) |
| --- | --- | ---: | ---: | ---: | --- |
| Three heuristic | Search without moon planning | 8,000 | 42.75 | 35.30 | **+7.45 [5.08, 9.83]** |
| Heuristic + duck + GottaTry | Heuristic | 4,000 | 49.01 | 4.86 | **+44.16 [38.92, 49.40]** |
| Three GottaTry shooters | Moon planning without opponent model | 4,000 | -2.22 | -11.31 | **+9.09 [4.36, 13.82]** |
| Three search bots (32 samples) | Search without moon planning | 2,000 | 6.31 | 5.87 | +0.44 [-4.46, 5.35] |

The last row does **not** establish an improvement against stronger search opponents.

On the first row, completed moons increased from **247 to 560**. Near misses
(queen plus 8–12 hearts) also increased, from 150 to 269; the positive money
difference already includes those costs. Against three shooters, opponents'
completed moons decreased from 1,580 to 1,492. This shows a defensive improvement,
but the negative absolute money still exposes a weakness against that lineup.

Mean decision computation per *whole hand* was about 9–12 ms on this Apple
Silicon machine; maximum recorded individual decisions were below 27 ms in these
three runs. These timings were collected while other experiments ran and are
not a portable latency guarantee. The production one-second cap is soft: it
finishes a complete batch so all moves receive equal samples.

## Development iterations

These are exploratory comparisons, not independent confirmation. Larger
numbers of samples do not guarantee better play with an imperfect rollout model.

| Change | Opponents | Hands/version | Gain in money/hand (95% CI) | Decision |
| --- | --- | ---: | --- | --- |
| 128 vs 32 samples | Heuristic | 4,000 | +5.98 [3.27, 8.69] | Keep 128 |
| Exact 12-card endgame vs rollouts, 32 samples | Heuristic | 4,000 | -0.72 [-1.19, -0.25] | Opponent-dependent |
| Exact endgame, 128 samples | Search with 32 samples | 2,000 | +1.72 [0.61, 2.83] | Retain for stronger play |
| 512 vs 128 samples, no exact endgame | Heuristic | 4,000 | +2.08 [-0.03, 4.18] | Inconclusive |
| 2048 vs 512 samples | Heuristic | 2,000 | +1.90 [-0.65, 4.45] | Inconclusive |
| Search all passes, 16 samples each | Heuristic | 4,000 | -0.69 [-3.74, 2.35] | Removed |
| First neural prior, 32 pseudo-samples at 128 samples | Heuristic | 4,000 | +0.46 [-1.00, 1.93] | Opt-in |
| First neural prior, 128 pseudo-samples | Heuristic | 4,000 | -1.73 [-3.73, 0.27] | Do not enable |
| Add own shooting continuation | Heuristic | 4,000 | +8.48 [5.46, 11.51] | Keep |
| Add own shooting continuation | Duck | 4,000 | +18.49 [16.02, 20.97] | Keep |
| Add own shooting continuation | Search with 32 samples | 2,000 | +0.91 [-3.89, 5.71] | Uncertain vs stronger field |
| Add opposing-shooter mixture | GottaTry | 4,000 | +6.49 [2.00, 10.98] | Keep |
| Add opposing-shooter mixture | Heuristic | 4,000 | -0.57 [-3.22, 2.08] | No clear regression |
| 512 vs 128 samples with moon planning | Heuristic | 2,000 | +0.35 [-3.60, 4.31] | Keep faster 128 |
| Moon-trained prior, 16 pseudo-samples | Heuristic | 4,000 | +0.74 [-0.87, 2.36] | Opt-in |
| Moon-trained prior, 16 pseudo-samples | Mixed | 4,000 | -0.48 [-2.43, 1.48] | Opt-in |

## Neural checkpoints

Both new models are checked in with SHA-256 and training metadata, so their
weights can be reproduced or reused. Runtime inference uses the same versioned
Rust feature encoder as data generation, and a test compares the exported
weights against an independent NumPy forward pass.

- `policy-avoid.thp`: 477,032 training actions from 3,600 hands; 53,750 actions
  from 400 separate validation hands. Teacher: 64-sample ordinary search.
  Best validation regret: 5.10 money, epoch 39 of 45. Training seeds 10000–13999.
- `policy-moon.thp`: 489,384 training actions from 3,600 hands; 54,236 actions
  from 400 separate validation hands. Teacher: 64-sample moon-aware search with
  opponent modeling. Best validation regret: 8.13 money, epoch 20 of 45.
  Training seeds 20000–23999. These regret values have different teachers and
  target distributions, so they are not a direct comparison of model quality.

The networks predict centered action advantages, and the blend is an explicit
optional prior. The default avoids relying on a network until match performance
justifies it. Hold out complete deals when training and reserve fresh match seeds
when selecting future variants; loss on randomly split action rows would leak
almost identical positions across splits.

Useful next experiments include fitting opponents' shooting tendencies from
public action history, improving shooting continuations against defenders,
training on more varied opponents, and using a learned policy to allocate a
fixed search budget. Uniform hidden deals do not yet infer ownership from
opponents' voluntary passing or playing preferences.

## Validation

`cargo test --locked --workspace --all-targets` passed: 68 unit/integration tests,
plus example compilation and benchmark smoke checks. New tests exercise all six
charging rules and all four passing directions. The default bot also completed
matches when launched from outside the repository, without loading any weights.
