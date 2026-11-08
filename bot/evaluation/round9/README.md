# Engine throughput and iterative passing/charging

Enabled: the semantics-preserving engine optimizations and pass-two for a lone
bot against three public human opponents in Classic, on left/right/across
rounds. Fixed-search computation fell by 21.5%; equal-deadline search completed
33.1% more evaluations. Learned passing improved two fresh lone-bot match
sets, but harmed tables where several bots changed together, so the default
is explicitly restricted. Charging and keeper passing retain their heuristics;
the 128-sample/12-card play settings are unchanged.

This round measures simulation optimizations and trains separate passing and
charging policies from responsive terminal-money comparisons. The frozen
baseline is `round7`: 128 shared samples, 12-card exact endgames, the fitted
successful-human continuation model, and heuristic passing/charging.
[PLAN.md](PLAN.md) records development and confirmation seed boundaries.

## Engine changes

- Cache the conditional suit-allocation distributions once per decision and
  reuse them for hidden-hand sampling. Shuffle cards in stack buffers.
- Replace eleven single-rank propagation steps with four parallel-prefix
  steps when collapsing equivalent legal cards.
- Test packed trick membership directly, without constructing a card set.
- Use the same play and void transitions directly in simulation inner loops,
  avoiding general event dispatch.
- Reuse the heuristic's legal-card computation, and skip the skilled mixture
  calculation for forced plays while preserving both original random draws.

The first 100-deal paired replay (400 hands/version) against `actionhybrid`
fell from 21.39 to 17.16 ms/hand, about 20% less computation or 25% more
throughput. Money, moons, win shares, and completed search work were identical.
The additional six-rule replay checks also retained these exact outputs.
These are semantics-preserving changes; they do not themselves prescribe a
new strategic policy. See `engine-before.json`, `engine-after-first.json`, and
`engine-parity-all-rules.json`. Timings are machine-specific.

Final sequential measurements at seed 1906000, after training and the primary
matches finished, give 22.40 versus 17.58 ms/hand at the same 128 samples:
**21.5% less time**, with identical scores, win shares, moons, and search work.
At the same 5 ms soft decision limit and 8,192-sample ceiling, two repetitions
average 20,213 versus 26,906 terminal evaluations per hand: **33.1% more work**.
Measured whole-hand computation is 43.05 versus 43.07 ms. Runs are sequential
in old/new/new/old/old/new order across fixed/timed measurements; each uses the
same 100 deals. Repetitions quantify timing variation, not additional
independent match evidence. Timed scores differ as search stops at different
samples; no strength claim is based on these small timed comparisons.
See `engine-summary.json` and `engine-*-fixed/timed*.json` for executable hashes
and detailed results. Production retains 128 samples and 12-card endgames.

Budget development used 100 different deals at 1902000 against the human/search
hybrid. Increasing samples to 160 or 192 gave -2.08 [-12.74, 8.58] and
-5.04 [-17.88, 7.81] money/hand respectively. Extending exact endgames to
16 cards gave -3.26 [-10.87, 4.34], while costing 56.97 versus 17.93 ms/hand.
These small checks do not support changing the search settings. Fixed-sample
development ran alongside other jobs; it is not the final timing benchmark.

## Learning procedure

Each candidate pass/charge is applied to a cloned legal game and played to
completion with responsive opponents. Targets are actual zero-sum terminal
money, including charges, moons, JD and TC, averaged over two paired future
random streams and centered within each decision. Hidden hands are used only
to generate targets. Each acting policy receives normal redacted events.
The teacher never chooses its play plan after seeing a counterfactual's hidden
cards, and never forces a recorded future sequence after changing an action.

Passing compares sixteen candidate triples: the current policy, heuristic,
shooting and duck policies, nearby one-card substitutions, and random triples.
Inference scores all 286 legal triples, so full matches are essential to test
generalization beyond the sampled candidates. Charging enumerates all legal
subsets, including no charge. Prior declines, repeated charging, blind rules,
and keeper's two charging stages are represented. In keeper, a first charge
can prevent the redraw; if everybody declines, discards are shuffled into a
common pool and may return to their original owner.

TPQ1 heads use 256 inputs and 32 ReLU hidden units. Inputs encode the actor's
hand, retained cards, candidate action, four pass directions, rule flags,
keeper stage, public relative charges/turn/declines, and suit length/control
summaries for every suit. Neither human identity nor other players' private
hands, passes, or unrevealed charge identities enter the features.
Heads minimize decision-centered squared monetary error; this retains rare
large charged outcomes in the expected-money objective. All decisions and
actions from one deal share a split using a mixed hash of its ID. The mixed
hash avoids accidentally holding out only particular seats or directions.
Validation target regret selects an epoch, followed by independent match
evaluation. Model files retain seeds, hashes, fitting metrics, and history.

