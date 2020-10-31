#[macro_export]
macro_rules! sql_str {
    ($t:ty) => {
        impl rusqlite::types::ToSql for $t {
            fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
                Ok(rusqlite::types::ToSqlOutput::Owned(
                    rusqlite::types::Value::Text(self.to_string()),
                ))
            }
        }

        impl rusqlite::types::FromSql for $t {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> Result<Self, rusqlite::types::FromSqlError> {
                match value.as_str() {
                    Ok(value) => Ok(value.parse().unwrap()),
                    Err(e) => Err(e),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! sql_json {
    ($t:ty) => {
        impl rusqlite::types::ToSql for $t {
            fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
                let json = serde_json::to_string(self).unwrap();
                Ok(rusqlite::types::ToSqlOutput::Owned(
                    rusqlite::types::Value::Text(json),
                ))
            }
        }

        impl rusqlite::types::FromSql for $t {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> Result<Self, rusqlite::types::FromSqlError> {
                value.as_str().map(|s| serde_json::from_str(s).unwrap())
            }
        }
    };
}
