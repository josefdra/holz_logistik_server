use crate::local_storage::core_local_storage::CoreLocalStorage;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

pub struct NoteLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl NoteLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = NoteLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    pub fn get_note_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM notes WHERE deleted = 0 AND arrivalAtServer > ? ORDER BY lastEdit ASC",
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let last_edit: i64 = row.get(1)?;
            let text: String = row.get(2)?;
            let user_id: String = row.get(3)?;

            let note_json = serde_json::json!({
                "id": id,
                "lastEdit": last_edit,
                "text": text,
                "userId": user_id,
            });

            Ok(note_json)
        })?;

        let mut notes = Vec::new();
        for row in rows {
            match row {
                Ok(note) => notes.push(note),
                Err(e) => eprintln!("Error fetching note: {}", e),
            }
        }

        Ok(notes)
    }

    pub fn save_note(&self, note_data: &Value) -> Result<i64> {
        let mut note_for_save = note_data.clone();
        if let serde_json::Value::Object(ref mut map) = note_for_save {
            map.insert("arrivalAtServer".to_string(), chrono::Utc::now().timestamp_millis().into());
        }

        let result = self.core_storage
            .insert_or_update("notes", &note_for_save)?;

        Ok(result)
    }
}
