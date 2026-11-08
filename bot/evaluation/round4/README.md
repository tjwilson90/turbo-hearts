# Human-play follow-up and premature stopper discards

The reported first hand of game
`a29f8493-60f9-445c-9992-bd91b6593caf` is retained as an anonymized regression.
South discarded AH from `AJ964H 742D` onto West's `KC 5C TC` trick, before any
moon points had been captured. South knew it had passed QH to West. AH was the
only legal discard that made West's remaining hand claimable.

## Production change and interpretation

The production bot now preserves a supported current master heart when search's
best alternative is within 8 money of discarding it. This applies only when
void in the led suit, losing the current trick, holding at least three hearts,
and an opponent can still shoot. It releases the preference once opposing moons
are impossible, and permits a discard when search finds a greater advantage.
This is an explicit bounded strategic preference, not a claim that estimated
expected money is calibrated. `evaluate` continues to return unmodified search
values; `play` applies the preference. No additional rollouts are required.

On the reported position, the frozen production configuration discards AH in
58/100 seeded 128-sample replays; the 8-money candidate does so in 0/100.
The separate defensive continuation alone still discards it in 61/100 runs.
That result narrows the original diagnosis: adding a defensive continuation
alone does not repair this risk estimate. Root choices in this fixture are
also tested across 32 seeds, including the maximum permitted value sacrifice.
The original server's build and random stream were unavailable; these are
replays using current code, not recovered original search traces.

The candidate met the fresh confirmation rule fixed in [PLAN.md](PLAN.md).
`NeuralNetworkBot::new()` and `with_seed()` now enable it. No server deployment
was performed. `SearchExperiments::round_three()` freezes the previous default,
and research profiles continue to build on that frozen configuration.

| Opponents / starting positions | Hands per version | Money gain per hand | Paired 95% interval |
| --- | ---: | ---: | --- |
| mixed | 8,000 | +1.241 | [+0.781, +1.702] |
| heuristic | 4,000 | -0.146 | [-0.571, +0.278] |
| defend | 2,000 | +0.147 | [-0.243, +0.537] |
| humanclaim | 4,000 | -0.056 | [-0.565, +0.454] |
| historical | 4,080 | +0.012 | [-0.400, +0.424] |

`mixed` is heuristic + duck + shooting; `defend` is three full-strength
128-sample search opponents. `humanclaim` is the fitted human model with claims.
`historical` uses all 255 held-out games' 1,020 hands after the original passes,
with that same simulated opponent. Historical intervals cluster by game;
other intervals cluster by original deal, not by seat replay.

On fresh mixed matches opponent moons fell from **1,475 to 1,419**, while own
moons stayed at 639. Mean play time was 17.27 vs. 17.09 ms per hand during
concurrent runs; this is not an isolated latency benchmark. The other four
comparisons are inconclusive: they neither establish human-playing-strength
gains nor prove equivalence. Five additional charging-rule checks found no clear
loss, but used only 100 deals each. The human corpus contains only Classic games.

Development comparisons are in [SCREENING.md](SCREENING.md), with all JSON
results retained. The early claim prototype used unconditional `can_claim`;
its `claim-v1-*` results are historical only. The final variant uses the server's
`should_claim` eligibility, including not voluntarily claiming avoidable
penalties after a moon is broken. The initial ace screen/follow-up files used
the claim prototype; `ace-replay-corrected-claims.json` records its corrected
variants. Non-claim profiles are unaffected.

## Other implementations explored

- `plan`: compare avoidance, shooting, and a separate own preventive-defense
  continuation. Preserve a supported master before a threat appears, then try
  to take or supply a point after the first opposing heart. Plans are averaged
  across all sampled deals before maximizing; no hidden-deal-specific choice
  between plans is permitted. This adds up to a third rollout per candidate.
- `claims`: recognize provable claims at trick boundaries using only the
  simulated actor's hand and public information. Score every remaining card
  through the real scoring engine. No omniscient opponent ownership is passed
  to the claim check.
- `early`: allow the existing fallback shooter mixture after one captured
  heart rather than three. The queen trigger and 50% mixture remain.
- `preserve`: the bounded root preference; its default experimental margin is
  4, and an explicit final parameter selects 8.
- `humanmodel` / `humanmix`: use a fitted human action policy for all / half of
  opponent continuations, respectively. These change simulated play, not hidden
  ownership. Opponent selection uses a separate RNG stream. They apply to every
  opponent seat in this research setting, not persistent player identities.