Generation one uses 12,000 deals at 1800000–1811999. It provides 10,964 pass
decisions and 12,207 charge decisions, with 209,132 candidate rows. The learner
uses `round7:8:8`; opponents are independently drawn from `actionclaim`,
`skilledclaim`, `round7:8:8`, heuristic, and shooting policies. The retained
passing/charging epochs are 12/30. Validation target gains over parent choices
are +9.44/+3.03 money, but these are not full-match estimates.

Generation two uses 6,000 new deals at 1812000–1817999. The combined first
generation becomes the learner and an archived population member, retaining
the five original opponents. Both heads initialize from generation one and
fit pooled data from all 18,000 deals (310,096 candidate rows). Both select
epoch 2 and stop at 23. Validation target gains over the respective recorded
parents are +5.33/+4.82. The parent and state distributions differ between
generations, so these gains and regrets are not direct strength comparisons.

The cheap 8-sample teacher permits broader coverage but is weaker than match
play. These are two iterations of population policy improvement, not converged
self-play or proof of optimal passing/charging. In particular, the fitted
human responders use heuristic preplay decisions; they do not reproduce
humans' strategic passing or charging.

## Development and match interpretation

Matches keep root card-play search at 128 samples/12-card exact endgames.
Original deals are replayed in all four root seats. Money differences and
95% normal intervals are paired and clustered by original deal; four seats
are not four independent observations. Opponents respond to every changed
pass, charge and play. Fresh matches mark all seats publicly human to exercise
the fitted continuation logic, including when an evaluation responder is a
search policy. These are proxy-opponent simulations, not live human trials.

The initial generation-one screens use 100 deals at 1901000, with 25 per
direction. All three pass/charge ablations share these development deals:

| Candidate | Human policy | Human/search hybrid | Round7 search (32 samples) |
| --- | ---: | ---: | ---: |
| Pass 1 | +30.77 [12.34, 49.19] | +4.62 [-15.74, 24.97] | -8.18 [-24.39, 8.04] |
| Charge 1 | +7.00 [-4.67, 18.66] | -4.79 [-16.68, 7.10] | -8.94 [-19.03, 1.15] |
| Both 1 | +51.42 [30.12, 72.72] | +3.30 [-17.79, 24.38] | -25.45 [-44.09, -6.80] |

The combined policy increases both successful and failed moon attempts. Its
large gain against the direct fitted policy does not transfer to search, so
that result alone cannot justify promotion. Charging one is also much more
aggressive than the old heuristic, including in short suits. Conditional
charge frequencies on generation-one validation states are recorded in
`charge1-length-diagnostic.json`; these describe learned behavior, not evidence
that every such charge is sound.

Keeper pass-one estimates are negative against all three responders:
-10.97/-16.36/-6.23 money. Subsequent development retains heuristic keeper
passing and tests the learned head on left/right/across (mask 7). The model
still conditions on individual direction. A configurable mask permits
independent promotion per round instead of imposing one learned policy on
all four. See `development-pass1-directed-*` and `screen2-*` for subsequent
development, including passing/charging ablations of the second generation.

The second-generation directed-pass screen gives +25.23/10.47/5.22 against
the direct/hybrid/search responders. Charging alone gives +3.98/-1.26/-10.73;
the last interval [-20.63, -0.83] excludes zero. Combining the heads gives
+50.11/+5.98/-7.97. Thus iteration improved passing's search transfer but did
not make the charging head reliable. A lower-variance charging control fits
480 additive contextual coefficients with ridge regression, selecting penalty
0.001 on the same whole-deal validation split. Its export uses eight active
ReLU units in TPQ1 and matches the additive values up to floating-point error.
Its 100-deal gains are +14.31 [2.41, 26.20], -2.03 [-15.07, 11.00], and
-8.76 [-18.24, 0.72]. Reducing model complexity does not resolve transfer to
the stronger responders. None of these charging heads earns promotion.

The larger directed-pass development uses 300 new deals at 1903000:

| Passing head, keeper unchanged | Human policy | Human/search hybrid | Round7 search (32) |
| --- | ---: | ---: | ---: |
| First iteration | +17.95 [9.74, 26.17] | +12.62 [3.26, 21.98] | +2.32 [-5.80, 10.43] |
| Second iteration | +18.42 [10.44, 26.40] | +19.93 [11.43, 28.42] | +4.84 [-2.67, 12.35] |

