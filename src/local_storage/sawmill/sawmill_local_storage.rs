use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::sawmill::sawmill_tables::SawmillTable;
use chrono::{DateTime, Utc};
use rusqlite::{Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sawmill {
    pub id: String,
    pub last_edit: String,
    pub name: String,
}

impl Sawmill {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "name": self.name,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Sawmill {
            id: json
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            last_edit: json
                .get("lastEdit")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

pub struct SawmillLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl SawmillLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = SawmillLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    pub fn get_sawmill_updates_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<Sawmill>> {
        let query = format!(
            "SELECT * FROM {} WHERE lastEdit >= ?",
            SawmillTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(params![last_edit.to_rfc3339()], |row| {
            let id: String = row.get(0)?;
            let last_edit: String = row.get(1)?;
            let name: String = row.get(2)?;

            Ok(Sawmill {
                id,
                last_edit,
                name,
            })
        })?;

        let mut sawmills = Vec::new();
        for row in rows {
            match row {
                Ok(sawmill) => sawmills.push(sawmill),
                Err(e) => eprintln!("Error fetching sawmill: {}", e),
            }
        }

        Ok(sawmills)
    }

    pub fn save_sawmill(&self, sawmill: &Sawmill) -> Result<i64> {
        let json_data = sawmill.to_json();
        let result = self
            .core_storage
            .insert_or_update(SawmillTable::TABLE_NAME, &json_data)?;

        Ok(result)
    }
}
