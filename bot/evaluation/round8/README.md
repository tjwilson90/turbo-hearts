# Model capacity, history, and payoff training

Richer inputs reduced later human-move prediction loss by about 12.6%, but
none of the new models showed a consistent playing-strength gain at equal
decision time. The selected smaller network regressed by 6.01 money/hand
against the human/search hybrid in fresh timed confirmation. Doubling the
rich model's width did not improve later loss; longer training improved the
smaller models' later loss by less than 0.0005.
No new learned model is enabled by default.

This round implements and tests larger human move predictors, richer card and
public-history inputs, persistent plans within simulated continuations, and
payoff training from responsive action comparisons. The frozen baseline is
`round7`, the successful-human fallback with supported-heart preservation.
Checkpoints, trainers, and research profiles remain available for reproduction.

## Data and what these experiments measure

The chronological split and successful-human cohort are unchanged from
[round seven](../round7/README.md): 1,189 training games, 255 validation games,
255 later games, and four recent diagnostic games. Five humans were selected
using training outcomes alone. Their nonforced decisions provide 86,820
training, 20,436 validation, and 19,464 later examples. Whole games retain their
split. The later block was examined in earlier rounds and is **not a pristine
holdout**. Databases, identities, chats, and full exports remain outside the
checked-in results.

Policy likelihood measures prediction of recorded human choices. Match scores
measure actual money in simulations with responsive opponents, including
charged moons, JD, and TC. Neither measures a causal improvement against live
humans. Historical matches preserve recorded hands, passes, charges, rules,
and public player types; all subsequent actions are simulated afresh.
Fresh-deal matches use heuristic passes/charges and mark every seat as human
so the human opponent models are exercised. This difference matters when
comparing historical and fresh results. Fresh performance runs use Classic
rules; the tests cover legal completion under all six charging rules, not
equal playing strength under each rule.

## Capacity and longer training

All new policies use a single ReLU hidden layer, a conditional action-feature
head, and optional per-card logits. Adam uses batch size 512, initial rate
0.003, L2 0.0001, and a fixed seed. Learning rate halves after five and ten
epochs without improvement; training stops after fifteen. The selected epoch
minimizes validation loss averaged over full and masked private-pass inputs.
Later losses are evaluated only after checkpoint selection.

| Policy | Active parameters | Best epoch / epochs run | Validation NLL | Later NLL |
| --- | ---: | ---: | ---: | ---: |
| Previous direct model | 432 | 35 / 60 | 1.16183 | 1.16772 |
| Old features, 32 hidden units | 1,400 | 72 / 80 | 1.12661 | 1.13458 |
| Card-aware output, 32 units | 3,248 | 76 / 80 | 1.07256 | 1.08018 |
| Rich input, 32 units | 14,288 | 30 / 45 | 1.02229 | 1.02062 |
| Rich input, 64 units | 28,496 | 11 / 26 | 1.02308 | 1.02109 |

Lower NLL is better. These are full-private-input losses. Masked later losses
are 1.01733 and 1.01846 for the rich 32/64 models. The capacity control keeps
only the old 18 context and 24 action features active (3,248 parameters are
stored, but 1,848 output parameters are fixed to zero). The card-aware model
also adds suit indicators and per-card logits, so its gain cannot be attributed
to capacity alone.

The richer 32-unit model reduces later NLL by about 12.6% relative to the old
direct model. Doubling it to 64 units lowers training NLL from 0.891 to 0.849
but does not improve validation or later NLL. That is evidence of overfitting
at this data size, not a reason to keep training that model longer. Rich inputs
combine several additions, so these results do not isolate history alone.
Private-pass inputs were slightly worse than masked inputs on the later set;
their individual benefit is unproven.

Extending the two small-context fits to at most 200 epochs selected epoch 88
and stopped at 103 in both cases. Later NLL moved from 1.13458 to 1.13414 for
the capacity control and from 1.08018 to 1.08017 for the card-aware model.
The extra training has essentially converged. These `*-long` checkpoints are
separate training ablations; matches keep the original 80-epoch checkpoints.

The rich observation contains 363 floats: the old 18 context values; three
52-card channels for own remaining cards, unseen cards, and the current trick;
three 52-card channels for the observer's own passes to relative opponents;
eight public behavior statistics per relative seat; and a private-pass flag.
Public statistics count early safe high/low follows, heart/nine sloughs, play
count, and last rank. They do not assume knowledge of another player's legal
alternatives. All events pass through normal redaction. Simulated opponents
receive public history but not the root player's private passes; training
masks private channels half the time to cover that case.

## Coherent simulated plans

