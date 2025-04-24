use crate::local_storage::core_local_storage::CoreLocalStorage;
use rusqlite::{Result, params};
use serde_json::Value;
use std::sync::Arc;

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

    pub fn get_shipments_by_date(&self, last_edit: i64) -> Result<Vec<Value>> {
        let query = format!(
            "SELECT * FROM shipments WHERE deleted = 0 AND lastEdit > ? ORDER BY lastEdit ASC",
        );

        let conn = self.core_storage.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let rows = stmt.query_map(params![last_edit], |row| {
            let id: String = row.get(0)?;
            let last_edit: i64 = row.get(1)?;
            let quantity: f64 = row.get(2)?;
            let oversize_quantity: f64 = row.get(3)?;
            let piece_count: i32 = row.get(4)?;
            let user_id: String = row.get(5)?;
            let contract_id: String = row.get(6)?;
            let sawmill_id: String = row.get(7)?;
            let location_id: String = row.get(8)?;

            let shipment_json = serde_json::json!({
                "id": id,
                "lastEdit": last_edit,
                "quantity": quantity,
                "oversizeQuantity": oversize_quantity,
                "pieceCount": piece_count,
                "userId": user_id,
                "contractId": contract_id,
                "sawmillId": sawmill_id,
                "locationId": location_id,
            });

            Ok(shipment_json)
        })?;

        let mut shipments = Vec::new();
        for row in rows {
            match row {
                Ok(shipment) => shipments.push(shipment),
                Err(e) => eprintln!("Error fetching shipment: {}", e),
            }
        }

        Ok(shipments)
    }

    pub fn save_shipment(&self, shipment_data: &Value) -> Result<i64> {
        let result = self
            .core_storage
            .insert_or_update("shipments", shipment_data)?;

        Ok(result)
    }
}
