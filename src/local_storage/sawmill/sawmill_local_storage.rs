use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::sawmill::sawmill_tables::SawmillTable;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

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

    pub fn get_sawmill_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM {} WHERE deleted = 0 AND lastEdit > ? ORDER BY lastEdit ASC",
            SawmillTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let last_edit: i64 = row.get(1)?;
            let name: String = row.get(2)?;

            let sawmill_json = serde_json::json!({
                "id": id,
                "lastEdit": last_edit,
                "name": name,
            });

            Ok(sawmill_json)
        })?;

        let mut sawmills = Vec::new();
        for row in rows {
            match row {
                Ok(sawmill) => {
                    sawmills.push(sawmill);
                }
                Err(e) => eprintln!("Error fetching sawmill: {}", e),
            }
        }

        Ok(sawmills)
    }

    pub fn save_sawmill(&self, sawmill_data: &Value) -> Result<i64> {
        let result = self
            .core_storage
            .insert_or_update(SawmillTable::TABLE_NAME, sawmill_data)?;

        Ok(result)
    }
}
