use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::sawmill::sawmill_tables::SawmillTable;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sawmill {
    pub id: String,
    pub last_edit: String,
    pub name: String,
}

impl Sawmill {
    pub fn new(name: String) -> Self {
        Sawmill {
            id: Uuid::new_v4().to_string(),
            last_edit: Utc::now().to_rfc3339(),
            name,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "name": self.name,
        })
    }
    
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Sawmill {
            id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            last_edit: json.get("lastEdit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: json.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }
}

pub struct SawmillLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
    sawmills: Arc<Mutex<HashMap<String, Sawmill>>>,
}

impl SawmillLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = SawmillLocalStorage {
            core_storage: core_storage.clone(),
            sawmills: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Initialize sawmills
        storage.init()?;
        
        Ok(storage)
    }
    
    fn init(&self) -> Result<()> {
        let sawmills_json = self.core_storage.get_all(SawmillTable::TABLE_NAME)?;
        
        let mut sawmills = HashMap::new();
        for sawmill_json in sawmills_json {
            match Sawmill::from_json(&sawmill_json) {
                Ok(sawmill) => {
                    sawmills.insert(sawmill.id.clone(), sawmill);
                },
                Err(e) => eprintln!("Error parsing sawmill: {}", e),
            }
        }
        
        let mut sawmills_lock = self.sawmills.lock().unwrap();
        *sawmills_lock = sawmills;
        
        Ok(())
    }
    
    pub fn save_sawmill(&self, sawmill: &Sawmill) -> Result<i64> {
        let json_data = sawmill.to_json();
        let result = self.core_storage.insert_or_update(
            SawmillTable::TABLE_NAME,
            &json_data
        )?;
        
        // Update the sawmills map
        let mut sawmills = self.sawmills.lock().unwrap();
        sawmills.insert(sawmill.id.clone(), sawmill.clone());
        
        Ok(result)
    }
    
    pub fn delete_sawmill(&self, id: &str) -> Result<usize> {
        // Delete from database
        let result = self.core_storage.delete(
            SawmillTable::TABLE_NAME,
            id
        )?;
        
        // Update the sawmills map
        let mut sawmills = self.sawmills.lock().unwrap();
        sawmills.remove(id);
        
        Ok(result)
    }
    
    pub fn get_sawmills(&self) -> HashMap<String, Sawmill> {
        let sawmills = self.sawmills.lock().unwrap();
        sawmills.clone()
    }
}
