# Seven-direction exploration

This round starts from commit `826b6cc`. The frozen search baseline is
`defend:128:12`: 128 sampled deals, two own continuations, an opposing-shooter
mixture, and exact 12-card endgames. No historical lost weights are needed.

All seven directions have executable implementations and measured matches.
**This round kept the server's default unchanged.** Opponent modeling improved the
mixed lineup on fresh deals but regressed against heuristic and search opponents, failing
the promotion criterion. The more aggressive combinations also remain research
variants. Population self-play produced a stronger standalone network; its
checkpoint is retained as a fast research alternative. The subsequent
[third-round follow-up](../round3/README.md) produced a more selective opponent
model and became the new default; the results on this page retain their frozen
baseline and historical selection decision.

## Evaluation design

Development uses seeds 70000–70999; confirmation uses 90000–91999. Population
training uses 100000–111999 and hard-position collection uses 120000–121999.
Each original match deal is replayed in all four seats for each version. Money
is the actual terminal payoff, including charged moons, JD, and TC. Intervals
are paired 95% intervals clustered by original deal. Screens share deals and
are exploratory, without multiple-comparison correction. See [PLAN.md](PLAN.md)
for the choice made before confirmation results were opened.

The match harness redacts events **and game state**, including blind charges.
Play features and observation-tree keys cannot access sampled opponent hands.
Original behavior was checked against the prior round's seeded fingerprint:
1,000 mixed deals starting at 1000 yielded money 55.20375, 316 own moons, and
751 opposing moons for both baseline replays.

## Findings by direction

| Direction | Implementation and result | Disposition |
| --- | --- | --- |
| Learn opponents | Infer a mixture of heuristic, duck, and shooting policies from each opponent's observed choices. Fresh mixed gain **+17.69 [14.32, 21.05]**, but losses against heuristic **−5.05 [−8.54, −1.55]** and full-strength search **−7.82 [−10.93, −4.72]**. | Keep optional; failed the generalization criterion. |
| Infer hidden hands from behavior | Weight otherwise legal sampled hands by tempered action likelihood. Alone: **−2.79 [−9.97, 4.39]**. With the opponent model: **+9.82 [1.59, 18.04]** versus that model on mixed opponents, but **−6.86 [−14.16, 0.45]** versus baseline on search opponents. | Keep optional; misspecified behavior can distort ownership beliefs. |
| Population self-play | Three generations of 4,000 hands; policy-gradient updates on terminal money against diverse bots and archived policies. Generation 3 development gain: **+1.98 [−1.97, 5.92]** over the initial direct policy. Fresh deals: **+6.53 [2.74, 10.32]**. | Retain the improved fast policy checkpoint; do not substitute it for search or a calibrated value prior. |
| Plan beyond fixed continuations | Explore two future own decisions using shared public-observation trees. At 128 samples: **−0.81 [−8.53, 6.92]**. At 512 samples versus equally budgeted search: **+0.18 [−7.71, 8.07]**. Adding the tree to behavioral beliefs was also inconclusive. | Keep optional; this tree abstraction did not improve matches. |
| Joint passing, charging, play | Compare pass candidates under three charging plans and two play continuations; enumerate charge subsets at the actual charge decision. The full combination with behavior and belief models gained **+72.65 [58.50, 86.81]** on mixed bots but lost **−26.54 [−37.86, −15.22]** against search bots. | Keep optional. Stronger opponents punish the speculative moons. |
| Mine difficult positions | Select disagreements, small action gaps, and live moon decisions; label with 256-sample search. Match 7,996 hard positions against 7,996 uniformly selected positions from the same 2,000 games. At 45 training epochs, hard minus uniform: **−2.14 [−6.92, 2.64]**; extending both to 120 epochs gave **−2.92 [−7.67, 1.84]**. | No demonstrated playing-strength gain. |
| Neural allocation of search | Successive halving at a fixed root-evaluation budget, with/without neural guidance. Waiting 32 rounds improved on the aggressive 8-round prototype, but the guided version still gave **−0.48 [−6.61, 5.65]** versus ordinary search. At 16 samples, neural guidance versus unguided allocation gave **−1.97 [−7.48, 3.54]**. | Keep ordinary sampling as the default. |