Pass two, margin zero, mask 7 was frozen before fresh confirmation. This
retains the existing charging and keeper passing policies. The 18,000 training
deals, initial screens, and these development comparisons do not enter the
fresh confirmation samples.

Choice diagnostics use 1,000 other development deals at 1905000, evaluating
both policies on identical parent-policy states. Across left/right/across,
pass two passes mean ranks 7.04–7.37 versus 9.89–9.95 for the heuristic. It
retains about 1.20 cards in uninterrupted top-of-suit sequences versus
0.76–0.78, while creating fewer voids. It passes 0.18–0.24 aces/kings per
decision versus 0.81–0.83. Pass one retains even more control; the second
iteration moves partway back toward the heuristic. These aggregates describe
the learned strategy, not a recommendation to always retain high cards.
`preplay_choices` reproduces the full legal-action choice comparison; keeper
diagnostics are conditional on every player declining the first charge.

## Fresh passing confirmation

All four Classic comparisons use the frozen pass-two/mask-7 policy on the
same 500 unseen deals at 2000000: 2,000 seat replays per version and responder.
Charging, keeper passing, and card-play settings are unchanged.

| Responder | Learned passing money | Parent money | Paired gain, 95% interval |
| --- | ---: | ---: | ---: |
| Fitted successful-human policy | 72.91 | 57.66 | +15.25 [9.39, 21.11] |
| Human/search hybrid | 31.43 | 20.75 | +10.67 [3.68, 17.66] |
| Rich-neural/search hybrid | 25.62 | 20.89 | +4.73 [-2.44, 11.90] |
| Round7 search, 32 samples | 7.08 | 8.30 | -1.22 [-8.54, 6.10] |

The first two gains confirm transfer to these human-oriented responders.
The other two results are inconclusive; this is not evidence of universal
improvement or proof of no search regression. The independent neural hybrid
uses a different learned architecture but shares the historical human cohort.
All responders share deals, so do not pool their intervals as independent runs.

Against the human/search hybrid, completed bot moons increase from 132 to
281, failed queen-plus-8–12-heart attempts increase from 138 to 222, and
opponent moons move from 138 to 137. The money gain includes the extra failed
attempts. Against search, moons also increase (106 to 210) but money does not
improve. Retaining control makes a more capable offensive hand; its value
depends on how effectively the opponents defend it.

Per-direction gains against the direct human policy are +6.77/+28.17/+26.06
for left/right/across; against its search hybrid they are +19.65/+20.00/+3.03.
Keeper is exactly unchanged. Each direction has only 125 deals. Left/across
intervals for the direct/hybrid responder respectively include zero; see the
JSON files for all intervals rather than treating every round as proven.
The learned passing inference averages roughly 0.067 ms per whole hand,
including unchanged keeper hands. Subsequent card play takes about 1.4–1.7 ms
more per hand at identical sample counts because the new hands retain more
play options; the engine savings exceed this increase on these measurements.

The other-rule audit uses 200 unseen deals at 2001000 per ruleset against
`actionhybrid`. Gains are +3.54 [-6.72, 13.79] for Blind, +2.48 [-8.05, 13.02]
for Bridge, +5.93 [-4.33, 16.20] for BlindBridge, +3.39 [-7.54, 14.31] for Chain,
and +4.14 [-6.99, 15.26] for BlindChain. All point estimates are positive,
but none establishes a gain. Those rules also share deals and must not be
pooled as independent evidence. Default promotion beyond Classic would need
stronger evidence.

## Mixed-table transfer

The one-player arena does not establish what happens when every bot changes
at once. A separate check uses accurate public human/bot roles and switches
all bot seats together, keeping the human responders fixed. It replays every
placement of one human (four placements) or two humans (six) on 300 new deals
at 2002000. Payoff is mean money per bot, paired and clustered by deal.

| Human seats | Human responder | Unrestricted pass-two gain per bot, 95% interval |
| --- | --- | ---: |
| One | Direct policy | -5.53 [-8.11, -2.94] |
| One | Human/search hybrid | -3.04 [-5.67, -0.42] |
| Two | Direct policy | -4.40 [-8.00, -0.79] |
| Two | Human/search hybrid | -7.08 [-10.87, -3.29] |

