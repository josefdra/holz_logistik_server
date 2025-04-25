use crate::local_storage::core_local_storage::CoreLocalStorage;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

pub struct PhotoLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl PhotoLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = PhotoLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    pub fn get_photo_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM photos WHERE deleted = 0 AND arrivalAtServer > ? ORDER BY lastEdit ASC",
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let last_edit: i64 = row.get(1)?;
            let photo_file: Vec<u8> = row.get(2)?; 
            let location_id: String = row.get(3)?;
            let arrival_at_server: i64 = row.get(4)?;

            let photo_json = serde_json::json!({
                "id": id,
                "lastEdit": last_edit,
                "photoFile": photo_file,
                "locationId": location_id,
                "arrivalAtServer": arrival_at_server
            });

            Ok(photo_json)
        })?;

        let mut photos = Vec::new();
        for row in rows {
            match row {
                Ok(photo) => photos.push(photo),
                Err(e) => eprintln!("Error fetching photo: {}", e),
            }
        }

        Ok(photos)
    }

    pub fn save_photo(&self, photo_data: &Value) -> Result<i64> {
        let id = photo_data["id"].as_str().unwrap_or_default();
        let last_edit = photo_data["lastEdit"].as_i64().unwrap_or(0);
        let photo_file = match &photo_data["photoFile"] {
            Value::Array(arr) => {
                let bytes: Vec<u8> = arr.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                bytes
            },
            _ => Vec::new(),
        };
        let location_id = photo_data["locationId"].as_str().unwrap_or_default();
        let arrival_at_server = chrono::Utc::now().timestamp_millis();
        
        let conn = self.core_storage.get_connection()?;
        let query = format!(
            "INSERT OR REPLACE INTO photos (id, lastEdit, photoFile, locationId, arrivalAtServer) VALUES (?, ?, ?, ?, ?)",
        );

        let result = conn.execute(
            &query,
            params![
                id,
                last_edit,
                photo_file,
                location_id,
                arrival_at_server
            ],
        )?;

        Ok(result as i64)
    }
}
