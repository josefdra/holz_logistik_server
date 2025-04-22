use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::photo::photo_tables::PhotoTable;
use chrono::{DateTime, Utc};
use rusqlite::{Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: String,
    pub last_edit: String,
    pub photo_file: Vec<u8>,
    pub location_id: String,
}

impl Photo {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "photoFile": self.photo_file,
            "locationId": self.location_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Photo {
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
            photo_file: json
                .get("photoFile")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u8))
                        .collect::<Vec<u8>>()
                })
                .unwrap_or_default(),
            location_id: json
                .get("locationId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

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

    pub fn get_photo_updates_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<Photo>> {
        let query = format!(
            "SELECT * FROM {} WHERE {} >= ?",
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_LAST_EDIT,
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(params![last_edit.to_rfc3339()], |row| {
            let id: String = row.get(0)?;
            let last_edit: String = row.get(1)?;
            let photo_file: Vec<u8> = row.get(2)?;
            let location_id: String = row.get(3)?;

            Ok(Photo {
                id,
                last_edit,
                photo_file,
                location_id,
            })
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

    pub fn save_photo(&self, photo: &Photo) -> Result<i64> {
        let conn = self.core_storage.get_connection()?;
        let query = format!(
            "INSERT OR REPLACE INTO {} ({}, {}, {}, {}) VALUES (?, ?, ?, ?)",
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_ID,
            PhotoTable::COLUMN_LAST_EDIT,
            PhotoTable::COLUMN_PHOTO,
            PhotoTable::COLUMN_LOCATION_ID
        );

        let result = conn.execute(
            &query,
            params![
                photo.id,
                photo.last_edit,
                photo.photo_file,
                photo.location_id
            ],
        )?;

        Ok(result as i64)
    }
}
