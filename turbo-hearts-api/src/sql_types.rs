use crate::{BotStrategy, ChargingRules, GameEvent, GameId, Seat, Seed, UserId};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};

macro_rules! sql_str {
    ($t:ty) => {
        impl ToSql for $t {
            fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
                Ok(ToSqlOutput::Owned(Value::Text(self.to_string())))
            }
        }

        impl FromSql for $t {
            fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
                match value.as_str() {
                    Ok(value) => Ok(value.parse().unwrap()),
                    Err(e) => Err(e),
                }
            }
        }
    };
}

macro_rules! sql_json {
    ($t:ty) => {
        impl ToSql for $t {
            fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
                let json = serde_json::to_string(self).unwrap();
                Ok(ToSqlOutput::Owned(Value::Text(json)))
            }
        }

        impl FromSql for $t {
            fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
                value.as_str().map(|s| serde_json::from_str(s).unwrap())
            }
        }
    };
}

sql_str!(GameId);
sql_str!(UserId);
sql_json!(ChargingRules);
sql_json!(GameEvent);
sql_json!(Seat);
sql_json!(Seed);
sql_json!(BotStrategy);
