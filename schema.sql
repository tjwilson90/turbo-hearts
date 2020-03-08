PRAGMA journal_mode = WAL;
BEGIN;

CREATE TABLE IF NOT EXISTS user (
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    realm TEXT NOT NULL,
    external_id TEXT NOT NULL,
    PRIMARY KEY (user_id)
) WITHOUT ROWID;
CREATE UNIQUE INDEX IF NOT EXISTS idx_user ON user (realm, external_id);

CREATE TABLE IF NOT EXISTS auth_token (
    token TEXT NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (token)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS lobby_chat (
    timestamp INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    message TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lobby_chat ON lobby_chat (timestamp);

CREATE TABLE IF NOT EXISTS game (
    game_id TEXT NOT NULL,
    seed TEXT NOT NULL,
    created_time INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    last_updated_time INTEGER NOT NULL,
    last_updated_by TEXT NOT NULL,
    started_time INTEGER,
    completed_time INTEGER,
    PRIMARY KEY (game_id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS idx_game_completed ON game(game_id) WHERE completed_time IS NULL;
CREATE INDEX IF NOT EXISTS idx_game_completed_time ON game(completed_time) WHERE completed_time IS NOT NULL;

CREATE TABLE IF NOT EXISTS game_player (
    game_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    strategy TEXT,
    rules TEXT NOT NULL,
    seat TEXT,
    PRIMARY KEY (game_id, user_id)
) WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS event (
    game_id TEXT NOT NULL,
    event_id INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    event TEXT NOT NULL,
    PRIMARY KEY (game_id, event_id)
) WITHOUT ROWID;

END;
