use rusqlite::{params, Connection, Result};
use serde_json;
use base64::prelude::*;
use std::sync::Mutex;

pub struct CoreLocalStorage {
    connection: Mutex<Connection>,
}

impl CoreLocalStorage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(CoreLocalStorage { connection: Mutex::new(conn) })
    }

    pub fn get_connection(&self) -> Result<std::sync::MutexGuard<Connection>> {
        self.connection.lock().map_err(|_| rusqlite::Error::ExecuteReturnedResults)
    }
    
    pub fn get_by_id(&self, table_name: &str, id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.get_connection()?;
        let query = format!("SELECT * FROM {} WHERE id = ?", table_name);
        let mut stmt = conn.prepare(&query)?;

        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|name| name.to_string())
            .collect();

        let rows = stmt.query_map(params![id], |row| {
            let mut map = serde_json::Map::new();
            for (i, column_name) in column_names.iter().enumerate() {
                let value = self.get_value_from_row(row, i)?;
                map.insert(column_name.to_string(), value);
            }
            Ok(serde_json::Value::Object(map))
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            if let Ok(row_value) = row_result {
                results.push(row_value);
            }
        }

        Ok(results)
    }

    fn get_value_from_row(&self, row: &rusqlite::Row, index: usize) -> Result<serde_json::Value> {
        let column_type = row.get_ref(index)?.data_type();

        match column_type {
            rusqlite::types::Type::Null => Ok(serde_json::Value::Null),
            rusqlite::types::Type::Integer => {
                let val: i64 = row.get(index)?;
                Ok(serde_json::Value::Number(val.into()))
            }
            rusqlite::types::Type::Real => {
                let val: f64 = row.get(index)?;
                if let Some(n) = serde_json::Number::from_f64(val) {
                    Ok(serde_json::Value::Number(n))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            rusqlite::types::Type::Text => {
                let val: String = row.get(index)?;
                Ok(serde_json::Value::String(val))
            }
            rusqlite::types::Type::Blob => {
                let val: Vec<u8> = row.get(index)?;
                let encoded = BASE64_STANDARD.encode(&val);
                Ok(serde_json::Value::String(encoded))
            }
        }
    }

    pub fn insert(&self, table_name: &str, data: &serde_json::Value) -> Result<i64> {
        if let serde_json::Value::Object(map) = data {
            let conn = self.get_connection()?;
            let columns: Vec<String> = map.keys().cloned().collect();
            let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();

            let column_str = columns.join(", ");
            let placeholder_str = placeholders.join(", ");

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table_name, column_str, placeholder_str
            );

            let mut stmt = conn.prepare(&query)?;
            let mut param_values = Vec::new();

            for col in &columns {
                if let Some(value) = map.get(col) {
                    param_values.push(json_to_param(value));
                }
            }

            stmt.execute(rusqlite::params_from_iter(param_values))?;
            Ok(conn.last_insert_rowid())
        } else {
            Err(rusqlite::Error::InvalidParameterName(
                "Data must be a JSON object".to_string(),
            ))
        }
    }

    pub fn update(&self, table_name: &str, data: &serde_json::Value) -> Result<usize> {
        if let serde_json::Value::Object(map) = data {
            let conn = self.get_connection()?;
            if !map.contains_key("id") {
                return Err(rusqlite::Error::InvalidParameterName(
                    "Data must contain an 'id' field".to_string(),
                ));
            }

            let id = map.get("id").unwrap();
            let mut updates = Vec::new();
            let mut param_values = Vec::new();

            for (key, value) in map {
                if key != "id" {
                    updates.push(format!("{} = ?", key));
                    param_values.push(json_to_param(value));
                }
            }

            param_values.push(json_to_param(id));

            let update_str = updates.join(", ");
            let query = format!("UPDATE {} SET {} WHERE id = ?", table_name, update_str);

            let mut stmt = conn.prepare(&query)?;
            let rows_affected = stmt.execute(rusqlite::params_from_iter(param_values))?;
            Ok(rows_affected)
        } else {
            Err(rusqlite::Error::InvalidParameterName(
                "Data must be a JSON object".to_string(),
            ))
        }
    }

    pub fn insert_or_update(&self, table_name: &str, data: &serde_json::Value) -> Result<i64> {
        if let serde_json::Value::Object(map) = data {
            if !map.contains_key("id") {
                return Err(rusqlite::Error::InvalidParameterName(
                    "Data must contain an 'id' field".to_string(),
                ));
            }

            let id = map.get("id").unwrap().as_str().unwrap_or("");
            let existing = self.get_by_id(table_name, id)?;

            if !existing.is_empty() {
                self.update(table_name, data)?;
                Ok(0)
            } else {
                self.insert(table_name, data)
            }
        } else {
            Err(rusqlite::Error::InvalidParameterName(
                "Data must be a JSON object".to_string(),
            ))
        }
    }

    pub fn delete_by_column(
        &self,
        table_name: &str,
        column_name: &str,
        value: &str,
    ) -> Result<usize> {
        let conn = self.get_connection()?;
        let query = format!("DELETE FROM {} WHERE {} = ?", table_name, column_name);
        conn.execute(&query, params![value])
    }
}

fn json_to_param(value: &serde_json::Value) -> Box<dyn rusqlite::ToSql> {
    match value {
        serde_json::Value::Null => Box::new(Option::<String>::None),
        serde_json::Value::Bool(b) => Box::new(*b),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Box::new(n.as_i64().unwrap())
            } else if n.is_f64() {
                Box::new(n.as_f64().unwrap())
            } else {
                Box::new(Option::<String>::None)
            }
        }
        serde_json::Value::String(s) => Box::new(s.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Box::new(serde_json::to_string(value).unwrap_or_default())
        }
    }
}
