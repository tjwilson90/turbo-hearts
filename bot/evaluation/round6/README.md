# Charging, early control retention, and preventive discards

This report records the round-four default. Use `round4` to reproduce its
`default`-labeled artifacts after the [successful-human update](../round7/README.md).

This round implements the user's additional observations as separate experiments
on the round-four production policy. The [plan](PLAN.md) was fixed before match
screens. The new profiles remain available independently so that an improvement
in prediction or one particular hand cannot hide a loss elsewhere.
Early intent improved fresh mixed-opponent matches, but did not satisfy the
predeclared requirement to exclude a loss worse than 1 money/hand against the
search population. All new additions therefore remain experimental. The
production stopper preference is unchanged, and nothing was deployed.

## All four charged cards convey suit-length information

`train_charge_policy.py` extracts a human's first charging opportunity in each
Classic non-keeper hand. A positive example is an eligible held card charged on
that turn; a negative is an eligible held card declined on that turn. Later
charging turns do not duplicate the observation. Keeper hands are excluded
because ownership can change after an initial charging round. No bot charging
choices are used for fitting.

| Held card | Human holdings | Charges at first opportunity | Mean suit length, all holdings | Mean suit length, charged |
| --- | ---: | ---: | ---: | ---: |
| Q♠ | 4,776 | 1,464 | 4.60 | 5.74 |
| A♥ | 4,776 | 1,199 | 4.40 | 5.60 |
| J♦ | 4,709 | 1,008 | 4.26 | 5.44 |
| 10♣ | 4,817 | 1,702 | 4.81 | 5.89 |

The association is present for every charged card. These descriptive counts
use all three existing chronological splits. They differ from round five's
Q♠ counts, which included keeper hands and any eventual charge.

The fitted model estimates charge probability separately for each card in five
suit-length bins: 1–3, 4, 5, 6, and 7+. Sparse bins shrink toward that card's
overall training rate. Validation selected 64 pseudo-observations from the
predefined choices 4, 16, 64, and 256; probabilities are clipped to [0.02, 0.98].
The model does not force monotonicity, so short-suit fake Q♠ charges remain
possible. Coefficients and exact counts are in `assets/models/charge-policy.*`.

| Split | Decisions | Suit-length model NLL | Constant per-card NLL |
| --- | ---: | ---: | ---: |
| Train | 13,413 | 0.446 | 0.569 |
| Validation | 2,941 | 0.494 | 0.600 |
| Test | 2,724 | 0.568 | 0.674 |

Lower negative log likelihood is better. This is out-of-sample charging
prediction, not proof of playing-strength gains. The temporal test games are
the existing research corpus's test split, not a new independent human cohort.

`chargebelief` weights otherwise feasible hidden deals by these observed human
charging likelihoods to power 0.5. It incorporates both charges and declines,
skips the observing player's own decisions, and never relaxes known cards,
passed cards or public voids. It changes beliefs, not the bot's charge choices.
No assumption that a charger is shooting is made. Non-Classic and keeper
inference remains disabled. Public player type identifies human observations;
player identity is neither fitted nor retained.

## Earlier shooting evidence from retained control

`intent` adds a soft opposing-shooter mixture before captured hearts expose a
run. For each sampled hand, it checks whether an observed early player could
have unloaded a legal king/ace but instead played low in the led suit.

Evidence is restricted to the first 24 publicly played cards, live moon
possibilities, and non-forced follow-suit choices. The current trick must have
no moon points or 10♣, must not be hearts, and must not already contain the led
suit's nine. The context must be the opening club trick, the last play of an
ordinary trick, or the first spade trick with Q♠ charged. The last condition is
approximate evidence, not a claim that singleton charged queens are impossible.

Low-preferring and high-preferring rank distributions use exponential slope 2,
with 25% uniform action noise. Their log likelihood ratio updates initial odds
of 0.1/0.9, with power 0.5 and a maximum shooter mixture of 0.75. No evidence
means no additional mixture. These constants are heuristic assumptions, not
fitted or calibrated probabilities of human intent. High plays reduce the
weight; forced low plays do not contribute. Hidden-card uncertainty is averaged
across feasible deals. The experiment supplements the existing fallback style
only, uses a separate RNG stream, and stops treating a player as a shooter when
its moon is broken.

## Disrupt runs before they become obvious

`disrupt` adds a third own continuation alongside avoidance and shooting:

- Supply a low heart to another player's completed trick if that eliminates
  a still-possible opposing moon. This can act before any hearts are captured,
  while retaining a supported master. The real engine verifies completion,
  including nine extensions; current highest card is not assumed to win early.
- Slough an off-suit non-heart nine when the bot lacks master control of that
  suit. Removing it prevents a runner from later winning a trick extended by
  that nine. Keep nines supported by the bot's own master card. Nine of hearts
  is handled by the heart-supply tactic because it also donates a moon point.
- Cash a master or take/supply a point to break a run after opposing points
  have appeared, using the existing tactical defender.

This is an additional continuation to evaluate, not a mandatory root discard.
The continuation's value is averaged over all sampled deals before comparing
it with avoidance and shooting. There is no hidden-deal-specific choice of plan.
`humanaware` combines all three additions. All four profiles keep the existing
8-money stopper preference.

## Development comparisons

Each comparison uses paired seat replays with 128-sample candidates and the
current production baseline. Confidence intervals cluster by original deal.
The mixed population is heuristic + duck + GottaTry; the fitted human responder
also makes provable claims. Search opponents in the initial screen use 32
samples and 12-card exact endings. Seeds start at 600000.

