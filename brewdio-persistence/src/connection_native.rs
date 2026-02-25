#![cfg(feature = "native")]

use crate::connection::{run_migrations, Connection, DbError, Row, Value};

/// Open a native SQLite database and run migrations.
pub fn open(path: &str) -> Result<rusqlite::Connection, DbError> {
    let conn = rusqlite::Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Owned value extracted from a rusqlite row (since rusqlite::Row can't outlive the closure).
enum OwnedValue {
    Text(String),
    Blob(Vec<u8>),
    Int(i32),
    Null,
}

/// A row with pre-extracted column values.
struct NativeRow {
    values: Vec<OwnedValue>,
}

impl Row for NativeRow {
    fn get_text(&self, idx: usize) -> String {
        match &self.values[idx] {
            OwnedValue::Text(s) => s.clone(),
            OwnedValue::Int(i) => i.to_string(),
            _ => String::new(),
        }
    }

    fn get_optional_text(&self, idx: usize) -> Option<String> {
        match &self.values[idx] {
            OwnedValue::Text(s) => Some(s.clone()),
            OwnedValue::Null => None,
            _ => None,
        }
    }

    fn get_blob(&self, idx: usize) -> Vec<u8> {
        match &self.values[idx] {
            OwnedValue::Blob(b) => b.clone(),
            _ => Vec::new(),
        }
    }

    fn get_bool(&self, idx: usize) -> bool {
        match &self.values[idx] {
            OwnedValue::Int(i) => *i != 0,
            _ => false,
        }
    }
}

/// Convert a Value to a rusqlite-compatible type for parameter binding.
fn to_rusqlite_value<'a>(v: &'a Value<'a>) -> Box<dyn rusqlite::types::ToSql + 'a> {
    match v {
        Value::Text(s) => Box::new(*s),
        Value::OptionalText(opt) => match opt {
            Some(s) => Box::new(*s),
            None => Box::new(rusqlite::types::Null),
        },
        Value::Blob(b) => Box::new(*b),
        Value::Int(i) => Box::new(*i),
        Value::Bool(b) => Box::new(*b),
    }
}

impl Connection for rusqlite::Connection {
    fn execute(&self, sql: &str, params: &[Value]) -> Result<(), DbError> {
        let boxed: Vec<Box<dyn rusqlite::types::ToSql + '_>> =
            params.iter().map(to_rusqlite_value).collect();
        let refs: Vec<&dyn rusqlite::types::ToSql> = boxed.iter().map(|b| b.as_ref()).collect();
        self.execute(sql, refs.as_slice())?;
        Ok(())
    }

    fn execute_batch(&self, sql: &str) -> Result<(), DbError> {
        rusqlite::Connection::execute_batch(self, sql)?;
        Ok(())
    }

    fn query_map<T, F>(&self, sql: &str, params: &[Value], mut f: F) -> Result<Vec<T>, DbError>
    where
        F: FnMut(&dyn Row) -> T,
    {
        let boxed: Vec<Box<dyn rusqlite::types::ToSql + '_>> =
            params.iter().map(to_rusqlite_value).collect();
        let refs: Vec<&dyn rusqlite::types::ToSql> = boxed.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.prepare(sql)?;
        let col_count = stmt.column_count();

        let mut results = Vec::new();
        let mut rows = stmt.query(refs.as_slice())?;
        while let Some(row) = rows.next()? {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                use rusqlite::types::ValueRef;
                let val = match row.get_ref(i)? {
                    ValueRef::Text(b) => {
                        OwnedValue::Text(String::from_utf8_lossy(b).into_owned())
                    }
                    ValueRef::Blob(b) => OwnedValue::Blob(b.to_vec()),
                    ValueRef::Integer(n) => OwnedValue::Int(n as i32),
                    ValueRef::Real(f) => OwnedValue::Int(f as i32),
                    ValueRef::Null => OwnedValue::Null,
                };
                values.push(val);
            }
            let native_row = NativeRow { values };
            results.push(f(&native_row));
        }
        Ok(results)
    }
}