`coherent` samples one of six existing skilled-policy components once per
sampled continuation and retains it across subsequent moves. The same choice
is shared across candidate root actions. It starts with the contextual fitted
weights and updates them with observed public actions on each sampled hand,
using a tempered likelihood and 25% uniform-action noise. Existing confident
duck/shooter inferences take precedence. Shooting components abandon moons
once another player takes a moon point. The plan is reconsidered on the next
real decision. This is a heuristic latent-policy posterior, not an inferred
human intention label.

## Payoff training

`human_payoff_data` samples positions without conditioning on outcomes, using
an ID/event modulus of 53. Eligible positions are successful humans' decisions
with more than one distinct legal choice and fewer than 40 cards played.
It evaluates every legal action, with two seeds each under `actionclaim` and
`round7:16:8` responsive continuations. Actual hidden hands are used only to
generate counterfactual targets; model inputs contain the actor's observations.
Recorded future actions are never forced after a changed move.

The data contains 1,414 training, 379 validation, and 329 later positions.
Targets are centered money advantages divided by 100. The payoff model freezes
the 32-unit rich policy's hidden features and fits an output head with centered
Huber loss. It selected epoch 3 and stopped at 18. Validation/later Huber losses
are 0.0866/0.1159. This is a small, noisy pilot, not broad self-play training.

On later target positions, its greedy choices have +4.34 modeled advantage
over the equal-action mean; recorded human choices have +11.83. The paired
model-minus-human difference is -7.48 [-15.30, 0.33], clustered by game.
These counterfactual estimates do not establish what a human would have
earned after a different move. See [payoff-choice-metrics.json](payoff-choice-metrics.json).

`payoff` blends centered search and learned advantages before applying the
existing stopper preference. Weight 0.5 gives each half the influence. The
blend applies only outside exact endgames and with another public human seat.
Raw `evaluate` values remain search-only. A zero-output control at weight
0.25 changes the scale relative to the stopper margin; its 40-game differences
are +0.17 [-0.07, 0.41] and -0.10 [-0.47, 0.26] against the direct and hybrid
responders. The larger experimental effects are not solely that rescaling.

## Initial match screening

First 40 eligible validation games, 628 seat replays per version. All use a
128-sample ceiling and exact 12-card endgames. The payoff-0.5 screens also
have a 1,000 ms soft cap; other screens are uncapped. Intervals are paired 95% normal
approximations clustered by original game. These overlapping development
screens were used to select the larger checks; they are not confirmation.

| Candidate | Direct `actionclaim` gain | `actionhybrid` gain |
| --- | ---: | ---: |
| Persistent component | -1.62 [-9.80, 6.56] | +2.81 [-7.26, 12.87] |
| Old-feature 32-unit policy | +6.79 [-3.70, 17.28] | +3.87 [-7.70, 15.45] |
| Card-aware 32-unit policy | +4.55 [-8.36, 17.46] | +6.31 [-4.04, 16.66] |
| Rich 32-unit policy | -7.85 [-20.53, 4.83] | +0.28 [-10.78, 11.33] |
| Rich continuations at 50% | -4.28 [-18.30, 9.73] | -0.74 [-12.71, 11.22] |
| Payoff weight 0.25 | +0.05 [-6.89, 7.00] | +5.85 [-1.89, 13.60] |
| Payoff weight 0.5 | -2.94 [-12.67, 6.79] | +10.65 [0.99, 20.32] |

An independent responder mixes the rich 32-unit policy with frozen round7
search and provable claims. Against it, the card-aware policy gives -3.08
[-20.57, 14.42], coherent plans +2.24 [-9.97, 14.45], and payoff weight 0.25
+0.06 [-7.72, 7.84]. This responder uses a different learned architecture,
although it shares the training cohort. It is not an independent human sample.

The `neuralmix` screen substitutes neural continuations at half of eligible
human seats per sample. Remaining seats use the earlier inferred-style and
emerging-shooter fallback, not the fitted skilled policy. It therefore does
not isolate a 50/50 blend of the two learned policies.

The legacy-feature policy, card-aware policy, and payoff weight 0.5 were
selected for 100-game checks against all three responders before seeing those
expanded results. [PLAN.md](PLAN.md) records the selection and seed boundary.

Expanded validation: 100 eligible games, 1,564 seat replays per version.
These contain the initial 40 games and are still development results.

