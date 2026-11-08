-- Immutable observer-only cases, durable exposures, and append-only revisions.
CREATE TABLE judgment_case (
    id TEXT PRIMARY KEY, group_id TEXT NOT NULL, kind TEXT NOT NULL,
    direction TEXT NOT NULL, payload TEXT NOT NULL, queue_order INTEGER NOT NULL
);
CREATE TABLE judgment_assignment (
    user_id TEXT NOT NULL, case_id TEXT NOT NULL REFERENCES judgment_case(id),
    group_id TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    PRIMARY KEY(user_id, case_id), UNIQUE(user_id, group_id)
);
CREATE INDEX judgment_assignment_case ON judgment_assignment(case_id);
CREATE TABLE judgment_response (
    id INTEGER PRIMARY KEY, case_id TEXT NOT NULL, user_id TEXT NOT NULL,
    revision INTEGER NOT NULL, status TEXT NOT NULL CHECK(status IN ('answered','skipped')),
    payload TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    UNIQUE(user_id, case_id, revision),
    FOREIGN KEY(user_id, case_id) REFERENCES judgment_assignment(user_id, case_id)
);
