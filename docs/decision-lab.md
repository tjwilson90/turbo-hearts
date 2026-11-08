# Decision lab

Deploy the server and invite players to `/judge`, also linked from the lobby. The
page uses the site's normal login. It asks what they would pass, charge, or play,
then lets them save and move on. It accepts other reasonable choices, confidence,
strategic intentions, a suspected moon runner, and an explanation. Everything
except the preferred choice is optional; players can skip questions and revise
recent answers. Card-play questions include a suit-by-suit remaining-card
overview distinguishing your hand, unplayed cards elsewhere, and played cards.
The table and trick history use fixed compass positions with lead markers and
numbered cards, including both plays per seat in extended tricks.

These recommendations add information game logs cannot provide: what players
believed, what tradeoffs they considered, and which alternatives they consider
close. They are **expert recommendations**, not proof an action wins more money.
The page explains that responses are saved with their account for analysis.

## Deploy

Build and deploy the Rust server normally (`cargo build --locked --release -p
turbo-hearts`). The review page's HTML, CSS, JavaScript, and starter questions are
embedded in the binary; there is no extra service, runtime Python dependency, or
frontend build for `/judge`. Rebuild the lobby frontend as usual to include its
new link (`cd packages/lobby && yarn build`). The direct `/judge` URL works
without updating the lobby.

On startup, the server upgrades the database at `db_path` from schema version 1
to 2, then imports the bundled questions idempotently. Fresh databases apply both
migrations. Existing game/user tables and feedback are preserved. As with any
database upgrade, take a SQLite backup before deploying.

`external_uri` must match the site's public origin, for example
`https://play.anti.run`. The review API checks the browser's Origin against it,
including when the application runs behind a reverse proxy. The proxy must pass
`/judge` and `/judge/*` to the Rust server. No new credentials are needed.

The starter pool contains 240 positions (80 each for passing, charging, and play)
from 82 completed games in the training partitions of `turbo-hearts.db` and
`th2.db`. Positions cover left, right, across, and keeper rounds; charging covers
the rules and phases represented in those games. Sampling balances task/round
categories and mixes policy disagreements with random controls. Selection
metadata and policy choices remain in the database for offline analysis.

Each account gets at most one position from a source **game**, including positions
it merely opened. This avoids learning another hand or future outcome across
questions; later-hand scores could reveal earlier-hand outcomes too. The starter
pool therefore offers up to 82 distinct questions per account, with different
accounts reviewing different positions. Filters can exhaust earlier. Pending
questions resume after refresh; switching filters may expose another game.

## Storage and bringing results back

The application database contains all three tables:

- `judgment_case`: immutable question, observer context, source split, and sampling
  metadata. This is enough to reconstruct the input for later model experiments.
- `judgment_assignment`: account, question, source group, and first exposure time.
- `judgment_response`: account, answer metadata, timestamps, and every revision.

Responses use the verified login UserId, so they can later be joined to the same
account's game performance to study strong players. Browser-supplied reviewer
identities are rejected. Reviewers can access only their own assigned positions
and answers. The API does not return hidden hands, future actions, source game
IDs, policy suggestions, or other reviewers' labels. Blind charges remain
redacted according to the engine's observer rules.

Bring back a **SQLite backup** of the application's database. Do not copy only
the live `.db` file while WAL writes may still be pending. For example:

```sh
sqlite3 /path/to/turbo-hearts.db ".backup '/path/to/feedback-backup.db'"
python3 scripts/judgment_data.py stats --database /path/to/feedback-backup.db
python3 scripts/judgment_data.py export data/judgments/labels.jsonl \
  --database /path/to/feedback-backup.db
```

The offline utility reads the application database read-only. Export keeps the
latest revision of answered questions, excluding recognized hands by default,
and consistently pseudonymizes reviewer IDs. `--include-recognized` includes
recognized hands; `--raw-user-ids` permits joining responses to historical player
performance. The full DB backup preserves revision history and those joins.
The export retains source train/validation/test splits; keep whole source games
in their original split when training. Do not train on held-out judgments.

## Add questions

Generate questions offline from completed games, then package them:

```sh
python3 scripts/judge.py prepare --database /path/to/feedback-backup.db \
  --max-games 300 --count 600 --seed 43
python3 scripts/judgment_data.py pack data/judgments/more-questions.jsonl
```

`prepare` builds/runs the Rust `judgment_cases` example and reads source game
DBs read-only. It needs Cargo and Python 3; no Python packages. It defaults to the
oldest 70% of eligible games per source database, reserving the rest for
evaluation. Existing prepared questions are preserved. If changing the sampling
population would change a game's original split, preserve the original split
before using the labels in training. `--include-evaluation` is for evaluation
collection only and retains those split labels.

Place the JSONL on the server **outside the public `assets` directory**, set
`"judgment_cases_path": "/private/path/more-questions.jsonl"` in `config.json`,
and restart. The file adds to the bundled pool. Existing question IDs and all
responses remain intact; changed contents under an existing ID are rejected.
Use a new version/ID for a revised question and preserve its `group_id` so the
exposure limit still holds. New source games extend how much each account can
review. The question file and raw generation intermediates are administrator
inputs, never public static assets.

The standalone local workbench remains available with
`python3 scripts/judge.py serve --open`; it uses a separate local store and does
not authenticate other players. Use the deployed `/judge` page for collection.

## Using the labels

Analyze agreement, confidence, explanations, and alternate choices by decision,
round, and reviewer performance. Compare strong players' recommendations with
bot choices, then test candidate policy changes in held-out games and responsive
matches against human-like opponents. Keep observed game actions, judgment
labels, and rollout rewards as separate targets. The decision lab does not
change the deployed policy or train it automatically.

## Checks

```sh
cargo test --locked --workspace --all-targets
python3 -m unittest scripts.test_judgment_data
node --check scripts/judgment_ui/app.js

# Optional isolated DOM simulation (no browser, test-only dependency):
npm install --prefix /tmp/hearts-ui-test --ignore-scripts --no-audit --no-fund jsdom@27.0.1
NODE_PATH=/tmp/hearts-ui-test/node_modules node --test scripts/test_judgment_ui.cjs
```

The server tests cover both database migration paths, idempotent/atomic imports,
observer redaction, authenticated HTTP routes, cross-site rejection, user
isolation, legal choices, explicit no-charge choices, revision conflicts,
concurrent assignments/submissions, and durable resume. The Python tests cover
read-only exports, latest-revision selection, recognition/skip filtering, and
question packaging.

The DOM checks render all 240 starter positions and exercise passing, charging,
play, alternatives, saving, editing, skipping, and revisiting an empty queue.
They do not verify browser layout; visual desktop/mobile QA remains necessary.
