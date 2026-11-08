# Learning from successful humans

This round prioritizes play against humans, especially consistently successful
ones. It adds a training-only skill cohort, two fitted human continuation
policies, and an audit that compares real human decisions to the bot's choices.
The final default uses that fitted policy as a fallback while preserving
inferred runner styles. Candidate selection and follow-up decisions are recorded
in [PLAN.md](PLAN.md).

## Human population and models

The read-only export contains 1,699 completed historical human games, split
chronologically into 1,189 training, 255 validation, and 255 later evaluation
games. Four additional games from `th2.db` remain a separate diagnostic split.
Most historical games are human-versus-human. No player identity is used by
the bot; stable anonymous keys are used only to select the training cohort.

Training outcomes fit an opponent-adjusted ridge rating (penalty 30), with
money/hand modeled as own strength minus mean opposing strength. Among 13
humans with at least 30 training games, the top third selects five players.
Their losses are included. Neither later outcomes nor later decisions affect
cohort selection. The selected players average +5.64 money/hand in training,
+5.98 in validation, and +8.73 in later games; other humans average -9.97 in
the later block. These are observational population summaries, not bot gains.
See [cohort.json](cohort.json).

Two models fit the cohort's 86,820 non-forced training decisions:

* `SkilledPolicy`: a 96-coefficient contextual mixture of uniform, heuristic,
  duck, shooting, stopper-defense, and disruptive-defense action distributions.
  Its 16 context features include suit masters, controlled nines, longest-suit
  fraction, voids, losers, moon viability, and charges.
* `HumanActionPolicy`: a 432-coefficient conditional softmax over legal cards.
  Eighteen context features modulate 24 action features, including rank, suit
  support, control, special cards, winning a trick, and safe disposal contexts.

Both see only the simulated actor's own hand and public information. They
predict action probabilities, not monetary values. Coefficients are compiled
into the binary; there is no runtime model download. Model metadata and
reproducible training settings are in `assets/models/skilled-policy.*` and
`assets/models/human-action.*`.

| Successful-player decisions | General-human mixture NLL | Successful-human mixture NLL | Direct action model NLL |
| --- | ---: | ---: | ---: |
| Validation: 20,436 | 1.31536 | 1.30595 | 1.16183 |
| Later evaluation: 19,464 | 1.31811 | 1.30554 | 1.16772 |

Lower negative log likelihood means better prediction. The direct model also
improves other humans' later-game NLL from 1.29502 to 1.15235. Better prediction
did **not** make it the best continuation model inside search: all its search
variants were less consistent in the validation matches. It remains a useful
independently fitted opponent and research option.

## Initial fitted-only candidate: selection and confirmation

Each historical match begins with the actual post-pass hands and charges.
Subsequent opponents respond through their policies; their recorded future
moves are never forced. The bot sees redacted history, including its own passes,
not the actual opponents' hands. Candidate and baseline use the same starts
and seeds. Intervals cluster all hands and seat replays by original game.

The successful-player comparison includes only replacement seats with at least
one *other* seat originally belonging to the fixed successful cohort. All 244
eligible later games supply 976 hands and 3,876 seat replays per version.
The separate ordinary-human comparison uses all 255 later games.

Validation screened the first 30 eligible games. `skilledmodel`, using the
successful-human mixture for all human opponent continuations, had positive
point estimates against all four responders: +13.95 against `skilledclaim`,
+3.39 against `skilledhybrid`, +10.04 against `actionclaim`, and +1.56 against
`actionhybrid`. Most intervals were wide. It was frozen as the sole finalist
before running the larger later-game confirmation. Every screen is retained
in `validation-*.json` and `direct-validation-*.json`.

`actionclaim` uses the more accurate direct human action model and provable
claims. `skilledclaim` uses the contextual mixture and claims. The two hybrids
choose between the corresponding fitted move policy and frozen round-four
128-sample search with equal probability at each decision, and can claim.
These stronger hybrids check whether the result depends on easy imitation
opponents. They are approximations, not actual live humans.

| Later-game responder | Hands/version | Money gain/hand | Paired 95% interval |
| --- | ---: | ---: | ---: |
| Direct human policy + claims | 3,876 | +9.50 | [+4.19, +14.81] |
| Direct policy / search hybrid | 3,876 | -2.95 | [-9.38, +3.47] |
| Successful-human mixture + claims | 3,876 | +11.57 | [+6.40, +16.74] |
| Mixture / search hybrid | 3,876 | +3.30 | [-2.20, +8.80] |
| General-human policy, all later games | 4,080 | +9.48 | [+4.50, +14.46] |

The direct responder comparison increases own moons from 349 to 470 and
reduces opposing moons from 324 to 301. Its hybrid instead increases opposing
moons from 450 to 465; the uncertain negative money estimate must not be
hidden by the positive imitation-only result. Both hybrid comparisons remain
inconclusive. The ordinary-human check is positive as well.

