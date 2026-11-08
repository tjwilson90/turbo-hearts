# Dan's game: stopper retention, nine-assisted moons, and charge information

This report records the round-four default. Use `round4` to reproduce its
`default`-labeled artifacts after the [successful-human update](../round7/README.md).

`th2.db` contains one game for Dan Cervelli:
[771c80b5-8201-4d97-9e88-a77ba50f11bd](https://play.anti.run/game#771c80b5-8201-4d97-9e88-a77ba50f11bd),
started November 8, 2026 at 07:36 UTC. Dan sits East against three `neural_net`
bots. The database was opened read-only, including its WAL. The sanitized
[replay fixture](../../tests/fixtures/dan-game.json) excludes player identifiers,
chat, and the original seed. Event indices below are zero-based within that
fixture, not website trick numbers.

The existing production stopper preference substantially reduces the third
hand's A♥ discard. It does not guarantee prevention of the moon: a retained ace
must sometimes be cashed before a nine lets the shooter bypass it. No additional
production behavior was enabled in this investigation, and nothing was deployed.

## Recorded results and decision audit

| Hand | North score | Dan / East score | South score | West score |
| --- | ---: | ---: | ---: | ---: |
| 1 | 34 | 0 | 0 | 12 |
| 2 | 0 | 0 | 26 | 3 |
| 3 | 0 | -98 | 0 | 0 |
| 4 | -8 | 14 | 3 | 14 |

Dan's game money was +336; the third-hand charged moon alone contributed +294.
Scores and money use the game's actual scoring engine, including Q♠ charging,
J♦ and 10♣. They are different quantities: lower score and higher money are better.

`audit_game` evaluated all 103 non-forced bot decisions among the game's 170
recorded plays, with 32 paired seeds per decision. It compared `default` with
the frozen previous policy, `cautiousdynamic:128:12:none:0:32:0:0.6`.
Only event 145 changed: 20 of the 32 seeds selected a different discard.
This comparison holds the recorded history fixed up to each decision. It does
not pretend that the recorded later actions would persist after changing a move.
The actual server build and original random stream are unavailable; these are
comparisons of current code at recorded information sets.

## Hand 3: the same ace-of-hearts mistake

South leads 9♣ on the second trick. West is void in clubs and holds
`T6S A862H AQJT92D`. At event 145 it discards A♥. West also knows it passed
K♥ K♣ 5♣ across to Dan, who charged Q♠. Dan wins the extended club trick,
collecting A♥ and 10♣, then completes the charged moon.

Expanded decision replays, using 100 seeds and 128 samples per candidate:

| Policy | A♥ discards / 100 |
| --- | ---: |
| Previous production | 65 |
| Current production, 8-money stopper preference | 8 |

The current policy's other choices were 8♥ (40), 9♦ (20), 2♥ (14), 6♥ (10),
A♦ (6), and 10♠ (2). The raw search means were +30.68 for A♥ and +27.75 for
8♥. The preference is bounded: it only keeps the supported master when an
alternative is within 8 estimated money. Sampling variation can exceed that
margin, so this is not an absolute prohibition. At 512 samples it retained A♥
in all 32 tested seeds; at 4,096 samples all 32 chose 8♥.

For a stronger check, `branch_game` starts at the actual dealt position and
simulates the rest of the hand with responsive opponents. Each actor receives
only its own hand and redacted public history. All actors can make claims using
the server's public-information `should_claim` rule. Recorded future moves are
never forced after the branch.

| Policy replacing Dan after event 145 | Dan moons, old bots / 100 | Dan moons, new bots / 100 |
| --- | ---: | ---: |
| Fitted human policy | 27 | 20 |
| GottaTry shooting policy | 91 | 78 |
| Full-strength search | 92 | 80 |

These are repeated simulations on **one deal**, not actual Dan counterfactuals
or independent population-strength evidence. The broad range across responders
shows why a single fitted human model is not enough. With new bots throughout,
forcing A♥ versus 8♥ produced 28 versus 21 human-model moons, 95 versus 71
GottaTry moons, and 99 versus 85 search moons. Branch seeds are `seed*4+seat`,
whereas direct decision replays use `seed`; initial A♥ counts in automatic
branches are consequently 60 versus 6, rather than 65 versus 8.

### Keeping the ace is only the first step

The saved [search continuation](trace-keep-ace-default.json) forces West's 8♥
discard and lets everyone respond. This is a simulated line, not Dan's recorded
play. East captures hearts; North wins a scoreless spade trick and leads 3♦;
West takes J♦. West can now cash A♥ and break the moon, but chooses 2♦ in this
line. East eventually has `A9S KH K85C` and can validly claim even though West
still holds A♥.

The mechanism is specific to Turbo Hearts: A♠ removes the last opposing spade,
then the extended 9♠ trick lets East slough K♥ onto its own winning trick.
There is no need to lead K♥ into A♥. The regression test both exhibits legal
plays doing this and verifies that cashing A♥ at the earlier opportunity splits
the hearts and prevents East's moon.

At West's missed opportunity (event 160 in the *simulated* event stream), the
current policy leads A♥ in 74/100 replays and 2♦ in 26. A♥ averages +57.60
estimated money versus +55.54 for 2♦. Experimental variants do not repair this:

| Variant | A♥ leads / 100 |
| --- | ---: |
| Current production | 74 |
| Separate defensive continuation (`plan`) | 73 |
| Claim-aware continuations (`claims`) | 74 |
| Earlier shooter mixture (`early`) | 7 |
| Preserve + claims + early (`safeearly`, margin 8) | 7 |
| Plan + claims + early (`humanlike`) | 4 |

Starting with forced 8♥, `safeearly` also allowed more moons than production:
79 versus 71 against GottaTry and 89 versus 85 against search. Simply activating
early defensive heuristics or claim-aware rollouts is therefore not supported
by these results. The useful next problem is valuing **when to spend control**
before an extended trick can dispose of a losing heart.

## Hand 2: more samples can favor the wrong move on the actual deal

South's failed moon ended at score 26, money -75. At event 103 it led A♥.
Current and previous policies both choose K♠ in 74/100 replays, A♥ in 22, and
10♠ in four. Mean search estimates favor K♠ (+54.16) over A♥ (+42.01).
At 512 and 4,096 samples, all 32 seeds choose K♠.

That consistency does not establish an improvement. Adaptive continuations on
the actual hands favor A♥ under both responders tested:

| Dan responder | South mean money after forced A♥ | After forced K♠ |
| --- | ---: | ---: |
| Fitted human policy | -35.45 | -79.93 |
| Full-strength search | -55.54 | -110.70 |

Each cell uses 100 continuations with current bots. The discrepancy points to
hidden-hand or continuation-model calibration as something to investigate,
rather than just sampling variance. These hindsight results do not prove K♠ is
worse in expectation given only South's available information. They do rule out
using this example as evidence that more samples necessarily improve play.

At event 107, the following 5♥ lead is nearly tied with 10♠ (51 versus 49 choices
in 100 seeds, means -41.17 versus -40.97). There is no demonstrated policy fix
for that decision here. Other high-heart discards, such as West's K♥ in hand 2
and North's K♥ in hand 4, are unchanged. Those kings are not current masters;
the evidence does not justify a blanket ban on discarding high hearts.

## A promising source of better hidden-hand beliefs: charging

The production sampler enforces ownership of charged cards, passed cards, and
public voids. It does not weight otherwise legal hands by the likelihood that
a human would voluntarily charge with them. In this third hand, Dan actually
held six spades including A♠ and 9♠.

Descriptive counts from the earlier 1,699-game human corpus:

| Human Q♠ holders at first play | Hands | At least six spades | Both A♠ and 9♠ |
| --- | ---: | ---: | ---: |
| All holders | 6,361 | 26.6% | 10.3% |
| Charged Q♠ | 1,986 | 64.7% | 18.6% |

`scripts/queen_charge_statistics.py` reproduces the stored integer counts.
It uses post-pass hands, counts each hand once, and restricts to humans under
Classic rules. These are descriptive statistics across the existing corpus,
not held-out validation of a model. The overall-holder row includes chargers.

A useful next experiment is learning the likelihood of charging from the
charger's own cards and information available at charge time, including
non-charge examples. Tempered likelihood weights could then update sampled
hidden hands while preserving exact known-card and void constraints. Passing
knowledge and charging rules matter; directly substituting the aggregate 65%
frequency would not be a valid posterior for every observer. Fit on training
games, select calibration on validation games, and test on untouched games and
paired matches against several responder populations before enabling it.
Dan's game is now a diagnostic fixture, so it must not count as fresh
confirmation data for a model tuned to this investigation.

## Reproduction and checks

From the repository root:

```sh
cargo build --locked --release -p turbo-hearts-bot --examples
target/release/examples/audit_game bot/tests/fixtures/dan-game.json round4 cautiousdynamic:128:12:none:0:32:0:0.6 32
target/release/examples/replay_decision bot/tests/fixtures/dan-game.json 145 round4 100
target/release/examples/replay_decision bot/tests/fixtures/dan-game.json 145 cautiousdynamic:128:12:none:0:32:0:0.6 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 round4 gottatry auto 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 cautiousdynamic:128:12:none:0:32:0:0.6 gottatry auto 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 145 round4 round4 8H 1
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 103 round4 human AH 100
target/release/examples/branch_game bot/tests/fixtures/dan-game.json 103 round4 human KS 100
python3 scripts/queen_charge_statistics.py data/round4/human-games.jsonl
cargo test --locked -p turbo-hearts-bot --test historical_replay
python3 -m unittest test_historical_data
```

Use `human`, `gottatry`, and `round4` as the responder argument to reproduce the
other branch comparisons. Use `AH` or `8H` instead of `auto` for forced initial
discards. All output JSON records the exact profiles and seed counts. Early
branch result files predate the trace-output field; separate `trace-*` files
retain the first-seed continuation.

To reproduce the missed lead at simulated event 160, concatenate the first 145
events of `dan-game.json` with `dan-kept-ace-continuation.json`, then pass that
array to `replay_decision` at index 160. The fixture contains no inferred human
future moves: its continuation was produced by the simulator.

New regression coverage checks the complete recorded scores, private pass
knowledge, unchanged raw search values and the bounded preference, the earlier
ace-cashing opportunity, the nine-assisted bypass, and claim scoring. Python
tests cover charge counts after passing and identity stripping from older
database metadata. No source database is modified.

Validation passed: `cargo test --locked -p turbo-hearts-bot --all-targets`
(including examples and the inference smoke check), and all six tests in
`test_training`, `test_human_policy`, and `test_historical_data`. The rebuilt
branch executable reproduces the saved first-seed trace and scores exactly;
the charge-statistics script reproduces the stored counts exactly.
