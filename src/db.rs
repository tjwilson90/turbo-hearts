use crate::error::CardsError;
use r2d2::{CustomizeConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, DropBehavior, Transaction, TransactionBehavior};
use std::time::Duration;
use tokio::task;

#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(manager: SqliteConnectionManager) -> Result<Self, CardsError> {
        let pool = Pool::builder()
            .connection_customizer(Box::new(Customizer))
            .build(manager)
            .unwrap();
        Database::seed(&pool.get().unwrap())?;
        Ok(Self { pool })
    }

    fn seed(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            BEGIN;
            CREATE TABLE IF NOT EXISTS user (
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                realm TEXT NOT NULL,
                external_id TEXT NOT NULL,
                PRIMARY KEY (user_id)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_user ON user (realm, external_id);
            CREATE TABLE IF NOT EXISTS auth_token (
                token TEXT NOT NULL,
                user_id TEXT NOT NULL,
                PRIMARY KEY (token)
            );
            CREATE TABLE IF NOT EXISTS game (
                game_id TEXT NOT NULL,
                PRIMARY KEY (game_id)
            );
            CREATE TABLE IF NOT EXISTS event (
                game_id TEXT NOT NULL,
                event_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                event TEXT NOT NULL,
                PRIMARY KEY (game_id, event_id)
            );
            END;",
        )
    }

    pub fn run_blocking_read_only<F, T>(&self, f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        self.run_blocking(TransactionBehavior::Deferred, f)
    }

    pub fn run_read_only<F, T>(&self, f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        self.run_sql(TransactionBehavior::Deferred, f)
    }

    pub fn run_with_retry<F, T>(&self, f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        self.run_sql(TransactionBehavior::Immediate, f)
    }

    fn run_sql<F, T>(&self, behavior: TransactionBehavior, f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        task::block_in_place(|| self.run_blocking(behavior, f))
    }

    fn run_blocking<F, T>(&self, behavior: TransactionBehavior, mut f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        let mut conn = self.pool.get().unwrap();
        for i in 0.. {
            let result = conn
                .transaction_with_behavior(behavior)
                .map(|mut tx| {
                    tx.set_drop_behavior(DropBehavior::Commit);
                    tx
                })
                .map_err(|e| e.into())
                .and_then(&mut f);
            match result {
                Err(e) if i < 5 && e.is_retriable() => continue,
                v => return v,
            }
        }
        unreachable!()
    }
}

#[derive(Debug)]
struct Customizer;

impl CustomizeConnection<Connection, rusqlite::Error> for Customizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(())
    }

    fn on_release(&self, _: Connection) {}
}