The original fitted-only finalist also ran on 1,000 fresh deals per responder
(4,000 seat replays/version), seeds 1200000–1200999:

| Fresh-deal responder | Money gain/hand | Paired 95% interval |
| --- | ---: | ---: |
| actionclaim | +3.50 | [-0.33, +7.34] |
| actionhybrid | +1.32 | [-2.94, +5.59] |

Both fresh-deal comparisons are inconclusive. The synthetic arena marks its
simulated seats as human to exercise the human model. A separate integration
test checks exact unchanged search values when all public seats are bots.

## Final default: keep inferred styles, use the human policy as fallback

`SearchExperiments::production()` now selects `skilledfallback`: full fitted
continuations for public human seats unless the existing conservative model
has inferred a duck/shooter style. The initial finalist overwrote that
evidence; preserving it lets the bot retain a recognized runner model.
The supported-heart margin stays 8, budgets stay 128/12, and passing and
charging stay heuristic. No deployment was performed.

This follow-up was motivated after seeing the initial hybrid results.
Two variants were screened on 100 validation games: `skilledfallback`
(retain inferred styles) and `skilledadaptive` (also retain sampled early
intent for human seats). Gains against `actionclaim` were +16.75
[9.90, 23.59] and +15.78 [8.49, 23.07]; against `actionhybrid` they were
-0.005 [-7.27, 7.26] and +0.009 [-7.28, 7.30]. The extra early signal
showed no incremental benefit, so the simpler fallback was frozen.

| Final comparison | Hands/version | Money gain/hand | Paired 95% interval |
| --- | ---: | ---: | ---: |
| Successful-human starts / direct learned responder | 3,876 | +9.52 | [+4.19, +14.86] |
| Successful-human starts / direct-search hybrid | 3,876 | -2.92 | [-9.35, +3.51] |
| Successful-human starts / mixture-search hybrid | 3,876 | +3.15 | [-2.36, +8.66] |
| All later starts / general-human responder | 4,080 | +9.26 | [+4.29, +14.24] |
| Fresh deals / direct learned responder | 4,000 | +10.20 | [+6.59, +13.80] |
| Fresh deals / direct-search hybrid | 4,000 | -3.23 | [-7.59, +1.13] |

The final fresh checks use 1,000 new deals per responder, seeds
1400000–1400999. The direct-human gain is confirmed there; the hybrid
comparison remains inconclusive with a negative point estimate. The
historical confirmations use the already examined later block, and are
reported as diagnostics for the final variant, not new untouched evidence.

Promotion is justified by the human-focused gains, including the separate
direct action predictor and ordinary-human responder, rather than proof
of improvement against every stronger population. The hybrid result is a
material limitation: there is no demonstrated gain against that stronger
responder, and losses remain plausible. Keeping inferred styles is a
structural correction; these data do not establish a separate significant
gain from it over the fitted-only version.

Against the direct human responder on fresh deals, own moons increase
221→387 and opposing moons decrease 219→192. Against the hybrid, own
moons increase 203→289 and opposing moons decrease 297→283, but the
queen-plus-8–12-hearts near-miss proxy rises 168→285. Better defense alone
does not imply better total payoff. This suggests investigating overly
optimistic own moon continuations against capable defenders; the proxy
does not label deliberate shooting attempts.

Mean candidate play computation is about 25–26 ms per hand versus
13–14 ms for round4, with maximum candidate decisions below 95 ms in
these concurrent runs. This is not an isolated or portable benchmark.
The one-second soft production decision cap remains.

A rollout optimization evaluates only the sampled policy component.
All choices and raw values matched the earlier implementation across
100 seeds at each of three recorded positions; see
[component-optimization-parity.json](component-optimization-parity.json).
The full Rust workspace/all-target suite and all nine Python tests passed
after enabling the final default, including legality across all charging
rules, redaction, Python/Rust inference agreement, chronological cohort
selection, unchanged all-bot values, and default/profile parity.

The historical test block was also used in earlier rounds of research. These
are later-game results for the same population, not an entirely untouched
corpus or a test on new player identities. The fitted model may exploit its
responders' limitations. Actual human strength needs a subsequent live-game
comparison; the offline results do not establish a live-human win-rate gain.
The corpus contains only Classic games. Game-clustered intervals do not quantify
generalization to new people; the cohort contains only five selected players.

## What the human decisions teach us

The `human_choices` audit samples positions independently of outcomes, compares
the human move with a bot recommendation made from redacted history, and branches
both choices with responsive continuations on the actual hands. Equivalent
ordinary ranks are treated alike; nines, chargeable cards, and suits remain
distinct. Actual recorded final money is kept separate from simulated action
credit. A good recorded final result alone does not make a particular move good.