- `safe` = preserve + early; `safeclaims` = preserve + claims;
  `safeearly` = preserve + claims + early; `planclaims` and `earlyclaims` combine
  the named features. `humanlike` combines plan + claims + early (it does not
  load the fitted human model). `safemix` combines preserve + claims + humanmix.

The fitted model and more aggressive changes remain research options. Learning
human behavior improved held-out action prediction, but did not consistently
improve play across opponent populations. On historical validation hands the
full fitted model gained 9.57 [-0.71, 19.85] against its own claiming population,
but lost 10.41 [-21.22, 0.41] against full-strength search. These are exploratory
screens, not independent evidence of human playing strength.

## Human data and model

The read-only export found 1,699 completed games with human players, all Classic
rules, ending in July 2026. Chats, identities, game IDs and seeds are removed
from generated data. Whole games are split chronologically 70/15/15, keeping all
four hands together. One training game contains no non-forced human plays.

| Split | Games with non-forced human actions | Decisions | Fitted NLL | Fixed noisy policy-prior NLL | Uniform NLL |
| --- | ---: | ---: | ---: | ---: | ---: |
| Train | 1,188 | 153,808 | 1.2863 | 1.4402 | 1.4373 |
| Validation | 255 | 32,487 | 1.3008 | 1.4567 | 1.4525 |
| Test | 255 | 29,644 | 1.3102 | 1.4633 | 1.4456 |

Lower negative log likelihood is better. The comparator is the fixed
90% heuristic / 5% duck / 5% shooter prior with 25% uniform action noise,
not the production bot's complete adaptive history inference.

The model is a softmax gate over five normalized action distributions: uniform,
heuristic, duck, shooter, and preventive defender. Eight actor-information
features condition the gate: intercept, own moon viability, captured hearts,
queen ownership, an established opposing threat, held heart fraction, a master
heart indicator, and public game progress. It contains 40 fitted coefficients,
not a value network. Validation selected epoch 71 of 100; the test block was
scored afterward. Rust/NumPy agreement and likelihood gradients are checked.
The coefficients and training metadata are in `assets/models/human-policy.*`.
They are compiled in for optional experiments; there is no runtime model file.

The temporal split measures later games of this population, not unseen-player
identity generalization. Passing and charging are not learned. The `humanclaim`
opponent adds the existing provable-claim rule to the fitted action model.
Simulated opponents are approximations of humans, including their mistakes.

## Historical evaluation

`historical_arena` starts at the first play of each recorded hand, preserving
the humans' actual passes and charges. Both algorithms see their own cards and
redacted histories. Subsequent moves come from simulated opponents: recorded
human actions are never replayed after changing the game. Every seat is tested,
and confidence intervals cluster all hands and seat replays by original game.
This tests human-shaped starting positions; it is not a counterfactual claim
about what the original humans would have done.

## Reproduction

```sh
cargo build --locked --release -p turbo-hearts-bot --examples
python3 scripts/export_human_games.py turbo-hearts.db data/round4/human-games.jsonl
target/release/examples/human_data data/round4/human-games.jsonl data/round4/human-actions.bin
OPENBLAS_NUM_THREADS=1 python3 train_human_policy.py data/round4/human-actions.bin assets/models/human-policy.rs
# Rebuild after fitting the compiled coefficients.
cargo build --locked --release -p turbo-hearts-bot --examples

target/release/examples/replay_decision bot/tests/fixtures/ace-slough.json 37 preserve:128:12:none:0:32:8 100
target/release/examples/arena preserve:128:12:none:0:32:8 cautiousdynamic:128:12:none:0:32:0:0.6 mixed 2000 180000
target/release/examples/historical_arena data/round4/human-games.jsonl preserve:128:12:none:0:32:8 cautiousdynamic:128:12:none:0:32:0:0.6 humanclaim 255 2

cargo test --locked --workspace --all-targets
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy
```

The replay example reports privileged claim diagnostics separately from search
values. Those diagnostics never enter action selection. Generated event and
training data remain under ignored `data/`; the source database is unchanged.

## Validation

`cargo test --locked --workspace --all-targets` passed after enabling the new
default, including example compilation and benchmark smoke checks. Both Python
training test modules passed (four numerical/export tests). Regression coverage
includes redacted pass knowledge, legal and normalized human probabilities,
Rust/NumPy coefficient agreement, bounded stopper preference across seeds,
claim scoring, all charging rules, and deterministic replay. The existing
unrelated server dead-code warning remains.

Production parity is checked against the explicitly configured candidate on
200 mixed and 200 claiming-model deals, including all four seats. The production
constructor also retains AH in all 100 replays of the reported decision. These
checks confirm wiring; they are not additional independent strength evidence.
