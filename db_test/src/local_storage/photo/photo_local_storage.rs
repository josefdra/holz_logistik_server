use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::photo::photo_tables::PhotoTable;
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use base64::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: String,
    pub last_edit: String,
    pub photo_file: Vec<u8>,
    pub location_id: String,
}

impl Photo {
    pub fn new(photo_file: Vec<u8>, location_id: String) -> Self {
        Photo {
            id: Uuid::new_v4().to_string(),
            last_edit: Utc::now().to_rfc3339(),
            photo_file,
            location_id,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        // Encode the binary photo data as base64
        let photo_base64 = BASE64_STANDARD.encode(&self.photo_file);
        
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "photoFile": photo_base64,
            "locationId": self.location_id,
        })
    }
    
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        // Decode the base64 photo data
        let photo_base64 = json.get("photoFile").and_then(|v| v.as_str()).unwrap_or("");
        let photo_file = BASE64_STANDARD.decode(photo_base64).unwrap_or_default();
        
        Ok(Photo {
            id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            last_edit: json.get("lastEdit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            photo_file,
            location_id: json.get("locationId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
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
    
    pub fn get_photos_by_location(&self, location_id: &str) -> Result<Vec<Photo>> {
        let json_values = self.core_storage.get_by_column(
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_LOCATION_ID,
            location_id
        )?;
        
        let mut photos = Vec::new();
        for json_value in json_values {
            match Photo::from_json(&json_value) {
                Ok(photo) => photos.push(photo),
                Err(e) => eprintln!("Error parsing photo: {}", e),
            }
        }
        
        Ok(photos)
    }
    
    pub fn get_photo_ids_by_location(&self, location_id: &str) -> Result<Vec<String>> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            PhotoTable::COLUMN_ID,
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_LOCATION_ID
        );
        
        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(
            params![location_id],
            |row| {
                let id: String = row.get(0)?;
                Ok(id)
            }
        )?;
        
        let mut photo_ids = Vec::new();
        for row in rows {
            match row {
                Ok(id) => photo_ids.push(id),
                Err(e) => eprintln!("Error fetching photo ID: {}", e),
            }
        }
        
        Ok(photo_ids)
    }
    
    pub fn check_if_photo_exists(&self, photo_id: &str) -> Result<bool> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            PhotoTable::COLUMN_ID,
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_ID
        );
        
        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(
            params![photo_id],
            |row| {
                let _: String = row.get(0)?;
                Ok(())
            }
        )?;
        
        let mut exists = false;
        for row in rows {
            if row.is_ok() {
                exists = true;
                break;
            }
        }
        
        Ok(exists)
    }
    
    pub fn save_photo(&self, photo: &Photo) -> Result<i64> {
        let json_data = photo.to_json();
        self.core_storage.insert_or_update(
            PhotoTable::TABLE_NAME,
            &json_data
        )
    }
    
    pub fn delete_photo(&self, id: &str, _location_id: &str) -> Result<usize> {
        self.core_storage.delete(
            PhotoTable::TABLE_NAME,
            id
        )
    }
    
    pub fn delete_photos_by_location_id(&self, location_id: &str) -> Result<usize> {
        self.core_storage.delete_by_column(
            PhotoTable::TABLE_NAME,
            PhotoTable::COLUMN_LOCATION_ID,
            location_id
        )
    }
}
