use crate::error::CardsError;
use r2d2::{CustomizeConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, DropBehavior, Transaction, TransactionBehavior, NO_PARAMS};
use std::time::Duration;
use tokio::task;

static SQL: &[&'static str] = &[include_str!("../sql/schema.sql")];

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(manager: SqliteConnectionManager) -> Result<Self, CardsError> {
        let pool = Pool::builder()
            .connection_customizer(Box::new(Customizer))
            .build(manager)
            .unwrap();
        let conn = pool.get().unwrap();
        let version = conn.query_row("PRAGMA user_version", NO_PARAMS, |row| {
            Ok(row.get::<_, i64>(0)? as usize)
        })?;
        if version < SQL.len() {
            if version == 0 {
                conn.execute_batch(SQL[0])?;
            } else {
                for upgrade in SQL[version..].iter() {
                    conn.execute_batch(upgrade)?;
                }
            }
            conn.execute_batch(&format!("PRAGMA user_version = {}", SQL.len()))?;
        }
        Ok(Self { pool })
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