This rules out globally enabling the unrestricted passing model despite its
positive primary confirmation. Changing other bots' passing changes the
learner's incoming cards too. A plausible failure is that several bots retain
control while receiving fewer of the high cards formerly passed by heuristic
bots, undermining the learner's anticipated offensive hand. The experiment
establishes the payoff regression; that mechanism is a hypothesis.

A subsequent development variant applies learned passing only when the
destination is publicly human, preserving heuristic passes between bots.
It uses the same frozen checkpoint and direction mask; only the recipient
condition changes. The independent lone-bot/three-human check also verifies
the original positive result with accurate public roles. Neither adjustment
uses hidden player information or user identities.

The recipient-only development check also fails to generalize. With one human,
it gives -4.87 [-8.61, -1.13] against the direct responder and -5.35
[-9.50, -1.20] against the hybrid. With two humans, the results are +3.55
[-2.41, 9.51] and -3.80 [-9.44, 1.84]. This variant remains opt-in research.

The independent lone-bot check uses 300 fresh deals at 2003000 with three
actual public human seats, four placements per deal, and the frozen pass-two
checkpoint. The recipient restriction is automatically satisfied in this
setting, so the policy is identical to the primary confirmation candidate.
Gains are **+26.72 [17.71, 35.73]** against the direct human policy and
**+8.71 [0.36, 17.06]** against its search hybrid. These support a limited default
promotion in conjunction with the earlier 500-deal positive confirmations.

`NeuralNetworkBot::new/with_seed` and the arena's `default` now load the bundled
passing checkpoint once, retaining heuristic keeper passing and charging.
The learned head is used only when rules are Classic and all three other
seats are publicly human. Mixed tables, all-bot games and other rules preserve
old passing behavior and random streams. `with_config` and the `round7` profile
remain frozen; callers configuring production manually can append
`with_production_preplay()`. Tests check the exact rule, direction, public-role
scope and unchanged charging, as well as matching the confirmed explicit
profile. No larger play model, learned charging head, or deeper search setting
is enabled by this round.

## Reproduction and validation

The generator's population depends on its shard seed, so reproduce generation
one as three shards of 6000/3000/3000 at 1800000/1806000/1809000. Generation two
uses two 3000-deal shards at 1812000/1815000. Concatenate complete shards in
that order; include generation one before generation two when fitting the
second iteration. Raw training data and archived executables are ignored in
`data/round9`; checkpoint and result JSON files contain hashes and settings.

```sh
cargo build --release -p turbo-hearts-bot --example preplay_data --example arena
target/release/examples/preplay_data data/round9/generation1.bin 6000 1800000 round7:8:8 2 16
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py data/round9/generation1-all.bin assets/models/preplay-pass1.tpq pass
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py data/round9/generation1-all.bin assets/models/preplay-charge1.tpq charge
target/release/examples/preplay_data data/round9/generation2-0.bin 3000 1812000 prelearn:assets/models/preplay-pass1.tpq:assets/models/preplay-charge1.tpq:8:8 2 16 prelearn:assets/models/preplay-pass1.tpq:assets/models/preplay-charge1.tpq:8:8
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py data/round9/generation2-all.bin assets/models/preplay-pass2.tpq pass --initial assets/models/preplay-pass1.tpq
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py data/round9/generation2-all.bin assets/models/preplay-charge2.tpq charge --initial assets/models/preplay-charge1.tpq
OPENBLAS_NUM_THREADS=1 python3 train_preplay_additive.py data/round9/generation2-all.bin assets/models/preplay-charge-additive.tpq
target/release/examples/arena prelearn:assets/models/preplay-pass1.tpq:none:128:12:none:0:7 round7 actionhybrid 300 1903000
target/release/examples/preplay_table prelearn:assets/models/preplay-pass2.tpq:none:128:12:none:0:7:1 round7 actionhybrid 300 2003000 3
cargo test --locked --workspace --all-targets
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy test_historical_data test_human_neural test_preplay
```

Tests cover exhaustive single-suit equivalence against the old algorithm,
packed trick membership, legal/terminating games under all rules, private
observations, directed pass destinations, keeper ownership conservation,
per-round policy selection, model-format validation, Rust/NumPy arithmetic,
centered monetary gradients, and whole-deal train/validation separation.

Final validation passed: the full locked workspace/all-targets Rust suite,
all 13 Python tests above, formatting, and `git diff --check`. Release-binary
checks on eight deals per table composition matched round7 exactly with one
or two humans, and the confirmed learned profile with three humans; scores
and moon counts matched (`production-scope-parity.json`). A generator smoke
check also verifies the minimum four-candidate passing budget is respected.
