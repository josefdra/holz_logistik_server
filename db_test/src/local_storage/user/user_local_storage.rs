use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::user::user_tables::UserTable;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub last_edit: String,
    pub role: i32,
    pub name: String,
}

impl User {
    pub fn new(name: String, role: i32) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            last_edit: Utc::now().to_rfc3339(),
            role,
            name,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "role": self.role,
            "name": self.name,
        })
    }
    
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(User {
            id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            last_edit: json.get("lastEdit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            role: json.get("role").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            name: json.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }
}

pub struct UserLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl UserLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = UserLocalStorage {
            core_storage: core_storage.clone(),
            users: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Initialize users
        storage.init()?;
        
        Ok(storage)
    }
    
    fn init(&self) -> Result<()> {
        let users_json = self.core_storage.get_all(UserTable::TABLE_NAME)?;
        
        let mut users = HashMap::new();
        for user_json in users_json {
            match User::from_json(&user_json) {
                Ok(user) => {
                    users.insert(user.id.clone(), user);
                },
                Err(e) => eprintln!("Error parsing user: {}", e),
            }
        }
        
        let mut users_lock = self.users.lock().unwrap();
        *users_lock = users;
        
        Ok(())
    }
    
    pub fn save_user(&self, user: &User) -> Result<i64> {
        let json_data = user.to_json();
        let result = self.core_storage.insert_or_update(
            UserTable::TABLE_NAME,
            &json_data
        )?;
        
        // Update the users map
        let mut users = self.users.lock().unwrap();
        users.insert(user.id.clone(), user.clone());
        
        Ok(result)
    }
    
    pub fn delete_user(&self, id: &str) -> Result<usize> {
        // Delete from database
        let result = self.core_storage.delete(
            UserTable::TABLE_NAME,
            id
        )?;
        
        // Update the users map
        let mut users = self.users.lock().unwrap();
        users.remove(id);
        
        Ok(result)
    }
    
    pub fn get_users(&self) -> HashMap<String, User> {
        let users = self.users.lock().unwrap();
        users.clone()
    }
}