| Candidate | Direct human policy | Human/search hybrid | Rich-neural/search hybrid |
| --- | ---: | ---: | ---: |
| Capacity32 | -1.27 [-8.70, 6.16] | +8.56 [1.22, 15.90] | +1.84 [-5.68, 9.36] |
| Card-aware32 | -4.19 [-12.33, 3.95] | +4.10 [-2.22, 10.42] | +6.11 [-3.22, 15.44] |
| Payoff 0.5 | -9.51 [-15.16, -3.85] | +0.64 [-5.84, 7.12] | +3.66 [-3.06, 10.39] |

The original-feature capacity32 checkpoint was selected for confirmation.
Its equally weighted mean across these responders is +3.04, versus +2.01
for card-aware32 and -1.74 for payoff. Those means are descriptive; the
responders share deals and are not independent samples for a pooled interval.
The earlier payoff hybrid gain largely disappeared, and its direct-responder
regression rules it out for promotion. See `extended-*.json`.

## Confirmation of the capacity control

The original 80-epoch capacity32 checkpoint was frozen before these runs.
Each fresh comparison uses 500 deals starting at seed 1600000, with four seat
replays per deal (2,000 hands per version). The three responders share the
same 500 original deals. Intervals are paired and clustered by deal; they
must not be pooled as 1,500 independent deals.

| Responder | Capacity32 money | Round7 money | Gain, 95% interval |
| --- | ---: | ---: | ---: |
| Direct human policy | 52.33 | 49.77 | +2.56 [-2.26, 7.39] |
| Human/search hybrid | 24.51 | 21.68 | +2.84 [-3.14, 8.81] |
| Rich-neural/search hybrid | 15.78 | 13.08 | +2.70 [-3.12, 8.52] |

All three point estimates favor the model, but none rules out no improvement.
Fixed samples also give the network more computation: the direct-responder
run averages 75.26 versus 22.83 ms/hand under concurrent load, before the
last unused-input/output optimization. See `fresh-*.json`.

The first 100 eligible games from the **reused later historical block** give
1,600 seat replays per version. They preserve the successful humans' actual
post-pass hands and charging decisions:

| Responder | Capacity32 money | Round7 money | Gain, game-clustered 95% interval |
| --- | ---: | ---: | ---: |
| Direct human policy | 78.46 | 72.23 | +6.23 [-1.13, 13.58] |
| Human/search hybrid | 35.21 | 33.15 | +2.06 [-7.04, 11.16] |
| Rich-neural/search hybrid | 26.09 | 24.23 | +1.87 [-7.71, 11.44] |

These also favor the model descriptively without establishing a clear gain.
Against the neural hybrid, opponent moons fall from 209 to 181, but the bot's
own moons also fall from 149 to 137. Money captures both effects; avoiding
more opponent moons is not by itself evidence of a net improvement.
See `later-*.json`.

For the equal-time comparison, both players use an 8,192-sample ceiling,
exact 12-card endgames, and a 5 ms soft limit per decision. The optimized
capacity checkpoint is unchanged. Runs are sequential after training and
fixed-sample comparisons finish. This is a comparison at the same nominal
deadline; actual measured time and completed work are reported too.

The 100-deal development checks at seed 1510000 gave -12.66 [-25.01, -0.32]
against the direct responder, -8.45 [-24.83, 7.94] against the action hybrid,
and -3.53 [-17.22, 10.16] against the neural hybrid. Both players averaged
42.6–43.7 ms/hand. The network completed roughly 9,500–10,200 terminal
evaluations per hand versus 20,000–20,400 for round7. These small development
checks motivated testing extra search separately; they are not the final
equal-time confirmation. See `time-development-*.json`.

Fresh equal-time confirmation uses 500 deals at seed 1610000, again with
2,000 seat replays per version and paired deal-clustered intervals:

| Responder | Neural minus round7 money, 95% interval | Neural / round7 ms per hand | Neural / round7 terminal evaluations per hand |
| --- | ---: | ---: | ---: |
| Direct human policy | -1.43 [-6.64, 3.78] | 42.88 / 42.82 | 10,428 / 20,391 |
| Human/search hybrid | -6.01 [-11.21, -0.82] | 42.43 / 42.18 | 10,079 / 19,452 |
| Rich-neural/search hybrid | +3.25 [-2.82, 9.32] | 42.73 / 42.80 | 10,016 / 19,863 |

The new network does not earn promotion. Its better prediction loss does not
compensate consistently for the lost search work, and it regresses against
the action/search hybrid at matched time. The positive independent-hybrid
point estimate remains uncertain. No new learned policy or payoff head is
enabled by default. See `time-fresh-*.json`.

The equal-time results also motivated a separate search-budget control:
`round7:8192:12:5` versus today's `round7` at 128 samples. It uses 500 fresh
deals at seed 1620000 per responder, sequentially after the model comparisons.
This tests spending more computation on the existing policy; it is **not**
a comparison at equal computation. The model weights and continuation logic
are identical on both sides.

