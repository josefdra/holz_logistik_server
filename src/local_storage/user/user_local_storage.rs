use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::user::user_tables::UserTable;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

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

    pub fn get_user_by_id(&self, id: &str) -> Result<Option<Value>> {
        let user_json = self.core_storage.get_by_id(UserTable::TABLE_NAME, id)?;

        if user_json.is_empty() {
            return Ok(None);
        }

        Ok(Some(user_json[0].clone()))
    }

    pub fn get_user_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM {} WHERE deleted = 0 AND lastEdit > ? ORDER BY lastEdit ASC",
            UserTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let last_edit: i64 = row.get(1)?;
            let role: i32 = row.get(2)?;
            let name: String = row.get(3)?;

            let user_json = serde_json::json!({
                "id": id,
                "lastEdit": last_edit,
                "role": role,
                "name": name,
            });

            Ok(user_json)
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

    pub fn save_user(&self, user_data: &Value) -> Result<i64> {
        let result = self
            .core_storage
            .insert_or_update(UserTable::TABLE_NAME, user_data)?;

        Ok(result)
    }
}
