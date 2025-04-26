use crate::local_storage::core_local_storage::CoreLocalStorage;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

pub struct LocationLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl LocationLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = LocationLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    fn get_sawmill_ids(&self, id: &str, is_oversize: bool) -> Result<Vec<String>> {
        let query = format!(
            "SELECT sawmillId FROM locationSawmillJunction WHERE locationId = ? AND isOversize = ?"
        );

        let conn = self.core_storage.get_connection()?;

        let mut stmt = conn.prepare(&query)?;
        let is_oversize_val = if is_oversize { 1 } else { 0 };

        let rows = stmt.query_map(params![id, is_oversize_val], |row| {
            let sawmill_id: String = row.get(0)?;
            Ok(sawmill_id)
        })?;

        let mut sawmill_ids = Vec::new();
        for row in rows {
            match row {
                Ok(id) => sawmill_ids.push(id),
                Err(e) => eprintln!("Error fetching sawmill ID: {}", e),
            }
        }

        Ok(sawmill_ids)
    }

    pub fn get_location_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let location_ids = {
            let query = format!(
                "SELECT id FROM locations WHERE arrivalAtServer > ? ORDER BY lastEdit ASC",
            );

            let conn = self.core_storage.get_connection()?;
            let mut stmt = conn.prepare(&query)?;

            let rows = stmt.query_map(params![last_edit], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?;

            let mut ids = Vec::new();
            for row in rows {
                match row {
                    Ok(id) => ids.push(id),
                    Err(e) => eprintln!("Error fetching location ID: {}", e),
                }
            }
            ids
        };

        let mut locations = Vec::new();
        for (_, id) in location_ids.iter().enumerate() {
            match self.get_location_by_id(id) {
                Ok(location) => locations.push(location),
                Err(e) => eprintln!("Error fetching location {}: {}", id, e),
            }
        }

        Ok(locations)
    }

    pub fn get_location_by_id(&self, id: &str) -> Result<Value> {
        let location_json = self.core_storage.get_by_id("locations", id)?;

        if location_json.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let mut location_data = location_json[0].clone();
        let sawmill_ids = self.get_sawmill_ids(id, false)?;
        let oversize_sawmill_ids = self.get_sawmill_ids(id, true)?;

        if let serde_json::Value::Object(ref mut map) = location_data {
            map.insert(
                "sawmillIds".to_string(),
                serde_json::Value::Array(
                    sawmill_ids
                        .into_iter()
                        .map(|id| serde_json::Value::String(id))
                        .collect(),
                ),
            );
            map.insert(
                "oversizeSawmillIds".to_string(),
                serde_json::Value::Array(
                    oversize_sawmill_ids
                        .into_iter()
                        .map(|id| serde_json::Value::String(id))
                        .collect(),
                ),
            );
        }

        Ok(location_data)
    }

    fn insert_location_sawmill_junction(
        &self,
        location_id: &str,
        sawmill_id: &str,
        is_oversize: bool,
    ) -> Result<i64> {
        let junction_data = serde_json::json!({
            "locationId": location_id,
            "sawmillId": sawmill_id,
            "isOversize": if is_oversize { 1 } else { 0 },
        });

        self.core_storage
            .insert("locationSawmillJunction", &junction_data)
    }

    pub fn save_location(&self, location_data: &Value) -> Result<bool> {
        let location_id = location_data["id"].as_str().unwrap_or("");

        self.core_storage.delete_by_column(
            "locationSawmillJunction",
            "locationId",
            location_id,
        )?;

        if let Some(sawmill_ids) = location_data["sawmillIds"].as_array() {
            for sawmill_value in sawmill_ids {
                if let Some(sawmill_id) = sawmill_value.as_str() {
                    self.insert_location_sawmill_junction(location_id, sawmill_id, false)?;
                }
            }
        }

        if let Some(sawmill_ids) = location_data["oversizeSawmillIds"].as_array() {
            for sawmill_value in sawmill_ids {
                if let Some(sawmill_id) = sawmill_value.as_str() {
                    self.insert_location_sawmill_junction(location_id, sawmill_id, true)?;
                }
            }
        }

        let mut location_for_save = location_data.clone();
        if let serde_json::Value::Object(ref mut map) = location_for_save {
            map.remove("sawmillIds");
            map.remove("oversizeSawmillIds");
            map.insert("arrivalAtServer".to_string(), chrono::Utc::now().timestamp_millis().into());
        }

        self.core_storage
            .insert_or_update("locations", &location_for_save)
    }
}