| Candidate | Mixed: gain [95% interval] | Claiming human model | Search, 32 samples |
| --- | ---: | ---: | ---: |
| Charging beliefs | -1.06 [-10.31, 8.19] | +3.73 [-4.86, 12.32] | +4.45 [-8.46, 17.36] |
| Early intent | +6.82 [1.69, 11.95] | +0.89 [-3.17, 4.96] | +2.61 [-4.41, 9.64] |
| Disruptive continuation | +1.17 [-2.45, 4.80] | +0.06 [-2.10, 2.22] | -1.28 [-3.92, 1.36] |
| All three | +2.19 [-7.40, 11.78] | -0.99 [-9.43, 7.45] | +0.34 [-12.64, 13.32] |

Mixed uses 800 hands/version; the other screens use 400. These are small,
exploratory screens. The synthetic arena presents all seats as humans, so the
human charging likelihood is deliberately stressed against its synthetic
opponents' charging behavior. Historical evaluations instead retain original
player types and actual recorded charging decisions.

On the first 30 validation games' post-pass hands (480 seat replays/version),
early intent gained -2.17 [-6.90, 2.56] against the claiming human model and
-2.20 [-9.16, 4.76] against search. Charging beliefs gained -0.43 [-7.41, 6.55]
and +1.26 [-7.21, 9.74], respectively. Intervals cluster by original game.
None establishes an improvement on human-shaped starting positions. Opponents
respond through policies; original human actions are never forced after the
start of a counterfactual hand.

## Fresh confirmation of early intent

Only `intent` advanced from the initial screens. Fresh seeds start at 800000;
each comparison uses the same current production baseline and all four seats.

| Opponents | Hands/version | Money gain per hand | Paired 95% interval |
| --- | ---: | ---: | ---: |
| Mixed | 8,000 | +2.582 | [0.875, 4.288] |
| Claiming human model | 8,000 | +0.712 | [-0.353, 1.777] |
| Search, 32 samples | 4,000 | -0.136 | [-2.176, 1.904] |
| Heuristic | 4,000 | -0.432 | [-2.294, 1.430] |
| Full-strength production search | 2,000 | -1.166 | [-3.617, 1.285] |

Against mixed opponents, own moons increased 572→618 and opposing moons fell
1,398→1,350. Against search, opposing moons also fell, 460→419, but own moons
fell 173→165 and the money comparison remained uncertain. Preventing more moons
does not by itself establish higher overall expected payoff. These matches
support the early-control-retention idea against the mixed population, but do
not establish a gain against live humans or satisfy the broader promotion rule.

## Dan's game and the original reported slough

At Dan's event 145, direct 100-seed replays discard A♥ 8 times under current
production, 5 with charging beliefs, 4 with intent, 8 with disruption, and 3
with all three. All four research variants retain A♥ in all 100 replays of the
original reported game's ace-slough position. These fixtures are diagnostics,
not independent evidence of strength.

Adaptive continuations from Dan's position, 100 runs per cell:

| Dan responder | Current production: Dan moons | Early intent | All three |
| --- | ---: | ---: | ---: |
| Fitted human policy | 20 | 12 | 18 |
| GottaTry | 78 | 71 | 90 |
| Full-strength search | 80 | 74 | 76 |

The combined model's lower initial A♥ discard count does not guarantee a better
continuation: it permits more moons against GottaTry. All numbers are simulations
on one deal, not claims about what Dan himself would do.

The second-hand choice at event 103 is mostly unchanged. Charging beliefs and
the combined profile choose A♥ 23 times and K♠ 70; intent and disruption remain
at A♥ 22 and K♠ 74, with the remaining choices 10♠. The hidden-hand/continuation
miscalibration identified in round five is not demonstrably repaired.

The later ace-cashing opportunity in the saved *simulated* continuation also
remains unresolved. Production leads A♥ in 74/100 replays; intent does so in 59,
disruption in 73, charging beliefs in 28, and the combination in 22. Better
initial stopper retention therefore cannot be substituted for checking later
decisions and whole-hand outcomes.

## Reproduction and validation

```sh
OPENBLAS_NUM_THREADS=1 python3 train_charge_policy.py data/round4/human-games.jsonl assets/models/charge-policy.rs
cargo build --locked --release -p turbo-hearts-bot --examples
target/release/examples/arena intent round4 mixed 200 600000
target/release/examples/arena intent round4 mixed 2000 800000
target/release/examples/historical_arena data/round4/human-games.jsonl intent round4 humanclaim 30 1
target/release/examples/replay_decision bot/tests/fixtures/dan-game.json 145 intent 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 intent gottatry auto 100
cargo test --locked --workspace --all-targets
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy test_historical_data
```

Profiles accept the usual `:SAMPLES:EXACT` suffix. Charging power occupies slot
6, e.g. `chargebelief:128:12:none:0:32:0.5`. Every result JSON stores its exact
profile, sample count/seed information where applicable, and interpretation.
The existing chronological export is unchanged and the databases are read-only.

Regression coverage checks all charged suits, negative charging evidence,
first-opportunity deduplication, scope restrictions, high versus low safe play,
forced/dangerous-play exclusions, early point supply and extended-trick
completion, off-suit nines and own control, legal/reproducible search budgets,
and all charging rules. Existing production decision regressions remain intact.
The complete workspace test suite, example compilation, and benchmark smoke
checks passed, as did all seven Python tests. The existing unrelated server
dead-code warning remains.
The production policy exactly reproduces the choices and raw search values of
the 100-seed Dan replay saved before this round (`production-parity.json`).
