pub mod local_storage;

#[cfg(test)]
pub mod test;

use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::contract::contract_tables::ContractTable;
use local_storage::location::location_tables::{LocationTable, LocationSawmillJunctionTable};
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_tables::UserTable;

pub fn initialize_database(db_path: &str) -> rusqlite::Result<()> {
    let conn = rusqlite::Connection::open(db_path)?;
    
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    
    conn.execute(&UserTable::create_table(), [])?;
    conn.execute(&SawmillTable::create_table(), [])?;
    conn.execute(&ContractTable::create_table(), [])?;
    
    conn.execute(&LocationTable::create_table(), [])?;
    conn.execute(&LocationSawmillJunctionTable::create_table(), [])?;
    conn.execute(&NoteTable::create_table(), [])?;
    conn.execute(&PhotoTable::create_table(), [])?;
    conn.execute(&ShipmentTable::create_table(), [])?;
    
    Ok(())
}
