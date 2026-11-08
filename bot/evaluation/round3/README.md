# Follow-up: adapting without assuming too much

The subsequent [human-play follow-up](../round4/README.md) retains these changes
and adds a bounded preference for preserving supported master hearts.

The previous round found a real opportunity and a real failure: modeling
opponents improved mixed matches by 17.69 money per hand, but lost 7.82 against
full-strength search. This follow-up tests whether useful adaptation can be
retained while reducing unsupported assumptions about opponents and moons.

**The selective model plus changing-shooter continuations is now enabled by
`NeuralNetworkBot::new()`**, which is the server's existing `NeuralNet` strategy
constructor. The frozen baseline remains available as `defend:128:12`. Passing
and charging stay heuristic, with 128 samples and 12-card exact endgames. No
checkpoint is loaded at runtime, and no deployment was performed.

## Independent confirmation

The preferred candidate was fixed before opening fresh seeds 150000–151999.
The simpler selective model was prespecified as a fallback. Mixed gains had to
remain positive at 97.5% confidence to account for the two candidate choices;
the other two primary lineups could not show a clear loss at 95% confidence.

| Opponents | Original deals / hands per version | Gain over prior default, 95% CI |
| --- | ---: | ---: |
| Heuristic + duck + shooting | 2,000 / 8,000 | **+4.23 [2.34, 6.11]** |
| Three heuristic bots | 1,000 / 4,000 | **+1.95 [0.41, 3.49]** |
| Three full-strength search bots | 1,000 / 4,000 | −0.33 [−2.44, 1.77] |

The mixed-lineup 97.5% interval is **[2.07, 6.38]**, so the preferred candidate
meets the selection rule. The search-opponent result is inconclusive; it does
not establish an improvement or equivalence against that lineup.

On the fresh mixed matches, opponent moons decreased from **1,603 to 1,485**.
Own moons increased from 614 to 641, while queen-plus-8–12-hearts near misses
decreased from 143 to 134. The money gain includes all failed attempts and
charges. Mean play computation was 15.15 ms per whole hand versus 12.20 ms for
the frozen baseline during concurrent runs. The existing one-second soft
decision cap remains in effect in production.

Additional checks across the other five charging rules and against shooting
and random opponents are recorded in the `scope-*.json` files. These reused
seed checks assess scope, not independent parameter selection. Some rule
variants produce identical event sequences under the tested charge policies.
All primary and scope results are collected in [CONFIRMATION.md](CONFIRMATION.md).

## What was tested

1. **Selective opponent modeling.** Start with a 90% heuristic, 5% duck, 5%
   shooting prior. Adopt a non-heuristic policy only after at least three
   non-forced observations and sufficient posterior support. Compare support
   thresholds of 90%, 75%, and 60%. Otherwise preserve the original continuation,
   including its opposing-shooter mixture. Inference uses a cloned random
   stream, so uncertain evidence leaves sampled hands and simulation randomness
   unchanged. This is a model posterior, not a calibrated probability of a
   human's intentions. No persistent player identity is learned.
2. **Changing intentions during a rollout.** The original model decides whether
   an opponent might be shooting only at the initial search position. The new
   variant lets a still-viable shooter emerge later, after capturing three
   hearts or the queen. It keeps the existing 50% shooting mixture and uses
   only public captured points plus each simulated actor's own hand. It stops
   shooting when another player wins a moon point.
3. **Explicit moon defense.** Simulated defenders may deliberately take a point
   or give one to another defender to break an established opposing moon.
   Compare 100% and 50% defender mixtures; do not convert known duck/shooter
   policies into defenders. This changed play but did not show a reliable gain.
4. **Agreement before valuing a moon.** Both alternating halves of sampled
   deals must prefer shooting before it is valued above avoidance. This tests
   noisy continuation selection at the same rollout budget. It did not produce
   a clear general improvement.
5. **Charge planning with changing opponents.** Reuse the promising continuation
   in charge search, leaving passes heuristic. It improved heuristic and mixed
   matches but again traded away performance against search opponents.
6. **One more trick of exact search.** Compare exact 16-card and 12-card horizons.
   More perfect-information search did not fix the opponent-model problem: it
   regressed against heuristic players and cost substantially more computation.
   Two bounded caches were implemented and checked independently of strength.

An additional ablation merged duck/shooting posterior mass after the modeled
opponent could no longer shoot (the two policies then have identical actions).
It did not improve on the simpler 60% selective model in development.

## Development results

All rows below use 300 original deals, four seat replays per version, Classic
rules, and seeds beginning at 140000. Baseline is the frozen `defend:128:12`.
Values are paired money-per-hand gains with 95% intervals, clustered by deal.
These are exploratory screens on reused deals, without multiplicity correction.

