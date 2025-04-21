use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::note::note_tables::NoteTable;
use chrono::Utc;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
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
    notes: Arc<Mutex<Vec<Note>>>,
}

impl NoteLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = NoteLocalStorage {
            core_storage: core_storage.clone(),
            notes: Arc::new(Mutex::new(Vec::new())),
        };

        // Initialize notes
        storage.init()?;

        Ok(storage)
    }

    fn init(&self) -> Result<()> {
        let notes = self.get_all_notes()?;
        let mut notes_lock = self.notes.lock().unwrap();
        *notes_lock = notes;
        Ok(())
    }

    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        let notes_json = self.core_storage.get_all(NoteTable::TABLE_NAME)?;

        let mut notes = Vec::new();
        for note_json in notes_json {
            match Note::from_json(&note_json) {
                Ok(note) => notes.push(note),
                Err(e) => eprintln!("Error parsing note: {}", e),
            }
        }

        Ok(notes)
    }

    pub fn get_note_by_id(&self, id: &str) -> Result<Note> {
        let json_values = self.core_storage.get_by_id(NoteTable::TABLE_NAME, id)?;

        if json_values.is_empty() {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        match Note::from_json(&json_values[0]) {
            Ok(note) => Ok(note),
            Err(e) => Err(rusqlite::Error::InvalidParameterName(format!(
                "Error parsing note: {}",
                e
            ))),
        }
    }

    pub fn save_note(&self, note: &Note) -> Result<i64> {
        let json_data = note.to_json();
        let result = self
            .core_storage
            .insert_or_update(NoteTable::TABLE_NAME, &json_data)?;

        // Update the notes list
        let mut notes = self.notes.lock().unwrap();

        let index = notes.iter().position(|n| n.id == note.id);
        if let Some(pos) = index {
            notes[pos] = note.clone();
        } else {
            notes.push(note.clone());
        }

        Ok(result)
    }

    pub fn delete_note(&self, id: &str) -> Result<usize> {
        // Delete from database
        let result = self.core_storage.delete(NoteTable::TABLE_NAME, id)?;

        // Update the notes list
        let mut notes = self.notes.lock().unwrap();
        if let Some(pos) = notes.iter().position(|n| n.id == id) {
            notes.remove(pos);
        }

        Ok(result)
    }
}
