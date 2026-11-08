# Bot development

The existing `neural_net` lobby strategy and `NeuralNetworkBot` API now use
sampled search with explicit moon planning. They work without the four lost
ONNX files. No model path or Python installation is needed to play.

The [successful-human update](evaluation/round7/README.md) enables a contextual
opponent policy fitted to consistently successful human players. It models
publicly identified human seats while retaining confident inferred styles,
including runners. The [supported-master-heart preference](evaluation/round4/README.md),
[selective style inference](evaluation/round3/README.md), and emerging-shooter
continuations remain. The report includes paired matches, uncertainty against
stronger responders, and comparisons of actual human decisions to bot choices.
These offline results use simulated responses, not live-human counterfactuals.
The [seven-direction exploration](evaluation/round2/README.md) and its standalone
networks remain available for research.
The [early-signal experiments](evaluation/round6/README.md) add an all-suit human
charging model, early control-retention evidence, and preventive heart/nine
discard continuations as separately selectable research profiles.
The [capacity and history experiments](evaluation/round8/README.md) compare
larger human policies, card and public-history inputs, persistent simulated
plans, and a payoff-trained head. These are opt-in research models; card play
retains the successful-human fallback described above. Better prediction of
human choices did not produce a consistent payoff gain at equal decision time.

The [engine and preplay update](evaluation/round9/README.md) reduces fixed-search
computation by about 21.5% and enables a learned passing policy **only in Classic
games against three human opponents**, on left/right/across rounds. Two rounds
of population training improved that setting, but changing every bot together
regressed at mixed tables. Keeper passing, charging, other rules, and passing
with multiple bots retain their existing policies. The bundled checkpoint
requires no runtime model path. More search and learned charging remain
experimental because their match results did not justify promotion.

The defaults are 128 hidden deals per decision, exact search with at most 12
cards remaining, and a one-second soft time limit. Most decisions finish much
sooner. Every candidate sees the same deals and random rollout streams.
`HandSampler` samples uniformly over deals consistent with known cards, passes,
revealed charges, claims, and public voids. It caches completion counts for the
whole decision, rather than taking the first feasible backtracking assignment.

For each move, search compares two continuations: the existing point-avoidance
heuristic and a dedicated shooting policy. Each continuation is averaged over
*all* sampled deals before choosing between them. Failed attempts receive their
actual terminal penalty, including charges and the ten of clubs. Once an
opponent has captured a heart or the queen, the shooting continuation stops.
The bot reconsiders the plan at each real decision; it does not commit to a run
just because it has already taken points.

Style inference starts with a conservative prior: 90% heuristic, 5% duck,
and 5% shooting. It adopts a non-heuristic style only after three informative
observations and at least 60% support under that model. Otherwise, the original
opposing-shooter mixture applies. In half the rollouts, an opponent can begin
shooting once it alone holds captured moon points and has three hearts or the
queen. This opportunity is reconsidered as the simulated hand develops, so a
threat need not already exist at the initial position. The shooting continuation
stops when another player wins a moon point.

For a publicly identified human without a confident inferred style, the default
uses the fitted successful-human continuation policy instead of that fallback
mixture. Context includes suit support, masters, nines, voids, moon viability,
and charges. It does not infer a person's skill from their identity. Bot seats
retain the previous continuation model. Models and trainers for a more detailed
direct action predictor are also available, but that predictor is not the default.

These are opponent-model assumptions, not known intentions. Inference uses a
separate random stream and uniformly constrained deals; it does not bias hidden
ownership by behavioral likelihood. Policies see their own hand and public
information. Search still has the limitations of approximate continuations and
hidden-card determinization; it is not optimal play.

When sloughing onto another player's trick, the bot prefers to keep its current
highest heart if it beats all unseen hearts, has at least two supporting hearts,
and an opponent can still moon. It accepts at most 8 money less in modeled value
than the best move. This bounded preference preserves inexpensive control before
captured points reveal a threat; a sufficiently better discard still wins.
`evaluate` exposes unmodified search values; `play` applies the preference.

Passing and charging retain the existing heuristic. Searching all three-card
passes in the first round did not improve development matches reliably, so
that experiment was removed. The second round adds a different opt-in joint
pass/charge planner, with results linked above. A larger play budget also failed to show a clear gain once moon
planning was added. The learned prior remains opt-in until it earns its place
in match results.

## Reproduce a comparison

Run from the repository root:

```sh
cargo run --locked --release -p turbo-hearts-bot --example arena -- \
  default search:128:12 heuristic 2000 50000
```

Arguments are candidate, baseline, opponent, number of deals, first seed, and
optional charging rules. `--help` lists the options. Profiles:

- `default`: the production configuration with the time cap disabled for repeatability.
- `search:N:E`: N samples, exact search at E remaining cards, ordinary rollouts.
- `moon:N:E`: also compare the bot's shooting continuation.
- `defend:N:E`: the frozen first-round opposing-shooter model. `defend:128:12`
  remains the benchmark for the current default.
