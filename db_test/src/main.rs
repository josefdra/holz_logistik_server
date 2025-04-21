mod local_storage;

use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::contract::contract_local_storage::{Contract, ContractLocalStorage};
use local_storage::contract::contract_tables::ContractTable;
use local_storage::location::location_local_storage::{Location, LocationLocalStorage};
use local_storage::location::location_tables::{LocationTable, LocationSawmillJunctionTable};
use local_storage::note::note_local_storage::NoteLocalStorage;
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_local_storage::PhotoLocalStorage;
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_local_storage::{Sawmill, SawmillLocalStorage};
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::shipment::shipment_local_storage::ShipmentLocalStorage;
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_local_storage::{User, UserLocalStorage};
use local_storage::user::user_tables::UserTable;

use chrono::{Duration, Utc};
use rusqlite::Result;
use std::sync::Arc;

fn main() -> Result<()> {
    // Database path
    let db_path = "holz_logistik.db";
    
    // Initialize database and create tables
    initialize_database(db_path)?;
    
    // Create Core Storage
    let core_storage = Arc::new(CoreLocalStorage::new(db_path)?);
    
    // Initialize all storage modules
    let user_storage = UserLocalStorage::new(core_storage.clone())?;
    let sawmill_storage = SawmillLocalStorage::new(core_storage.clone())?;
    let contract_storage = ContractLocalStorage::new(core_storage.clone())?;
    let location_storage = LocationLocalStorage::new(core_storage.clone())?;
    let _note_storage = NoteLocalStorage::new(core_storage.clone())?;
    let _photo_storage = PhotoLocalStorage::new(core_storage.clone())?;
    let _shipment_storage = ShipmentLocalStorage::new(core_storage.clone())?;
    
    // Create demo data
    create_demo_data(
        &user_storage,
        &sawmill_storage,
        &contract_storage,
        &location_storage
    )?;
    
    // Query and print demo data to verify structure
    print_data_summary(
        &user_storage,
        &sawmill_storage,
        &contract_storage,
        &location_storage
    )?;
    
    println!("Database operations completed successfully");
    
    Ok(())
}

fn initialize_database(db_path: &str) -> Result<()> {
    let conn = rusqlite::Connection::open(db_path)?;
    
    // Create all tables in the correct order
    // First, tables without foreign keys
    conn.execute(&UserTable::create_table(), [])?;
    conn.execute(&SawmillTable::create_table(), [])?;
    conn.execute(&ContractTable::create_table(), [])?;
    
    // Then tables with foreign keys
    conn.execute(&LocationTable::create_table(), [])?;
    conn.execute(&LocationSawmillJunctionTable::create_table(), [])?;
    conn.execute(&NoteTable::create_table(), [])?;
    conn.execute(&PhotoTable::create_table(), [])?;
    conn.execute(&ShipmentTable::create_table(), [])?;
    
    println!("Database initialized with all tables");
    
    Ok(())
}

fn create_demo_data(
    user_storage: &UserLocalStorage,
    sawmill_storage: &SawmillLocalStorage,
    contract_storage: &ContractLocalStorage,
    location_storage: &LocationLocalStorage,
) -> Result<()> {
    // Create a user
    let user = User::new("Administrator".to_string(), 0);
    println!("Creating user: {} (ID: {})", user.name, user.id);
    user_storage.save_user(&user)?;
    
    // Create sawmills
    let sawmill1 = Sawmill::new("Sawmill 1".to_string());
    let sawmill2 = Sawmill::new("Sawmill 2".to_string());
    println!("Creating sawmills: {} and {}", sawmill1.name, sawmill2.name);
    sawmill_storage.save_sawmill(&sawmill1)?;
    sawmill_storage.save_sawmill(&sawmill2)?;
    
    // Create a contract
    let mut contract = Contract::new("Sample Contract".to_string());
    contract.additional_info = "This is a test contract".to_string();
    contract.available_quantity = 100.0;
    contract.start_date = (Utc::now() - Duration::days(7)).to_rfc3339();
    contract.end_date = (Utc::now() + Duration::days(30)).to_rfc3339();
    
    println!("Creating contract: {} (ID: {})", contract.title, contract.id);
    contract_storage.save_contract(&contract)?;
    
    // Create a location
    let mut location = Location::new(contract.id.clone());
    location.partie_nr = "LOC-001".to_string();
    location.additional_info = "Test location".to_string();
    location.latitude = 48.137154;
    location.longitude = 11.576124;
    location.initial_quantity = 50.0;
    location.current_quantity = 50.0;
    location.sawmill_ids = vec![sawmill1.id.clone()];
    
    println!("Creating location: {} (ID: {})", location.partie_nr, location.id);
    location_storage.save_location(&location)?;
    
    Ok(())
}

fn print_data_summary(
    user_storage: &UserLocalStorage,
    sawmill_storage: &SawmillLocalStorage,
    contract_storage: &ContractLocalStorage,
    location_storage: &LocationLocalStorage,
) -> Result<()> {
    println!("\n----- Data Summary -----");
    
    // Print users
    let users = user_storage.get_users();
    println!("Users: {}", users.len());
    for (id, user) in &users {
        println!("  - {} (ID: {})", user.name, id);
    }
    
    // Print sawmills
    let sawmills = sawmill_storage.get_sawmills();
    println!("Sawmills: {}", sawmills.len());
    for (id, sawmill) in &sawmills {
        println!("  - {} (ID: {})", sawmill.name, id);
    }
    
    // Print contracts
    let active_contracts = contract_storage.get_active_contracts()?;
    println!("Active contracts: {}", active_contracts.len());
    for contract in &active_contracts {
        println!("  - {} (ID: {})", contract.title, contract.id);
        println!("    Available: {}, Booked: {}, Shipped: {}", 
            contract.available_quantity, 
            contract.booked_quantity, 
            contract.shipped_quantity);
    }
    
    // Print locations
    let active_locations = location_storage.get_locations_by_condition(false)?;
    println!("Active locations: {}", active_locations.len());
    for location in &active_locations {
        println!("  - {} (ID: {})", location.partie_nr, location.id);
        println!("    Lat: {}, Long: {}", location.latitude, location.longitude);
        println!("    Contract ID: {}", location.contract_id);
        println!("    Sawmill IDs: {:?}", location.sawmill_ids);
    }
    
    println!("-----------------------\n");
    
    Ok(())
}
