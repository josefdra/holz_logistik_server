mod local_storage;

use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::contract::contract_local_storage::ContractLocalStorage;
use local_storage::contract::contract_tables::ContractTable;
use local_storage::location::location_local_storage::LocationLocalStorage;
use local_storage::location::location_tables::{LocationTable, LocationSawmillJunctionTable};
use local_storage::note::note_local_storage::NoteLocalStorage;
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_local_storage::PhotoLocalStorage;
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_local_storage::SawmillLocalStorage;
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::shipment::shipment_local_storage::ShipmentLocalStorage;
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_local_storage::UserLocalStorage;
use local_storage::user::user_tables::UserTable;

use rusqlite::Result;
use std::sync::Arc;
use std::env;

fn main() -> Result<()> {
    let db_path = "databases/holz_logistik.db";
    
    let args: Vec<String> = env::args().collect();
    let run_tests = args.len() > 1 && args[1] == "--test";
    
    if run_tests {
        return Ok(());
    }
    
    initialize_database(db_path)?;
    
    let core_storage = Arc::new(CoreLocalStorage::new(db_path)?);
    
    let user_storage = UserLocalStorage::new(core_storage.clone())?;
    let sawmill_storage = SawmillLocalStorage::new(core_storage.clone())?;
    let contract_storage = ContractLocalStorage::new(core_storage.clone())?;
    let location_storage = LocationLocalStorage::new(core_storage.clone())?;
    let _note_storage = NoteLocalStorage::new(core_storage.clone())?;
    let _photo_storage = PhotoLocalStorage::new(core_storage.clone())?;
    let _shipment_storage = ShipmentLocalStorage::new(core_storage.clone())?;
    
    Ok(())
}

fn initialize_database(db_path: &str) -> Result<()> {
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
    
    println!("Database initialized with all tables");
    
    Ok(())
}