- `skilledfallback`: the current production policy, with the time cap disabled.
- `round7[:N:E:TIME_MS]`: frozen successful-human production policy. `coherent`
  uses the same arguments and samples a persistent human component per rollout.
- `neural:MODEL[:N:E:TIME_MS]`, `neuralmix:MODEL[:N:E:TIME_MS]`: use a THH1
  human policy for all or half of eligible human continuations.
- `payoff:MODEL[:N:E:TIME_MS:WEIGHT]`: blend a THH1 monetary advantage head
  into play selection (default weight 0.25). Raw `evaluate` values stay unchanged.
- `neuralplayer:MODEL`, `neuralhybrid:MODEL[:N:E:TIME_MS]`: learned responders
  with claims; the hybrid uses frozen round7 search half the time.
  These new profiles omit the time cap unless `TIME_MS` is supplied (`none`
  explicitly disables it). A high
  `N` and explicit cap support comparisons at equal soft decision-time limits.
- `round4` or `preserve:128:12:none:0:32:8`: the frozen supported-stopper policy.
- `cautiousdynamic:128:12:none:0:32:0:0.6`: the frozen round-three default.
- `human`, `humanclaim`: the fitted human action model, without/with provable
  claims. These are simulated opponents, not real human players.
- `chargebelief`, `intent`, `disrupt`, `humanaware`: experimental additions to
  frozen round4, with the usual `:N:E` sample/endgame suffix. Respectively enable
  learned charge-conditioned hidden hands, soft early shooting inference, a
  preventive continuation, or all three. Charging inference is human-only,
  Classic non-keeper; its likelihood power defaults to 0.5 and can be set with
  `chargebelief:128:12:none:0:32:POWER`.
- `skilledmix`, `skilledmodel`, `skilledintent`, `skilledclaims`, `skilledfallback`,
  `skilledadaptive`: successful-human continuation variants. Respectively use a
  half mixture, full mixture, half plus early intent, half plus claims, full
  fallback retaining inferred styles, or that fallback plus human early intent.
- `actionmix`, `actionmodel`, `actionintent`, `actionclaims`, `actionplan`:
  direct human move-model variants, with half/full human continuations, early
  intent, claims, or an additional own learned continuation.
- `skilled`, `action`: fitted policies used directly as simulated players.
  Their `claim` variants can make provable claims. `skilledhybrid` and
  `actionhybrid` mix each policy with frozen round4 search at every decision.
- `heuristic`, `duck`, `gottatry`, `random`: comparison policies. The `random`
  profile randomizes plays and uses heuristic passes/charges.
- `mixed` as the opponent: heuristic, duck, and GottaTry at the other three seats.

The arena also exposes `model`, `belief`, `posterior`, `tree`, `modeltree`,
`preplay`, `pass`, `charge`, `combined`, `modelcharge`, `posteriorcharge`,
`allocate`, and `guided`, using the same `:N:E` suffix. Their definitions and
reproduction commands are in the [round-two report](evaluation/round2/README.md).
Direct network players use `policy:PATH:TEMPERATURE` (zero means greedy).
Follow-up profiles and their additional parameters are documented in the
[round-three report](evaluation/round3/README.md), the
[human-play report](evaluation/round4/README.md), and `arena --help`.

Append `:MODEL:PRIOR_SAMPLES` to a search profile to test a learned prior.
Every unique deal is replayed with the candidate in all four seats, then with
the baseline in those same seats. Opponent cards and passes are redacted through
the real game event API. Deals, passes, charging, hidden-hand sampling, and
policy randomness are seeded. The runner asserts that all played cards are
legal and exercises keeper redistribution through the real seed API.

The main metric is average **money per hand**, higher is better. It includes
charged moons and all other scoring through `WonState::scores`. Confidence
intervals use paired differences clustered by original deal, not eight
supposedly independent seat replays. Win shares split ties. JSON output goes to
stdout; progress goes to stderr. `ms_per_hand` covers card decisions;
`preplay_ms_per_hand` separately covers passes and charges. Both exclude deal
and model loading. `queen_and_8_to_12_hearts` is a near-miss proxy, not an
observed count of deliberate failed attempts.

See [evaluation/README.md](evaluation/README.md) for experiments and held-out
results. Development seeds are 1000–1999; the final evaluation starts at 50000.
Training and validation deals are separate from both sets.

## Train and evaluate a neural prior

The replacement network predicts each legal move's advantage using only the
acting player's hand and public state. It is a 256-input, 64-ReLU, scalar-output
MLP, with a versioned float32 format and a small Rust inference implementation.
It does not reconstruct final scores by multiplying independently predicted
heart/queen/ten probabilities, which would miss their correlations and the
nonlinear moon payoff.

```sh
python3 -m venv .venv
.venv/bin/pip install -r requirements-training.txt
mkdir -p data
cargo run --locked --release -p turbo-hearts-bot --example policy_data -- \
  data/moon-policy.bin 4000 20000 64 defend
OPENBLAS_NUM_THREADS=1 .venv/bin/python neural_net.py \
  data/moon-policy.bin data/moon-policy.thp --epochs 45
cargo run --locked --release -p turbo-hearts-bot --example arena -- \
  defend:128:12:data/moon-policy.thp:32 defend:128:12 heuristic 1000 1000
```

