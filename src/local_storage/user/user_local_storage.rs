use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::user::user_tables::UserTable;
use chrono::{DateTime, Utc};
use rusqlite::{Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub last_edit: String,
    pub role: i32,
    pub name: String,
}

impl User {
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
            role: json
                .get("role")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            name: json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

pub struct UserLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl UserLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = UserLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    pub fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let user_json = self.core_storage.get_by_id(UserTable::TABLE_NAME, id)?;

        if user_json.is_empty() {
            return Ok(None);
        }

        match User::from_json(&user_json[0]) {
            Ok(user) => Ok(Some(user)),
            Err(e) => Err(rusqlite::Error::InvalidParameterName(format!(
                "Error parsing user: {}",
                e
            ))),
        }
    }

    pub fn get_user_updates_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<User>> {
        let query = format!(
            "SELECT * FROM {} WHERE lastEdit >= ?",
            UserTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(params![last_edit.to_rfc3339()], |row| {
            let id: String = row.get(0)?;
            let last_edit: String = row.get(1)?;
            let role: i32 = row.get(2)?;
            let name: String = row.get(3)?;

            Ok(User {
                id,
                last_edit,
                role,
                name,
            })
        })?;

        let mut users = Vec::new();
        for row in rows {
            match row {
                Ok(user) => users.push(user),
                Err(e) => eprintln!("Error fetching user: {}", e),
            }
        }

        Ok(users)
    }

    pub fn save_user(&self, user: &User) -> Result<i64> {
        let json_data = user.to_json();
        let result = self
            .core_storage
            .insert_or_update(UserTable::TABLE_NAME, &json_data)?;

        Ok(result)
    }
}