All values are money per hand, higher is better. Unless noted otherwise,
development rows use 300 original deals (1,200 hands per version); standalone
network comparisons use 1,000 deals. JSON files retain exact lineups, seeds,
budgets, moon counts, win shares, and computation costs. The complete screening
table is in [SCREENING.md](SCREENING.md), and the fresh-deal results and final
selection decision are in [CONFIRMATION.md](CONFIRMATION.md).

The fresh opponent-model comparison against mixed bots used 2,000 original
deals (8,000 hands per version). Own completed moons rose from 573 to 862;
opposing moons fell from 1,582 to 1,309. Against heuristic bots, own moons also
rose, from 280 to 305, but queen-plus-8–12-hearts near misses rose from 104 to
166. That comparison lost money overall. Counting more completed moons alone
would have selected a regression.

The final self-play network scored 13.49 money per hand against mixed opponents
on 1,000 fresh deals, compared with 6.96 for its initial network and 50.08 for
the production search bot on those same deals. Its improvement is real in this
test, but it is a faster, weaker alternative to search—not a reason to replace
the server bot.

Mean whole-hand play computation on fresh mixed deals was 14.86 ms for the
model versus 11.83 ms for baseline. Joint pre-play added about 25 ms per hand;
charge-only search added about 0.4 ms. The direct neural policies used about
0.28 ms per hand. These are concurrent-run timings on this machine, not latency
guarantees; production retains the existing one-second soft decision budget.

## What these experiments actually model

The opponent model learns **within a hand**; it does not maintain persistent
player identities. Its three-policy prior is 60% heuristic, 20% duck, 20%
shooting. The probability of an observed action mixes 75% policy support with
25% uniform legal moves. Forced choices provide no evidence. A half-power
likelihood tempers the style posterior. Model-only search averages conditional
style probabilities across uniformly sampled hands; it is an approximation,
not exact Bayesian inference. Opponent styles stay fixed within each rollout.

The belief variant further resamples hands using a half-power of the
marginalized style likelihood. It changes probabilities within the constrained
sampler's support; it never relaxes known ownership or public voids. Effective
sample size is available through `last_belief_ess()`.

The tree is a prototype with a coarse observation abstraction: own action
history, legal moves, captured scoring cards, trick suit/length, whether the bot
is winning, and the own continuation mode. It pools compatible decisions and
uses UCB exploration. Opponents still use fixed sampled policies. It is neither
a full information-set equilibrium solver nor a complete history tree. Larger
budgets did not overcome the abstraction and rollout limitations in these runs.

Pre-play search evaluates heuristic, duck, and shooting passes plus one-card
replacements, using 32 shared sampled deals. It evaluates no/heuristic/all
charging plans and avoidance/shooting continuations, averaging each plan across
deals before choosing it. Actual charge search enumerates all legal subsets and
reconsiders after receiving cards. Keeper discards remain heuristic; keeper
redraws and subsequent charging are simulated. Already hidden blind charges
are **not** reconstructed probabilistically, so those pre-play value estimates
are especially approximate. Legality tests cover all six charging rules, but
strength claims here concern Classic rules only.

The original pre-play prototype sampled the root's keeper discard style as
though it were an opponent. This was corrected to the actual heuristic;
`preplay-v1-*` files preserve the earlier screens, and `preplay-*` contain the
corrected version. `guided8-*` used eight initial rounds; reproduce it by
explicitly appending `:8`, since the later allocator defaults to 32 rounds.

Population learning uses a paired greedy game as an action-independent return
baseline, PPO-style ratio clipping, entropy regularization, and terminal money.
It is actual policy learning, not search-label imitation. The trained outputs
are **policy scores**, not calibrated monetary advantages. Rust softmax uses
those scores with temperature 20 during collection and greedy actions during
evaluation. Each generation adds its greedy checkpoint to the opponent archive.

Hard-position collection takes up to four hard and four random positions per
game. The sets overlap in 3,823 positions; shared positions receive identical
teacher labels. Selection sees only cheap-search values, network predictions,
and the player's public information. Entire deals stay in train or validation.
The two datasets match positions, teacher samples, and training epochs, rather
than row counts: hard positions contain 31,192 candidate rows, uniform positions
29,691. Validation regret alone does not establish better play.

Both initial trainers selected their final epoch (45), so both were extended
to 120 epochs with the same starting checkpoint and settings. Hard-data
validation selected epoch 50; the uniform control still selected epoch 45.
Hard-data validation regret improved from 7.36 to 7.10 money, yet its actual
match result did not improve. The `*-long` checkpoints preserve that iteration.

