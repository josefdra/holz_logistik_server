use crate::local_storage::core_local_storage::CoreLocalStorage;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

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

    pub fn get_contract_updates_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM contracts WHERE deleted = 0 AND arrivalAtServer > ? ORDER BY lastEdit ASC",
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let done: i64 = row.get(1)?;
            let last_edit: i64 = row.get(2)?;
            let title: String = row.get(3)?;
            let additional_info: String = row.get(4)?;
            let start_date: i64 = row.get(5)?;
            let end_date: i64 = row.get(6)?;
            let available_quantity: f64 = row.get(7)?;
            let booked_quantity: f64 = row.get(8)?;
            let shipped_quantity: f64 = row.get(9)?;

            let contract_json = serde_json::json!({
                "id": id,
                "done": done,
                "lastEdit": last_edit,
                "title": title,
                "additionalInfo": additional_info,
                "startDate": start_date,
                "endDate": end_date,
                "availableQuantity": available_quantity,
                "bookedQuantity": booked_quantity,
                "shippedQuantity": shipped_quantity,
            });

            Ok(contract_json)
        })?;

        let mut contracts = Vec::new();
        for row in rows {
            match row {
                Ok(contract) => contracts.push(contract),
                Err(e) => eprintln!("Error fetching contract: {}", e),
            }
        }

        Ok(contracts)
    }

    pub fn save_contract(&self, contract_data: &Value) -> Result<i64> {
        let mut contract_for_save = contract_data.clone();
        if let serde_json::Value::Object(ref mut map) = contract_for_save {
            map.insert("arrivalAtServer".to_string(), chrono::Utc::now().timestamp_millis().into());
        }

        let result = self.core_storage
            .insert_or_update("contracts", &contract_for_save)?;

        Ok(result)
    }
}
