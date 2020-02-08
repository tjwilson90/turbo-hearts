use crate::{
    error::CardsError,
    game::GameEvent,
    types::{ChargingRules, GameId, Player},
};
use r2d2::{CustomizeConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, DropBehavior, Transaction, TransactionBehavior, NO_PARAMS};
use std::{collections::HashMap, time::Duration};
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
            CREATE TABLE IF NOT EXISTS game (
                id TEXT NOT NULL,
                PRIMARY KEY (id)
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

    pub fn hydrate_games(
        &self,
    ) -> Result<HashMap<GameId, HashMap<Player, ChargingRules>>, CardsError> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare(
            "SELECT game_id, event FROM event
            WHERE event_id = 0 AND game_id NOT IN (SELECT id FROM game)",
        )?;
        let mut rows = stmt.query(NO_PARAMS)?;
        let mut games = HashMap::new();
        while let Some(row) = rows.next()? {
            let id = row.get(0)?;
            if let GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
            } = serde_json::from_str(&row.get::<_, String>(1)?)?
            {
                let mut players = HashMap::new();
                players.insert(north, rules);
                players.insert(east, rules);
                players.insert(south, rules);
                players.insert(west, rules);
                games.insert(id, players);
            }
        }
        Ok(games)
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

    fn run_sql<F, T>(&self, behavior: TransactionBehavior, mut f: F) -> Result<T, CardsError>
    where
        F: FnMut(Transaction) -> Result<T, CardsError>,
    {
        task::block_in_place(|| {
            let mut conn = self.pool.get().unwrap();
            for i in 0.. {
                let result = conn
                    .transaction_with_behavior(behavior)
                    .map(|mut tx| {
                        tx.set_drop_behavior(DropBehavior::Commit);
                        tx
                    })
                    .map_err(|err| CardsError::from(err))
                    .and_then(&mut f);
                match result {
                    Err(e) if i < 5 && e.is_retriable() => continue,
                    v => return v,
                }
            }
            unreachable!()
        })
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
