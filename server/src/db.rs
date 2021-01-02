use crate::CardsError;
use r2d2::{CustomizeConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    Connection, DropBehavior, Row, ToSql, Transaction, TransactionBehavior, NO_PARAMS,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, path::Path, str::FromStr, time::Duration};
use tokio::task;
use turbo_hearts_api::{BotStrategy, ChargingRules, GameEvent, GameId, Seat, Seed, UserId};

static SQL: &[&'static str] = &[include_str!("../sql/schema.sql")];

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let manager = SqliteConnectionManager::file(path);
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

pub struct SqlStr<T>(T);

pub trait ToSqlStr {
    fn sql(&self) -> SqlStr<&Self>;
}

macro_rules! sql_str {
    ($t:ty) => {
        impl ToSqlStr for $t {
            fn sql(&self) -> SqlStr<&$t> {
                SqlStr(self)
            }
        }
    };
}

sql_str!(GameId);
sql_str!(UserId);

impl<T> ToSql for SqlStr<T>
where
    T: ToString,
{
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Text(self.0.to_string())))
    }
}

impl<T> FromSql for SqlStr<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| SqlStr(s.parse().unwrap()))
    }
}

pub trait GetStr {
    fn get_str<T>(&self, idx: usize) -> Result<T, rusqlite::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug;
}

impl<'stmt> GetStr for Row<'stmt> {
    fn get_str<T>(&self, idx: usize) -> Result<T, rusqlite::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        let str: SqlStr<T> = self.get(idx)?;
        Ok(str.0)
    }
}

pub struct SqlJson<T>(T);

pub trait ToSqlJson {
    fn sql(&self) -> SqlJson<&Self>;
}

macro_rules! sql_json {
    ($t:ty) => {
        impl ToSqlJson for $t {
            fn sql(&self) -> SqlJson<&$t> {
                SqlJson(self)
            }
        }
    };
}

sql_json!(BotStrategy);
sql_json!(ChargingRules);
sql_json!(GameEvent);
sql_json!(Seat);
sql_json!(Seed);

impl<T> ToSql for SqlJson<T>
where
    T: Serialize,
{
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(&self.0).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl<T> FromSql for SqlJson<T>
where
    T: DeserializeOwned,
{
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value
            .as_str()
            .map(|s| SqlJson(serde_json::from_str(s).unwrap()))
    }
}

pub trait GetJson {
    fn get_json<T>(&self, idx: usize) -> Result<T, rusqlite::Error>
    where
        T: DeserializeOwned;
}

impl<'stmt> GetJson for Row<'stmt> {
    fn get_json<T>(&self, idx: usize) -> Result<T, rusqlite::Error>
    where
        T: DeserializeOwned,
    {
        let json: SqlJson<T> = self.get(idx)?;
        Ok(json.0)
    }
}