**Retaining a charged multiplier can be better than shedding it.** In hand two
of [this game](https://play.anti.run/game#ad9a17d1-4755-482c-bc50-d22cec3ac763),
event 85, East holds `J96S A65D KJT963C`, with Q♠ and 10♣ charged. On the current
`5C KS 7C` trick, both K♣ and 10♣ win. The human plays K♣ and keeps the charged
ten. In 100 adaptive continuations per choice, K♣ versus 10♣ produces +113.27
versus +70.08 money under the fitted mixture, and +259.69 versus +65.53 under
round-four search. The human eventually earned +588 in the recorded hand; that
entire payoff is **not** attributed to this one choice. Across 100 search seeds,
round4 chooses K♣ 63 times and 10♣ 37; the candidate chooses K♣ 72 times and
10♣ four times, with the rest choosing lower clubs. The final fallback is checked separately
below. This supports preserving
control and future options, not a universal rule to hold 10♣.

**Early hearts and nine-extended tricks remain important.** In hand one of
[this game](https://play.anti.run/game#5c8ca103-9ebc-4283-8f16-ebc2630d3fb8),
event 30, North holds `Q3S T6H AQT85D` after `9C 6C AH 3C`. The nine requires
another circuit. North knows it passed A♣ to East. The human discards 10♥;
the bot discards Q♠. The heart is better in 100-seed continuations under both
the mixture (+1.37 versus -9.37) and search (+20.57 versus -18.51). The candidate
still chooses Q♠ in all 100 root replays. This is an unresolved value-model
weakness, retained as a diagnostic fixture, not advertised as fixed.

The two anonymized prefixes are `human-king-before-charged-ten.json` and
`human-heart-before-queen.json` in `bot/tests/fixtures`. The `case-*.json` files
retain the 100-seed counterfactuals, including less favorable follow-ups: the
apparent 8♦ advantage in case 1195 shrank to +1.74/+4.98, and the 9♦ advantage
in case 1211 reversed under one responder (-6.90/+14.32). Case-file `default`
labels refer to the then-current round4 policy; reproduce them with `round4`.

The initial fitted-only later audit covers 386 sampled decisions from 60 games, eight continuation
seeds per move under `actionclaim` and `round4`. Round4 disagrees with the human
at 185 positions; the candidate disagrees at 192. Candidate-minus-round4 money
is +4.82 [-0.17, 9.81] with learned responders and -0.99 [-5.10, 3.12] with
search responders. Under search continuations, the real human choices average
+5.11 [0.05, 10.17] above round4. Under learned continuations, that comparison
reverses and is inconclusive. Lead choices show the largest changes. This is
evidence to investigate particular decisions, not to copy every disagreement.
See [later-choice-audit.json](later-choice-audit.json) for all categories and
game-clustered intervals. Subgroups are descriptive, not separate promotion tests.

## Original slough, Dan, and early intent

The supported-master-heart preference remains bounded at 8 money. The following
comparison is for the initial fitted-only candidate; final-default diagnostics
are listed separately below. At the
original reported A♥ position, round4 sloughs it 0/100 times and the candidate
1/100. In Dan's game, event 145, the count falls from 8/100 to 3/100. The new
model therefore helps the latter position but is not a universal stopper fix.

Continuing Dan's actual hands with all bot seats using each version gives:

| Dan responder | Round4: Dan moons / 100 | Candidate: Dan moons / 100 |
| --- | ---: | ---: |
| Direct human model with claims | 16 | 14 |
| Dedicated shooting policy | 78 | 85 |
| Round-four search | 80 | 80 |

Against the human responder, West's average money improves from +0.07 to +15.70.
Against the dedicated shooter it worsens from -63.76 to -73.86. Retaining the
ace does not ensure it will be cashed before a nine-assisted bypass. These
single-hand simulations are diagnostics, not predictions of Dan's behavior.

The final production replay keeps the same four initial decisions as the
fitted-only candidate above: 1/100 original A♥ sloughs, 3/100 Dan A♥ sloughs,
72/100 K♣ and 4/100 10♣ choices in case 1210, and 100/100 Q♠ in unresolved
case 1202. In the Dan continuation the final inferred-style fallback yields
14, 83, and 80 moons against the direct human, dedicated shooter, and search
responders respectively (round4: 16, 78, 80). West money against the direct
human responder is +14.37 versus round4 +0.07. The dedicated-shooter weakness
remains (-71.61 versus -63.76). See `replay-*-production.json` and
`dan-continuation-production-*.json`.

The same 386-position audit was repeated for the final default:

| Responsive continuation | Production minus round4 money/decision | Game-clustered 95% interval |
| --- | ---: | ---: |
| actionclaim | +4.82 | [-0.17, +9.81] |
| round4 | -0.99 | [-5.10, +3.12] |

These are simulated values of sampled moves, not gains per whole hand.
See [final-choice-audit.json](final-choice-audit.json). Its `default` profile
is the new `skilledfallback` configuration.

Early intent alone remains available as `intent`. Its earlier fresh mixed-bot
gain was real in that evaluation, and the earlier requirement to rule out even
a one-money search-population loss was unnecessarily restrictive for this task.
This round instead screens it on the successful-human starts: -0.49 against
`actionclaim` and -0.25 against `actionhybrid` (both inconclusive). The successful
human continuation model has stronger direct support here. Adding intent to a
fitted policy is not automatically additive: fitted continuations replace the
style choice at human seats. No new result establishes a benefit from blindly
combining all switches.

The all-suit charging association, early heart distribution, and discarding
nines remain represented in the research models and prior reports. Their
individual predictive plausibility is not proof that every corresponding
search change improves payoff.

## Reproduction

Generated full corpora stay in ignored `data/`; source databases, chats, and
identities are not published. Open SQLite read-only without bypassing its WAL.

```sh
cargo build --locked --release -p turbo-hearts-bot --examples
python3 scripts/export_ranked_games.py turbo-hearts.db data/round7/players.jsonl --recent th2.db
target/release/examples/corpus_scores data/round7/players.jsonl data/round7/scored.jsonl
OPENBLAS_NUM_THREADS=1 python3 scripts/rank_human_games.py data/round7/scored.jsonl data/round7/ranked.jsonl bot/evaluation/round7/cohort.json
target/release/examples/skilled_data data/round7/ranked.jsonl data/round7/actions.bin
OPENBLAS_NUM_THREADS=1 python3 train_skilled_policy.py data/round7/actions.bin assets/models/skilled-policy.rs
# Rebuild so the next exporter includes the newly fitted mixture.
cargo build --locked --release -p turbo-hearts-bot --examples
target/release/examples/human_action_data data/round7/ranked.jsonl data/round7/direct-actions.bin
OPENBLAS_NUM_THREADS=1 python3 train_human_action.py data/round7/direct-actions.bin assets/models/human-action.rs
cargo build --locked --release -p turbo-hearts-bot --examples

target/release/examples/historical_arena data/round7/ranked.jsonl skilledmodel round4 actionclaim 244 2 successful
# Repeat with actionhybrid, skilledclaim, and skilledhybrid.
target/release/examples/historical_arena data/round7/ranked.jsonl skilledmodel round4 humanclaim 255 2
target/release/examples/arena skilledmodel round4 actionclaim 1000 1200000
# Repeat with actionhybrid for the original finalist's fresh-deal cross-check.
# Final production profile; its historical block was already examined above.
target/release/examples/historical_arena data/round7/ranked.jsonl skilledfallback round4 actionclaim 244 2 successful
# Repeat with actionhybrid and skilledhybrid; humanclaim uses 255 games, no cohort argument.
target/release/examples/arena skilledfallback round4 actionclaim 1000 1400000
# Repeat with actionhybrid; both use independent new seeds.

target/release/examples/human_choices data/round7/ranked.jsonl data/round7/human-choices-later-round4.jsonl round4 2 60 8 actionclaim,round4
target/release/examples/human_choices data/round7/ranked.jsonl data/round7/human-choices-later-skilledmodel.jsonl skilledmodel 2 60 8 actionclaim,round4
OPENBLAS_NUM_THREADS=1 python3 scripts/summarize_human_choices.py data/round7/human-choices-later-round4.jsonl data/round7/human-choices-later-skilledmodel.jsonl bot/evaluation/round7/later-choice-audit.json

target/release/examples/human_choices data/round7/ranked.jsonl data/round7/human-choices-later-production.jsonl skilledfallback 2 60 8 actionclaim,round4
OPENBLAS_NUM_THREADS=1 python3 scripts/summarize_human_choices.py data/round7/human-choices-later-round4.jsonl data/round7/human-choices-later-production.jsonl bot/evaluation/round7/final-choice-audit.json

target/release/examples/replay_decision bot/tests/fixtures/human-king-before-charged-ten.json 85 skilledfallback 100
target/release/examples/branch_game bot/tests/fixtures/human-king-before-charged-ten.json 85 round4 round4 KC 100
# Compare TC; replace both profiles with skilledclaim for the other responder.
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 skilledfallback actionclaim auto 100

cargo test --locked --workspace --all-targets
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy test_historical_data
```

All research profiles build on the frozen `round_four()` baseline. `round4`
also freezes hybrid responders, so changing production does not silently change
the opponent or baseline in a reproduction. Production budget customization
still requires `with_experiments(SearchExperiments::production())` after
`with_config`. `evaluate` continues to expose raw search values before the
bounded stopper preference is applied by `play`.
