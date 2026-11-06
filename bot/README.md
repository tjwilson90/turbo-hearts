# Bot development

The existing `neural_net` lobby strategy and `NeuralNetworkBot` API now use
sampled search with explicit moon planning. They work without the four lost
ONNX files. No model path or Python installation is needed to play.

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

Opponent rollouts also include a possible shooter: when an opponent has all
captured moon points and at least three hearts or the queen, half the rollouts
use the shooting policy for that opponent. This lets search value taking a heart
to break an opposing moon. This is an opponent-model assumption, not a claim
that the opponent's intent is known. Rollout policies only see their own hand
and public information. Sampled search still has the usual limitations of a
fixed rollout policy and hidden-card determinization; it is not optimal play.

Passing and charging retain the existing heuristic. Searching all three-card
passes did not improve the development matches reliably, so that experiment
was removed. A larger play budget also failed to show a clear gain once moon
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
- `defend:N:E`: also model opposing moon threats; `defend:128:12` is the default.
- `heuristic`, `duck`, `gottatry`, `random`: comparison policies. The `random`
  profile randomizes plays and uses heuristic passes/charges.
- `mixed` as the opponent: heuristic, duck, and GottaTry at the other three seats.

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
stdout; progress goes to stderr. Timing covers card decisions, excluding deal,
pass, and model loading. `queen_and_8_to_12_hearts` is a near-miss proxy, not an
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

For application code, use `NeuralNetworkBot::with_config(config, seed)` and,
optionally, `.with_policy(PolicyNetwork::load(path)?, prior_samples)`.
`evaluate` exposes sampled move values for further experiments. External model
loading validates format, dimensions, length, and finite weights and returns an
error for missing or invalid files.

## Checks

```sh
cargo test --locked --workspace --all-targets
cargo bench --locked -p turbo-hearts-bot --bench inference
```

Regression tests cover uniform constrained deals, private observations,
repeatability and legal games across all six charging rules and four pass
directions, soft-budget fallback, certain versus impossible moons, abandoning
broken moon attempts, nine control, and Rust/NumPy inference agreement.
