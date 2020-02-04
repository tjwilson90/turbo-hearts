use crate::error::CardsError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, ErrorCode, Transaction, TransactionBehavior};
use std::time::Duration;
use tokio::task;
use warp::{reject, Rejection};

#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new() -> Self {
        let manager = SqliteConnectionManager::file("cards.db");
        let pool = Pool::new(manager).unwrap();
        Database::seed(&pool.get().unwrap()).unwrap();
        Self { pool }
    }

    fn seed(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            BEGIN;
            CREATE TABLE IF NOT EXISTS game (
                id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                north TEXT NOT NULL,
                east TEXT NOT NULL,
                south TEXT NOT NULL,
                west TEXT NOT NULL,
                rules TEXT NOT NULL,
                PRIMARY KEY (id)
            );
            CREATE TABLE IF NOT EXISTS event (
                game_id TEXT NOT NULL,
                hand_id INTEGER NOT NULL,
                event_id INTEGER NOT NULL,
                seat INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                event TEXT NOT NULL,
                PRIMARY KEY (game_id, event_id)
            );
            END;",
        )
    }

    pub fn run_read_only<F, T>(&self, f: F) -> Result<T, Rejection>
    where
        F: FnMut(Transaction) -> Result<T, rusqlite::Error>,
    {
        self.run_sql(TransactionBehavior::Deferred, f)
    }

    pub fn run_with_retry<F, T>(&self, f: F) -> Result<T, Rejection>
    where
        F: FnMut(Transaction) -> Result<T, rusqlite::Error>,
    {
        self.run_sql(TransactionBehavior::Immediate, f)
    }

    fn run_sql<F, T>(&self, behavior: TransactionBehavior, mut f: F) -> Result<T, Rejection>
    where
        F: FnMut(Transaction) -> Result<T, rusqlite::Error>,
    {
        task::block_in_place(|| {
            let mut conn = self.pool.get().unwrap();
            conn.busy_timeout(Duration::from_secs(5)).unwrap();
            loop {
                match conn.transaction_with_behavior(behavior).and_then(&mut f) {
                    Err(rusqlite::Error::SqliteFailure(e, _))
                        if e.code == ErrorCode::DatabaseBusy
                            || e.code == ErrorCode::DatabaseLocked =>
                    {
                        continue
                    }
                    Ok(v) => return Ok(v),
                    v => return v,
                }
            }
        })
        .map_err(|err: rusqlite::Error| reject::custom(CardsError::from(err)))
    }
}