Allocation eliminates roughly half the active moves after the minimum rounds,
then at twice as many rounds until two remain. Each elimination ranks the
sampled mean plus four pseudo-samples of the distilled network. The final move
uses search values alone (`PRIOR_SAMPLES=0`). Budget comparisons count terminal
evaluations of root actions; playing different moves can change later branching
and thus whole-hand totals. Timing includes overhead and is not equalized.

## Reproduce

From the repository root (set `CARGO_HOME` if your environment requires it):

```sh
cargo build --locked --release -p turbo-hearts-bot --examples
target/release/examples/arena model:128:12 defend:128:12 mixed 2000 90000
target/release/examples/arena posterior:128:12 model:128:12 mixed 300 70000
target/release/examples/arena combined:128:12 defend:128:12 defend:32:12 300 70000
target/release/examples/arena tree:512:12 defend:512:12 mixed 300 70000
target/release/examples/arena guided:128:12:assets/models/policy-moon.thp:0:32 defend:128:12 mixed 300 70000
```

`model` models behavior, `belief` only weights deals, `posterior` does both.
`preplay` searches passes and charges; `pass` and `charge` ablate either part.
`combined` adds pre-play to `posterior`. `modelcharge` and `posteriorcharge`
leave passing heuristic. `tree` adds own future decision search; `modeltree`
adds it to `posterior`. `allocate` uses successive halving without a network;
`guided` also uses the supplied network. `defend` always remains the frozen
baseline; do not use a moving `default` to reproduce older comparisons.

```sh
mkdir -p data/round2
OPENBLAS_NUM_THREADS=1 python3 train_self_play.py \
  --output data/round2/population --hands 4000 --generations 3 --seed 100000
target/release/examples/arena policy:assets/models/round2/policy-rl-3.thp:0 \
  policy:assets/models/policy-moon.thp:0 mixed 1000 90000

target/release/examples/hard_data data/round2/hard.bin data/round2/uniform.bin 2000 120000 256
OPENBLAS_NUM_THREADS=1 python3 neural_net.py data/round2/hard.bin data/round2/hard.thp \
  --initial assets/models/policy-moon.thp --epochs 45 --learning-rate 0.0003
OPENBLAS_NUM_THREADS=1 python3 neural_net.py data/round2/uniform.bin data/round2/uniform.thp \
  --initial assets/models/policy-moon.thp --epochs 45 --learning-rate 0.0003
target/release/examples/arena policy:data/round2/hard.thp:0 \
  policy:data/round2/uniform.thp:0 mixed 1000 70000
```

Checkpoints and SHA-256/training metadata are retained under
[`assets/models/round2`](../../../assets/models/round2). `hard` and `uniform`
are distilled value checkpoints; `policy-rl-*` are policy-score checkpoints.
Trajectories and label datasets are ignored generated files. Collection commands
and seeds reconstruct them. Hard-data rows use the existing 259-float format;
self-play rows use 261 floats: deal, decision, chosen flag, behavior probability,
paired terminal return /100, and 256 features. All are little-endian float32.

## Validation and remaining work

`cargo test --locked --workspace --all-targets` passed, including 75 distinct
Rust unit/integration tests, example tests, and benchmark smoke checks.
`OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training` passed both numerical
tests, and the Python scripts compiled. All seven retained checkpoints match
their recorded SHA-256 hashes. Existing unrelated server/dependency warnings
remain; there were no test failures.

Regression coverage includes policy-support legality, likelihood discrimination,
posterior normalization, history reset, deterministic replay, allocation budget
bounds, finite effective sample size, pre-play termination/card conservation,
all six charging rules, and blind-charge privacy. Python tests check PPO
gradients against finite differences, including clipping and masked actions,
and checkpoint round-trips/corruption. A 64-hand self-play smoke run exercises
the current collector, optimizer, and exporter together.

Pre-play development also exposed an existing `Cards::powerset()` size-hint
bug: exhausted iterators reported `usize::MAX`, causing collection to overflow.
The fix includes a regression for exact remaining sizes and empty powersets.

The strongest next research target is a defender-aware continuation model for
moon planning: the mixed-versus-search gap shows why evaluating a moon only
against point-avoidance policies is optimistic. Richer observation trees,
longer population training, and hard-example replay mixed with ordinary data
remain plausible, but this round does not establish their effectiveness.