| Responder | Extra-search gain, 95% interval | Extra search / current ms per hand |
| --- | ---: | ---: |
| Direct human policy | +3.97 [-0.80, 8.74] | 42.62 / 22.35 |
| Human/search hybrid | +3.87 [-2.00, 9.74] | 42.30 / 22.32 |
| Rich-neural/search hybrid | +2.79 [-3.17, 8.74] | 42.82 / 22.08 |

More search is a plausible modest improvement, but each interval includes
zero and it costs roughly twice the computation. The existing 128-sample
production budget remains unchanged. This control is available through the
documented profile for further evaluation. See `search-budget-*.json`.

## Original and Dan diagnostics

The replay runs repeat each decision under 100 search seeds. They are familiar
diagnostic positions, not independent confirmation. `mean_values` in the
payoff replay files intentionally remains raw search output; `choices`
includes the learned blend and stopper preference.

| Diagnostic choice | Round7 | Capacity32 | Payoff weight 0.5 |
| --- | ---: | ---: | ---: |
| Original A♥ slough | 1/100 | 2/100 | 0/100 |
| Dan A♥ slough | 3/100 | 4/100 | 0/100 |
| K♣ before charged 10♣ | 72/100 | 75/100 | 69/100 |
| Charged 10♣ in that position | 4/100 | 10/100 | 2/100 |
| Human's 10♥ before Q♠ | 0/100 | 3/100 | 0/100 |

Avoiding the ace alone does not prove better defense. In the Dan position,
the payoff blend chooses Q♦ 99/100 times, whereas round7 commonly distributes
a small heart. In the heart-before-queen case, payoff chooses 8♦ 100/100;
round7 chooses Q♠ 100/100. These are sizable strategic changes from a small
training set.

Continuing the Dan hand responsively from event 145, with each bot using the
named candidate and the recorded human seat using the named responder:

| Human responder | Round7: West money / East moons | Capacity32 | Payoff 0.5 |
| --- | ---: | ---: | ---: |
| Direct human policy | +14.37 / 14 | +3.12 / 15 | -13.40 / 38 |
| Dedicated shooter | -71.61 / 83 | -72.89 / 84 | -31.31 / 64 |
| Round7 search | -73.41 / 80 | -68.36 / 77 | -94.10 / 97 |

Each cell summarizes 100 continuations on the same actual hands. This changes
all subsequent bot decisions, so differences cannot be attributed solely to
the first card. The payoff model helps against the dedicated shooting policy
but harms the other two continuations. Neither new model resolves the Dan
game's remaining weakness. See `replay-*.json` and `dan-*.json`.

To isolate that first discard, force Q♦, 8♥, or 9♦ and keep **all subsequent
bot play on round7**. Each move uses the same 100 seeds within a responder:

| Human responder | Force Q♦: West money / East moons | Force 8♥ | Force 9♦ |
| --- | ---: | ---: | ---: |
| Direct human policy | +13.98 / 14 | +10.62 / 16 | +7.92 / 19 |
| Dedicated shooter | -76.37 / 88 | -75.91 / 85 | -83.23 / 91 |
| Round7 search | -69.81 / 77 | -64.38 / 75 | -68.23 / 77 |

The forced-Q♦ control does not reproduce the payoff model's 38 direct-policy
moons. Its first discard alone therefore does not explain the poor overall
continuation; later play matters substantially. An early heart is helpful
against some of these responders but is not uniformly the best discard in
this particular nine-extended club trick. The known-hand diagnostic does not
establish a general heart/nine rule or overturn their usefulness elsewhere.
See `dan-forced-*.json`.

## Runtime and reproduction

THH1 checkpoints contain dimensions, a role tag, and float32 weights. Rust
loads them without NumPy or another inference runtime; malformed dimensions,
unknown roles, truncated files, and nonfinite weights are rejected. Policy
and monetary heads cannot be interchanged. Input-major first-layer weights,
sparse input skipping, and forced-move shortcuts reduced rich-policy timing
from 1,303 to 117 ms/hand on the same 20-deal development case, compared with
23 ms/hand for round7. These timings were collected under concurrent load.
The optimization preserved scores, moons, and search counts exactly; forced
moves still consume the same random draw. See `timing-*.json`.

Before equal-time confirmation, the small-context models also stopped building
unused rich channels, and zero output columns in the capacity control were
skipped. The same 20-deal check preserved money, moons, win shares, and search
counts exactly. Concurrent capacity timing was 81.85 then 74.92 ms/hand, with
round7 at 24.60 then 24.47; this is not a controlled speed benchmark. NumPy
references cover both the rich and pruned capacity networks.

