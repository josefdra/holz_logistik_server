use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::location::location_tables::{LocationTable, LocationSawmillJunctionTable};
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use uuid::Uuid;

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
    pub fn new(contract_id: String) -> Self {
        Location {
            id: Uuid::new_v4().to_string(),
            done: false,
            started: false,
            last_edit: Utc::now().to_rfc3339(),
            latitude: 0.0,
            longitude: 0.0,
            partie_nr: String::new(),
            date: Utc::now().to_rfc3339(),
            additional_info: String::new(),
            initial_quantity: 0.0,
            initial_oversize_quantity: 0.0,
            initial_piece_count: 0,
            current_quantity: 0.0,
            current_oversize_quantity: 0.0,
            current_piece_count: 0,
            contract_id,
            sawmill_ids: Vec::new(),
            oversize_sawmill_ids: Vec::new(),
        }
    }
    
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
        let done_val = json.get("done")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
            
        let started_val = json.get("started")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
            
        let sawmill_ids = json.get("sawmillIds")
            .and_then(|v| {
                if let serde_json::Value::Array(arr) = v {
                    Some(arr.iter()
                        .filter_map(|id| id.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>())
                } else {
                    None
                }
            })
            .unwrap_or_default();
            
        let oversize_sawmill_ids = json.get("oversizeSawmillIds")
            .and_then(|v| {
                if let serde_json::Value::Array(arr) = v {
                    Some(arr.iter()
                        .filter_map(|id| id.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        
        Ok(Location {
            id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            done: done_val != 0,
            started: started_val != 0,
            last_edit: json.get("lastEdit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            latitude: json.get("latitude").and_then(|v| v.as_f64()).unwrap_or(0.0),
            longitude: json.get("longitude").and_then(|v| v.as_f64()).unwrap_or(0.0),
            partie_nr: json.get("partieNr").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            date: json.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            additional_info: json.get("additionalInfo").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            initial_quantity: json.get("initialQuantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            initial_oversize_quantity: json.get("initialOversizeQuantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            initial_piece_count: json.get("initialPieceCount").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            current_quantity: json.get("currentQuantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            current_oversize_quantity: json.get("currentOversizeQuantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            current_piece_count: json.get("currentPieceCount").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            contract_id: json.get("contractId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            sawmill_ids,
            oversize_sawmill_ids,
        })
    }
    
    pub fn copy_with(&self, 
        sawmill_ids: Option<Vec<String>>, 
        oversize_sawmill_ids: Option<Vec<String>>
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
    active_locations: Arc<Mutex<Vec<Location>>>,
}

impl LocationLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = LocationLocalStorage {
            core_storage: core_storage.clone(),
            active_locations: Arc::new(Mutex::new(Vec::new())),
        };
        
        // Initialize active locations
        storage.init()?;
        
        Ok(storage)
    }
    
    fn init(&self) -> Result<()> {
        let active_locations = self.get_locations_by_condition(false)?;
        let mut locations_lock = self.active_locations.lock().unwrap();
        *locations_lock = active_locations;
        Ok(())
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
    
    fn add_sawmills_to_locations(&self, locations_json: Vec<serde_json::Value>) -> Result<Vec<Location>> {
        let mut locations = Vec::new();
        
        for location_json in locations_json {
            match Location::from_json(&location_json) {
                Ok(mut location) => {
                    let sawmill_ids = self.get_sawmill_ids(&location.id, false)?;
                    let oversize_sawmill_ids = self.get_sawmill_ids(&location.id, true)?;
                    
                    location = location.copy_with(
                        Some(sawmill_ids),
                        Some(oversize_sawmill_ids)
                    );
                    
                    locations.push(location);
                },
                Err(e) => eprintln!("Error parsing location: {}", e),
            }
        }
        
        Ok(locations)
    }
    
    pub fn get_locations_by_condition(&self, is_done: bool) -> Result<Vec<Location>> {
        let done_value = if is_done { "1" } else { "0" };
        
        let locations_json = self.core_storage.get_by_column(
            LocationTable::TABLE_NAME,
            LocationTable::COLUMN_DONE,
            done_value
        )?;
        
        self.add_sawmills_to_locations(locations_json)
    }
    
    pub fn get_finished_locations_by_date(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Location>> {
        let query = format!(
            "SELECT * FROM {} WHERE {} = 1 AND ({} >= ? AND {} <= ?)",
            LocationTable::TABLE_NAME,
            LocationTable::COLUMN_DONE,
            LocationTable::COLUMN_DATE,
            LocationTable::COLUMN_DATE
        );
        
        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(
            params![
                start.to_rfc3339(),
                end.to_rfc3339()
            ],
            |row| {
                // This is a simplified version. You'd need to extract all fields here.
                let id: String = row.get(0)?;
                Ok(id)
            }
        )?;
        
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
        let location_json = self.core_storage.get_by_id(
            LocationTable::TABLE_NAME,
            id
        )?;
        
        if location_json.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        
        match Location::from_json(&location_json[0]) {
            Ok(mut location) => {
                let sawmill_ids = self.get_sawmill_ids(id, false)?;
                let oversize_sawmill_ids = self.get_sawmill_ids(id, true)?;
                
                location = location.copy_with(
                    Some(sawmill_ids),
                    Some(oversize_sawmill_ids)
                );
                
                Ok(location)
            },
            Err(e) => Err(rusqlite::Error::InvalidParameterName(
                format!("Error parsing location: {}", e)
            )),
        }
    }
    
    fn insert_location_sawmill_junction(&self, location_id: &str, sawmill_id: &str, is_oversize: bool) -> Result<i64> {
        let junction_data = serde_json::json!({
            LocationSawmillJunctionTable::COLUMN_LOCATION_ID: location_id,
            LocationSawmillJunctionTable::COLUMN_SAWMILL_ID: sawmill_id,
            LocationSawmillJunctionTable::COLUMN_IS_OVERSIZE: if is_oversize { 1 } else { 0 },
        });
        
        self.core_storage.insert(
            LocationSawmillJunctionTable::TABLE_NAME,
            &junction_data
        )
    }
    
    fn insert_or_update_location(&self, location: &Location) -> Result<i64> {
        // First, delete any existing junction records for this location
        self.core_storage.delete_by_column(
            LocationSawmillJunctionTable::TABLE_NAME,
            LocationSawmillJunctionTable::COLUMN_LOCATION_ID,
            &location.id
        )?;
        
        // Insert normal sawmill junctions
        for sawmill_id in &location.sawmill_ids {
            self.insert_location_sawmill_junction(&location.id, sawmill_id, false)?;
        }
        
        // Insert oversize sawmill junctions
        for sawmill_id in &location.oversize_sawmill_ids {
            self.insert_location_sawmill_junction(&location.id, sawmill_id, true)?;
        }
        
        // Create a location JSON object without the sawmill IDs for database storage
        let mut location_data = location.to_json();
        if let serde_json::Value::Object(ref mut map) = location_data {
            map.remove("sawmillIds");
            map.remove("oversizeSawmillIds");
        }
        
        // Insert or update the location
        self.core_storage.insert_or_update(
            LocationTable::TABLE_NAME,
            &location_data
        )
    }
    
    pub fn save_location(&self, location: &Location) -> Result<i64> {
        let result = self.insert_or_update_location(location)?;
        
        // Update the active locations list
        let mut active_locations = self.active_locations.lock().unwrap();
        
        if !location.done {
            let index = active_locations.iter().position(|l| l.id == location.id);
            if let Some(pos) = index {
                active_locations[pos] = location.clone();
            } else {
                active_locations.push(location.clone());
            }
        } else {
            if let Some(pos) = active_locations.iter().position(|l| l.id == location.id) {
                active_locations.remove(pos);
            }
        }
        
        Ok(result)
    }
    
    pub fn delete_location(&self, id: &str, done: bool) -> Result<usize> {
        // Get the location before deleting it
        let location = self.get_location_by_id(id)?;
        
        // Delete from database
        let result = self.core_storage.delete(
            LocationTable::TABLE_NAME,
            id
        )?;
        
        // If it was an active location, update the active list
        if !done {
            let mut active_locations = self.active_locations.lock().unwrap();
            if let Some(pos) = active_locations.iter().position(|l| l.id == id) {
                active_locations.remove(pos);
            }
        }
        
        Ok(result)
    }
}