The generator uses the real engine, runs search for every non-forced decision,
and explores a random distinct move 15% of the time. A row contains deal ID,
decision ID, 256 features, and centered money advantage divided by 100, all
little-endian float32. Rows from an entire deal stay in the same split:
`deal_id % 10 == 0` is validation. The trainer uses Huber loss and Adam, and saves
the epoch with lowest validation decision regret. Validation regret measures
agreement with the search teacher; use the arena to assess playing strength.

The checked-in checkpoints are `assets/models/policy-avoid.thp` and
`assets/models/policy-moon.thp`, with matching JSON metadata. Keep both together in version
control. The old `trainer` binary and TFRecord converter remain available for
historical perfect-information datasets; the current script consumes the new
`policy_data` format directly and does not require TensorFlow or ONNX.

For production application code, use `NeuralNetworkBot::new()` or the seeded
`NeuralNetworkBot::with_seed(seed)`. To customize budgets, use
`NeuralNetworkBot::with_config(config, seed)` followed by
`.with_experiments(SearchExperiments::production()).with_production_preplay()`.
`with_production_preplay()` installs the bundled passing head and its confirmed
rule/table/direction restrictions. `with_config` alone
preserves the frozen baseline for reproducible experiments. A calibrated neural
prior can optionally be supplied through `.with_policy(network, prior_samples)`.
`evaluate` exposes sampled move values for further experiments. External model
loading validates format, dimensions, length, and finite weights and returns an
error for missing or invalid files.

Use `.with_experiments(SearchExperiments { ... })` before replaying history
(including `Sit`, which supplies public human/bot player types) to
enable research variants. `SearchExperiments::default()` turns all extensions
off; `production()` selects the default subset. `round_seven()`, `round_four()`, and `round_three()`
preserve earlier defaults. `.with_samples(n)` preserves
observations when cloning a bot for a stronger teacher. Diagnostics expose the
last search's terminal evaluations, belief effective sample size, and tree nodes.
The standalone self-play checkpoints contain policy scores, so they should not
be supplied as calibrated monetary priors through `with_policy`.

`with_human_network(HumanNetwork)` selects a THH1 policy for enabled human
rollouts; pair it with `SearchExperiments::round_seven()` to restrict those
rollouts to public human seats and retain inferred runner styles.
`with_payoff_network(HumanNetwork, weight)` accepts only a monetary advantage
checkpoint and blends it before the stopper preference, outside exact endgames
and only when another public human seat is present. Both remain experimental.

`with_preplay_models(pass, charge, margin)` selects optional TPQ1 monetary
models for passing and charging. These score all 286 legal pass triples or all
available charge subsets, including declining. The margin is the predicted
money improvement required to replace the existing heuristic. The inputs
condition on pass direction, charging rules, public charges/declines, and both
keeper charging stages; they contain only the acting player's information.
Passing and charging heads can be tested independently with
`prelearn:PASS_MODEL:CHARGE_MODEL[:SAMPLES:EXACT:TIME_MS:MARGIN:PASS_MASK:HUMAN_RECIPIENTS]`
in the arena. Use `none` for a disabled head or an omitted time cap. The optional
passing mask defaults to 15; bits 1/2/4/8 enable left/right/across/keeper.
`with_preplay_pass_directions([bool; 4])` exposes the same control in Rust.
`HUMAN_RECIPIENTS=1` or `with_preplay_human_recipients(true)` retains heuristic
passing when the destination is publicly identified as a bot. `preplay_table`
tests simultaneous changes to every bot at mixed tables with accurate public
human/bot roles; the ordinary arena changes one player and marks all seats human.

Generate responsive policy-improvement targets with:

```sh
cargo run --release -p turbo-hearts-bot --example preplay_data -- \
  data/preplay.bin 12000 1800000 round7:8:8 2 16
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py \
  data/preplay.bin assets/models/preplay-pass.tpq pass
OPENBLAS_NUM_THREADS=1 python3 train_preplay.py \
  data/preplay.bin assets/models/preplay-charge.tpq charge
```

The generator cycles through all four directions and six rules. Its arguments
are output, deals, seed, learner profile, continuation repeats, number of pass
candidates, and optional archived opponent profiles. Every candidate is played
to completion with responsive opponents and normal event redaction. Later
iterations can use a `prelearn` learner and add archived policies to the
population; `train_preplay.py --initial MODEL` continues a head on pooled data.
Split whole deals, retain independent match seeds, and evaluate full games:
target loss alone is insufficient to establish stronger play.

## Checks

```sh
cargo test --locked --workspace --all-targets
cargo bench --locked -p turbo-hearts-bot --bench inference
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training test_human_policy test_historical_data test_human_neural test_preplay
```

Regression tests cover uniform constrained deals, private observations,
repeatability and legal games across all six charging rules and four pass
directions, soft-budget fallback, certain versus impossible moons, abandoning
broken moon attempts, nine control, and Rust/NumPy inference agreement.