The time cap is soft: setup and at least one complete candidate batch run.
Under an explicit time cap, inference setup is bounded at the default
128-sample budget (256 inference worlds), even if the rollout ceiling is much
higher. Normal 128-sample production and uncapped fixed-sample experiments are
unchanged by this fix. Wall-clock-capped matches are scheduling-dependent.

Generate the ranked input using the round-seven reproduction commands first.
Generated datasets stay under ignored `data/`.

```sh
cargo build --locked --release -p turbo-hearts-bot --examples
target/release/examples/human_neural_data data/round7/ranked.jsonl data/round8/human.bin
OPENBLAS_NUM_THREADS=1 python3 train_human_neural.py data/round8/human.bin assets/models/human-capacity32.thh --inputs 18 --hidden 32 --legacy-actions
OPENBLAS_NUM_THREADS=1 python3 train_human_neural.py data/round8/human.bin assets/models/human-basic32.thh --inputs 18 --hidden 32
OPENBLAS_NUM_THREADS=1 python3 train_human_neural.py data/round8/human.bin assets/models/human-history32.thh
OPENBLAS_NUM_THREADS=1 python3 train_human_neural.py data/round8/human.bin assets/models/human-history64.thh --hidden 64
# Longer training ablation: repeat capacity32/basic32 with --epochs 200 and
# output names human-capacity32-long.thh / human-basic32-long.thh.

target/release/examples/human_payoff_data data/round7/ranked.jsonl data/round8/payoff.bin data/round8/targets.bin 53 2
OPENBLAS_NUM_THREADS=1 python3 train_human_neural.py data/round8/payoff.bin assets/models/human-payoff32.thh --targets data/round8/targets.bin --initialize assets/models/human-history32.thh

target/release/examples/historical_arena data/round7/ranked.jsonl neural:assets/models/human-capacity32.thh round7 actionclaim 100 1 successful
# Repeat with basic32; payoff:assets/models/human-payoff32.thh:128:12:1000:0.5;
# and actionhybrid / neuralhybrid:assets/models/human-history32.thh responders.

target/release/examples/arena neural:assets/models/human-capacity32.thh round7 actionclaim 500 1600000
target/release/examples/historical_arena data/round7/ranked.jsonl neural:assets/models/human-capacity32.thh round7 actionclaim 100 2 successful
# Repeat both with actionhybrid and neuralhybrid:assets/models/human-history32.thh.

target/release/examples/arena neural:assets/models/human-capacity32.thh:8192:12:5 round7:8192:12:5 actionclaim 100 1510000
target/release/examples/arena neural:assets/models/human-capacity32.thh:8192:12:5 round7:8192:12:5 actionclaim 500 1610000
# Run wall-clock comparisons sequentially after other compute has stopped.
# Repeat with the same two hybrid responders.
target/release/examples/arena round7:8192:12:5 round7 actionclaim 500 1620000
# Extra-search control: repeat with the same two hybrid responders.

target/release/examples/replay_decision bot/tests/fixtures/ace-slough.json 37 neural:assets/models/human-capacity32.thh 100
target/release/examples/replay_decision bot/tests/fixtures/dan-game.json 145 neural:assets/models/human-capacity32.thh 100
target/release/examples/replay_decision bot/tests/fixtures/human-king-before-charged-ten.json 85 neural:assets/models/human-capacity32.thh 100
target/release/examples/replay_decision bot/tests/fixtures/human-heart-before-queen.json 30 neural:assets/models/human-capacity32.thh 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 neural:assets/models/human-capacity32.thh actionclaim auto 100
# For the payoff diagnostics use payoff:assets/models/human-payoff32.thh:128:12:none:0.5.
# Branch responders additionally include gottatry and round7.
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 round7 actionclaim QD 100
# Isolate the first discard: repeat with 8H and 9D, and the other responders.

cargo test --locked --workspace --all-targets
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy test_historical_data test_human_neural
```

The new profiles freeze round7; older profiles and hybrids keep their earlier
baselines. The research preplay planner does not use the new neural memory or
latent plans, and none of these profiles enables it. Passing and charging were
not retrained. No deployment is part of this experiment.

The full Rust workspace/all-targets checks and all ten Python tests pass.
Checks cover NumPy/Rust coefficient agreement, classification and centered
payoff gradients, private-pass redaction, unchanged all-bot values and choices,
model roles and malformed files, bounded inference setup under large timed
ceilings, and legal completion across every charging rule. `git diff --check`
also passes.
