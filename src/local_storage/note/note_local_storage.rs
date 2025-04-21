use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::note::note_tables::NoteTable;
use chrono::{Utc, DateTime};
use rusqlite::{Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub last_edit: String,
    pub text: String,
    pub user_id: String,
}

impl Note {
    pub fn new(text: String, user_id: String) -> Self {
        Note {
            id: Uuid::new_v4().to_string(),
            last_edit: Utc::now().to_rfc3339(),
            text,
            user_id,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "text": self.text,
            "userId": self.user_id,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Note {
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
            text: json
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_id: json
                .get("userId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

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

    pub fn get_note_updates_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<Note>> {
        let query = format!(
            "SELECT * FROM {} WHERE lastEdit >= ?",
            NoteTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(params![last_edit.to_rfc3339()], |row| {
            let id: String = row.get(0)?;
            let last_edit: String = row.get(1)?;
            let text: String = row.get(2)?;
            let user_id: String = row.get(3)?;

            Ok(Note {
                id,
                last_edit,
                text,
                user_id,
            })
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

    pub fn save_note(&self, note: &Note) -> Result<i64> {
        let json_data = note.to_json();
        let result = self
            .core_storage
            .insert_or_update(NoteTable::TABLE_NAME, &json_data)?;

        Ok(result)
    }
}
