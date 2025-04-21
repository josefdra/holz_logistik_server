use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::location::location_tables::{
    LocationSawmillJunctionTable, LocationTable,
};
use chrono::{DateTime, Utc};
use rusqlite::{Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub done: bool,
    pub started: bool,
    pub last_edit: String,
    pub latitude: f64,
    pub longitude: f64,
    pub partie_nr: String,
    pub date: String,
    pub additional_info: String,
    pub initial_quantity: f64,
    pub initial_oversize_quantity: f64,
    pub initial_piece_count: i32,
    pub current_quantity: f64,
    pub current_oversize_quantity: f64,
    pub current_piece_count: i32,
    pub contract_id: String,
    pub sawmill_ids: Vec<String>,
    pub oversize_sawmill_ids: Vec<String>,
}

impl Location {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "done": if self.done { 1 } else { 0 },
            "started": if self.started { 1 } else { 0 },
            "lastEdit": self.last_edit,
            "latitude": self.latitude,
            "longitude": self.longitude,
            "partieNr": self.partie_nr,
            "date": self.date,
            "additionalInfo": self.additional_info,
            "initialQuantity": self.initial_quantity,
            "initialOversizeQuantity": self.initial_oversize_quantity,
            "initialPieceCount": self.initial_piece_count,
            "currentQuantity": self.current_quantity,
            "currentOversizeQuantity": self.current_oversize_quantity,
            "currentPieceCount": self.current_piece_count,
            "contractId": self.contract_id,
            "sawmillIds": self.sawmill_ids,
            "oversizeSawmillIds": self.oversize_sawmill_ids,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let done_val = json.get("done").and_then(|v| v.as_i64()).unwrap_or(0);

        let started_val = json.get("started").and_then(|v| v.as_i64()).unwrap_or(0);

        let sawmill_ids = json
            .get("sawmillIds")
            .and_then(|v| {
                if let serde_json::Value::Array(arr) = v {
                    Some(
                        arr.iter()
                            .filter_map(|id| id.as_str().map(|s| s.to_string()))
                            .collect::<Vec<String>>(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let oversize_sawmill_ids = json
            .get("oversizeSawmillIds")
            .and_then(|v| {
                if let serde_json::Value::Array(arr) = v {
                    Some(
                        arr.iter()
                            .filter_map(|id| id.as_str().map(|s| s.to_string()))
                            .collect::<Vec<String>>(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(Location {
            id: json
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            done: done_val != 0,
            started: started_val != 0,
            last_edit: json
                .get("lastEdit")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            latitude: json.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0),
            longitude: json
                .get("longitude")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            partie_nr: json
                .get("partieNr")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            date: json
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            additional_info: json
                .get("additionalInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            initial_quantity: json
                .get("initialQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            initial_oversize_quantity: json
                .get("initialOversizeQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            initial_piece_count: json
                .get("initialPieceCount")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            current_quantity: json
                .get("currentQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            current_oversize_quantity: json
                .get("currentOversizeQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            current_piece_count: json
                .get("currentPieceCount")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            contract_id: json
                .get("contractId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sawmill_ids,
            oversize_sawmill_ids,
        })
    }

    pub fn copy_with(
        &self,
        sawmill_ids: Option<Vec<String>>,
        oversize_sawmill_ids: Option<Vec<String>>,
    ) -> Self {
        let mut new_location = self.clone();

        if let Some(ids) = sawmill_ids {
            new_location.sawmill_ids = ids;
        }

        if let Some(ids) = oversize_sawmill_ids {
            new_location.oversize_sawmill_ids = ids;
        }

        new_location
    }
}

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
            "SELECT {} FROM {} WHERE {} = ? AND {} = ?",
            LocationSawmillJunctionTable::COLUMN_SAWMILL_ID,
            LocationSawmillJunctionTable::TABLE_NAME,
            LocationSawmillJunctionTable::COLUMN_LOCATION_ID,
            LocationSawmillJunctionTable::COLUMN_IS_OVERSIZE
        );

        let conn = self.core_storage.get_connection();
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

    pub fn get_location_updates_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<Location>> {
        let query = format!(
            "SELECT * FROM {} WHERE {} >= ?",
            LocationTable::TABLE_NAME,
            LocationTable::COLUMN_LAST_EDIT,
        );

        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(params![last_edit.to_rfc3339()], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        })?;

        let mut location_ids = Vec::new();
        for row in rows {
            match row {
                Ok(id) => location_ids.push(id),
                Err(e) => eprintln!("Error fetching location ID: {}", e),
            }
        }

        let mut locations = Vec::new();
        for id in location_ids {
            match self.get_location_by_id(&id) {
                Ok(location) => locations.push(location),
                Err(e) => eprintln!("Error fetching location: {}", e),
            }
        }

        Ok(locations)
    }

    pub fn get_location_by_id(&self, id: &str) -> Result<Location> {
        let location_json = self.core_storage.get_by_id(LocationTable::TABLE_NAME, id)?;

        if location_json.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        match Location::from_json(&location_json[0]) {
            Ok(mut location) => {
                let sawmill_ids = self.get_sawmill_ids(id, false)?;
                let oversize_sawmill_ids = self.get_sawmill_ids(id, true)?;

                location = location.copy_with(Some(sawmill_ids), Some(oversize_sawmill_ids));

                Ok(location)
            }
            Err(e) => Err(rusqlite::Error::InvalidParameterName(format!(
                "Error parsing location: {}",
                e
            ))),
        }
    }

    fn insert_location_sawmill_junction(
        &self,
        location_id: &str,
        sawmill_id: &str,
        is_oversize: bool,
    ) -> Result<i64> {
        let junction_data = serde_json::json!({
            LocationSawmillJunctionTable::COLUMN_LOCATION_ID: location_id,
            LocationSawmillJunctionTable::COLUMN_SAWMILL_ID: sawmill_id,
            LocationSawmillJunctionTable::COLUMN_IS_OVERSIZE: if is_oversize { 1 } else { 0 },
        });

        self.core_storage
            .insert(LocationSawmillJunctionTable::TABLE_NAME, &junction_data)
    }

    fn insert_or_update_location(&self, location: &Location) -> Result<i64> {
        self.core_storage.delete_by_column(
            LocationSawmillJunctionTable::TABLE_NAME,
            LocationSawmillJunctionTable::COLUMN_LOCATION_ID,
            &location.id,
        )?;

        for sawmill_id in &location.sawmill_ids {
            self.insert_location_sawmill_junction(&location.id, sawmill_id, false)?;
        }

        for sawmill_id in &location.oversize_sawmill_ids {
            self.insert_location_sawmill_junction(&location.id, sawmill_id, true)?;
        }

        let mut location_data = location.to_json();
        if let serde_json::Value::Object(ref mut map) = location_data {
            map.remove("sawmillIds");
            map.remove("oversizeSawmillIds");
        }

        self.core_storage
            .insert_or_update(LocationTable::TABLE_NAME, &location_data)
    }

    pub fn save_location(&self, location: &Location) -> Result<i64> {
        let result = self.insert_or_update_location(location)?;

        Ok(result)
    }
}
