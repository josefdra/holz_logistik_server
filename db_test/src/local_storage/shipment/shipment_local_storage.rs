use crate::local_storage::core_local_storage::CoreLocalStorage;
use crate::local_storage::shipment::shipment_tables::ShipmentTable;
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub id: String,
    pub last_edit: String,
    pub quantity: f64,
    pub oversize_quantity: f64,
    pub piece_count: i32,
    pub user_id: String,
    pub contract_id: String,
    pub sawmill_id: String,
    pub location_id: String,
}

impl Shipment {
    pub fn new(
        quantity: f64,
        oversize_quantity: f64,
        piece_count: i32,
        user_id: String,
        contract_id: String,
        sawmill_id: String,
        location_id: String,
    ) -> Self {
        Shipment {
            id: Uuid::new_v4().to_string(),
            last_edit: Utc::now().to_rfc3339(),
            quantity,
            oversize_quantity,
            piece_count,
            user_id,
            contract_id,
            sawmill_id,
            location_id,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "lastEdit": self.last_edit,
            "quantity": self.quantity,
            "oversizeQuantity": self.oversize_quantity,
            "pieceCount": self.piece_count,
            "userId": self.user_id,
            "contractId": self.contract_id,
            "sawmillId": self.sawmill_id,
            "locationId": self.location_id,
        })
    }
    
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        Ok(Shipment {
            id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            last_edit: json.get("lastEdit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            quantity: json.get("quantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            oversize_quantity: json.get("oversizeQuantity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            piece_count: json.get("pieceCount").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(0),
            user_id: json.get("userId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            contract_id: json.get("contractId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            sawmill_id: json.get("sawmillId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            location_id: json.get("locationId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }
}

pub struct ShipmentLocalStorage {
    core_storage: Arc<CoreLocalStorage>,
}

impl ShipmentLocalStorage {
    pub fn new(core_storage: Arc<CoreLocalStorage>) -> Result<Self> {
        let storage = ShipmentLocalStorage {
            core_storage: core_storage.clone(),
        };
        
        Ok(storage)
    }
    
    pub fn get_shipments_by_date(&self, last_edit: DateTime<Utc>) -> Result<Vec<Shipment>> {
        let query = format!(
            "SELECT * FROM {} WHERE {} >= ?",
            ShipmentTable::TABLE_NAME,
            ShipmentTable::COLUMN_LAST_EDIT
        );
        
        let conn = self.core_storage.get_connection();
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(
            params![
                last_edit.to_rfc3339(),
            ],
            |row| {
                let id: String = row.get(0)?;
                let last_edit: String = row.get(1)?;
                let quantity: f64 = row.get(2)?;
                let oversize_quantity: f64 = row.get(3)?;
                let piece_count: i32 = row.get(4)?;
                let user_id: String = row.get(5)?;
                let contract_id: String = row.get(6)?;
                let sawmill_id: String = row.get(7)?;
                let location_id: String = row.get(8)?;
                
                Ok(Shipment {
                    id,
                    last_edit,
                    quantity,
                    oversize_quantity,
                    piece_count,
                    user_id,
                    contract_id,
                    sawmill_id,
                    location_id,
                })
            }
        )?;
        
        let mut shipments = Vec::new();
        for row in rows {
            match row {
                Ok(shipment) => shipments.push(shipment),
                Err(e) => eprintln!("Error fetching shipment: {}", e),
            }
        }
        
        Ok(shipments)
    }
    
    pub fn save_shipment(&self, shipment: &Shipment) -> Result<i64> {
        let json_data = shipment.to_json();
        let result = self.core_storage.insert_or_update(
            ShipmentTable::TABLE_NAME,
            &json_data
        )?;

        Ok(result)
    }
}
