use crate::local_storage::contract::contract_tables::ContractTable;
use crate::local_storage::core_local_storage::CoreLocalStorage;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub done: bool,
    pub last_edit: String,
    pub title: String,
    pub additional_info: String,
    pub start_date: String,
    pub end_date: String,
    pub available_quantity: f64,
    pub booked_quantity: f64,
    pub shipped_quantity: f64,
}

impl Contract {
    pub fn new(title: String) -> Self {
        Contract {
            id: Uuid::new_v4().to_string(),
            done: false,
            last_edit: Utc::now().to_rfc3339(),
            title,
            additional_info: String::new(),
            start_date: Utc::now().to_rfc3339(),
            end_date: Utc::now().to_rfc3339(),
            available_quantity: 0.0,
            booked_quantity: 0.0,
            shipped_quantity: 0.0,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "done": if self.done { 1 } else { 0 },
            "lastEdit": self.last_edit,
            "title": self.title,
            "additionalInfo": self.additional_info,
            "startDate": self.start_date,
            "endDate": self.end_date,
            "availableQuantity": self.available_quantity,
            "bookedQuantity": self.booked_quantity,
            "shippedQuantity": self.shipped_quantity,
        })
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let done_val = json.get("done").and_then(|v| v.as_i64()).unwrap_or(0);

        Ok(Contract {
            id: json
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            done: done_val != 0,
            last_edit: json
                .get("lastEdit")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            additional_info: json
                .get("additionalInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            start_date: json
                .get("startDate")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            end_date: json
                .get("endDate")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            available_quantity: json
                .get("availableQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            booked_quantity: json
                .get("bookedQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            shipped_quantity: json
                .get("shippedQuantity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }
}

pub struct ContractLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl ContractLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = ContractLocalStorage {
            core_storage: core_storage.clone(),
        };

        Ok(storage)
    }

    pub fn get_contract_updates_by_date(
        &self,
        last_edit: DateTime<Utc>,
    ) -> Result<Vec<Contract>> {
        let query = format!(
            "SELECT * FROM {} WHERE lastEdit >= ?",
            ContractTable::TABLE_NAME
        );

        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(
            params![
                last_edit.to_rfc3339(),
            ],
            |row| {
                let id: String = row.get(0)?;
                let done: i64 = row.get(1)?;
                let last_edit: String = row.get(2)?;
                let title: String = row.get(3)?;
                let additional_info: String = row.get(4)?;
                let start_date: String = row.get(5)?;
                let end_date: String = row.get(6)?;
                let available_quantity: f64 = row.get(7)?;
                let booked_quantity: f64 = row.get(8)?;
                let shipped_quantity: f64 = row.get(9)?;

                Ok(Contract {
                    id,
                    done: done != 0,
                    last_edit,
                    title,
                    additional_info,
                    start_date,
                    end_date,
                    available_quantity,
                    booked_quantity,
                    shipped_quantity,
                })
            },
        )?;

        let mut contracts = Vec::new();
        for row in rows {
            match row {
                Ok(contract) => contracts.push(contract),
                Err(e) => eprintln!("Error fetching contract: {}", e),
            }
        }

        Ok(contracts)
    }

    pub fn save_contract(&self, contract: &Contract) -> Result<i64> {
        let json_data = contract.to_json();
        let result = self
            .core_storage
            .insert_or_update(ContractTable::TABLE_NAME, &json_data)?;

        Ok(result)
    }
}