| Change | Mixed opponents | Heuristic opponents | Search opponents, 32 samples |
| --- | ---: | ---: | ---: |
| Selective model, 60% support | +1.71 [−0.68, 4.11] | 0.00 [0.00, 0.00] | +0.58 [−0.40, 1.56] |
| Changing shooters | +5.99 [1.51, 10.48] | −1.13 [−4.14, 1.89] | +0.14 [−3.34, 3.62] |
| Selective model + changing shooters | +7.12 [2.02, 12.22] | −1.13 [−4.14, 1.89] | +0.81 [−2.99, 4.61] |
| Changing shooters + charge search | +10.81 [3.34, 18.29] | +13.49 [7.30, 19.67] | −5.25 [−12.51, 2.01] |
| 50% defenders | −1.93 [−6.54, 2.69] | +1.70 [−2.12, 5.51] | +0.94 [−3.87, 5.75] |
| Agreement between sample halves | −0.84 [−2.90, 1.22] | +1.21 [−0.34, 2.76] | +0.08 [−0.98, 1.13] |
| Exact 16-card horizon | +0.27 [−3.01, 3.55] | −3.86 [−6.68, −1.05] | −0.37 [−2.39, 1.64] |

Against full-strength 128-sample search opponents, the selective/changing
combination scored −0.33 [−3.81, 3.15], while changing shooters plus charge
search scored −3.73 [−10.80, 3.34]. These do not establish equivalence, but they
favor testing the former rather than adopting the more aggressive charge plan.

Every screen, including unsuccessful combinations and thresholds, is retained
in JSON and summarized in [SCREENING.md](SCREENING.md). The prespecified
selection and confirmation procedure is in [PLAN.md](PLAN.md). Confirmation
uses new seeds 150000–151999, separate from all earlier training and matches.

## Exact-search cache

The first cache keys complete remaining ownership, trick order, captured
points, next actor, and suits led. Its overhead offset most of its benefit.
The second cache combines strategically equivalent states: preserve card
ownership order within each suit, the special nine, and each chargeable/scoring
card, while omitting gaps between neutral ranks. The trick key preserves the
winning player, number of higher remaining cards, suit, length, and extended
nine status. Captured points and remaining scoring cards determine the trick's
scoring content. Each cache belongs to one real decision, so charges are fixed.
Storage is capped at 65,536 entries; exceeding that limit falls back to exact
uncached traversal, not an approximate answer.

On 100 original mixed deals, both caches produced exactly the same outcomes
as uncached 16-card search. The canonical version reduced mean whole-hand play
time from 48.98 to 23.54 ms compared with the first cache. At 12 cards it also
preserved outcomes across 300 deals, with a smaller timing difference: 11.73
versus 12.45 ms. These are concurrent-run timings, not controlled benchmarks.
Caching does not itself establish stronger play, and the longer exact horizon
is not selected on these results.

## Reproduce

```sh
cargo build --locked --release -p turbo-hearts-bot --example arena
target/release/examples/arena cautiousdynamic:128:12:none:0:32:0:0.6 \
  defend:128:12 mixed 2000 150000
target/release/examples/arena cautious:128:12:none:0:32:0:0.6 \
  defend:128:12 mixed 2000 150000
target/release/examples/arena dynamiccharge:128:12 defend:128:12 defend:128:12 300 140000
target/release/examples/arena guard:128:12:none:0:32:0.5 defend:128:12 mixed 300 140000
target/release/examples/arena consensus:128:12 defend:128:12 mixed 300 140000
target/release/examples/arena canonical:128:16 cached:128:16 mixed 100 140000
```

The optional arguments after `:SAMPLES:EXACT` are model path (or `none`), neural
prior samples, allocation minimum rounds, defender fraction, and style-support
threshold. The final three slots only affect applicable experimental profiles.
Use `defend`, not a moving production `default`, to reproduce the baseline.

New profiles: `guard`, `cautious`, `cautiousguard`, `selective` (merge equivalent
styles), `consensus`, `guardconsensus`, `dynamic`, `dynamicguard`,
`cautiousdynamic`, `dynamiccharge`, `cached`, and `canonical`.

## Correctness checks

Regression tests cover a concrete deal in which an opponent's moon opportunity
emerges only after simulation starts, taking/supplying a heart to block a moon,
stopping defense once a moon is broken, unchanged baseline values and random
streams before sufficient evidence, and legal reproducible experimental games.
The canonical solver is compared with both the original recursive solver and
the existing full solver, including charged hands, extended tricks, 16-card
positions, and multiple constrained hidden deals sharing one cache.

The full workspace test suite passed after enabling the production policy,
and all release examples built successfully. A release-build comparison of
`default` with the confirmed candidate reproduced identical scores, moon counts,
and root evaluation counts across 300 deals and all four seats (1,200 hands per
version); see [production-parity.json](production-parity.json).

The still-active limitations are imperfect opponent models, sampled hidden
hands, and heuristic continuations. Fresh bot matches measure the implemented
lineups; they do not establish optimal Hearts play or performance against humans.
