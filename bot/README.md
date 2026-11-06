# Bot development

The existing `neural_net` lobby strategy and `NeuralNetworkBot` API now use
sampled search with explicit moon planning. They work without the four lost
ONNX files. No model path or Python installation is needed to play.

The [follow-up experiments](evaluation/round3/README.md) produced the current
default: selective opponent modeling and continuations that recognize emerging
moon attempts. Fresh matches improved money against mixed and heuristic bots;
the full-strength search comparison remained inconclusive. The earlier
[seven-direction exploration](evaluation/round2/README.md) and its trained
standalone networks remain available as research variants.

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

Opponent modeling starts with a conservative prior: 90% heuristic, 5% duck,
and 5% shooting. It adopts a non-heuristic style only after three informative
observations and at least 60% support under that model. Otherwise, the original
opposing-shooter mixture applies. In half the rollouts, an opponent can begin
shooting once it alone holds captured moon points and has three hearts or the
queen. This opportunity is reconsidered as the simulated hand develops, so a
threat need not already exist at the initial position. The shooting continuation
stops when another player wins a moon point.

These are opponent-model assumptions, not known intentions. Inference uses a
separate random stream and uniformly constrained deals; it does not bias hidden
ownership by behavioral likelihood. Policies see their own hand and public
information. Search still has the limitations of approximate continuations and
hidden-card determinization; it is not optimal play.

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
- `cautiousdynamic:128:12:none:0:32:0:0.6`: the current production policy.
- `heuristic`, `duck`, `gottatry`, `random`: comparison policies. The `random`
  profile randomizes plays and uses heuristic passes/charges.
- `mixed` as the opponent: heuristic, duck, and GottaTry at the other three seats.

The arena also exposes `model`, `belief`, `posterior`, `tree`, `modeltree`,
`preplay`, `pass`, `charge`, `combined`, `modelcharge`, `posteriorcharge`,
`allocate`, and `guided`, using the same `:N:E` suffix. Their definitions and
reproduction commands are in the [round-two report](evaluation/round2/README.md).
Direct network players use `policy:PATH:TEMPERATURE` (zero means greedy).
Follow-up profiles and their additional parameters are documented in the
[round-three report](evaluation/round3/README.md) and `arena --help`.

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
`.with_experiments(SearchExperiments::production())`. `with_config` alone
preserves the frozen baseline for reproducible experiments. A calibrated neural
prior can optionally be supplied through `.with_policy(network, prior_samples)`.
`evaluate` exposes sampled move values for further experiments. External model
loading validates format, dimensions, length, and finite weights and returns an
error for missing or invalid files.

Use `.with_experiments(SearchExperiments { ... })` before the first deal to
enable research variants. `SearchExperiments::default()` turns all extensions
off; `production()` selects the confirmed subset. `.with_samples(n)` preserves
observations when cloning a bot for a stronger teacher. Diagnostics expose the
last search's terminal evaluations, belief effective sample size, and tree nodes.
The standalone self-play checkpoints contain policy scores, so they should not
be supplied as calibrated monetary priors through `with_policy`.

## Checks

```sh
cargo test --locked --workspace --all-targets
cargo bench --locked -p turbo-hearts-bot --bench inference
OPENBLAS_NUM_THREADS=1 python3 -m unittest test_training
```

Regression tests cover uniform constrained deals, private observations,
repeatability and legal games across all six charging rules and four pass
directions, soft-budget fallback, certain versus impossible moons, abandoning
broken moon attempts, nine control, and Rust/NumPy inference agreement.
