mod local_storage;

use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::contract::contract_local_storage::{Contract, ContractLocalStorage};
use local_storage::contract::contract_tables::ContractTable;
use local_storage::location::location_tables::{LocationTable, LocationSawmillJunctionTable};
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::sawmill::sawmill_local_storage::{Sawmill, SawmillLocalStorage};
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_tables::UserTable;
use local_storage::user::user_local_storage::{User, UserLocalStorage};

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
    
    // Example: Create a User
    let user_storage = UserLocalStorage::new(core_storage.clone())?;
    let mut user = User::new("Administrator".to_string(), 0);
    println!("Saving new user: {} (ID: {})", user.name, user.id);
    user_storage.save_user(&user)?;
    
    // Example: Create a Sawmill
    let sawmill_storage = SawmillLocalStorage::new(core_storage.clone())?;
    let sawmill1 = Sawmill::new("Sawmill 1".to_string());
    let sawmill2 = Sawmill::new("Sawmill 2".to_string());
    println!("Saving sawmills: {} and {}", sawmill1.name, sawmill2.name);
    sawmill_storage.save_sawmill(&sawmill1)?;
    sawmill_storage.save_sawmill(&sawmill2)?;
    
    // Example: Create a Contract
    let contract_storage = ContractLocalStorage::new(core_storage.clone())?;
    let mut contract = Contract::new("Sample Contract".to_string());
    contract.additional_info = "This is a test contract".to_string();
    contract.available_quantity = 100.0;
    contract.start_date = (Utc::now() - Duration::days(7)).to_rfc3339();
    contract.end_date = (Utc::now() + Duration::days(30)).to_rfc3339();
    
    println!("Saving new contract: {} (ID: {})", contract.title, contract.id);
    contract_storage.save_contract(&contract)?;
    
    // Query active contracts
    let active_contracts = contract_storage.get_active_contracts()?;
    println!("Active contracts: {}", active_contracts.len());
    for (i, c) in active_contracts.iter().enumerate() {
        println!("  {}. {} ({})", i+1, c.title, c.id);
    }
    
    // Example: Mark contract as done
    if !active_contracts.is_empty() {
        let mut updated_contract = active_contracts[0].clone();
        updated_contract.done = true;
        updated_contract.last_edit = Utc::now().to_rfc3339();
        
        println!("Marking contract as done: {}", updated_contract.title);
        contract_storage.save_contract(&updated_contract)?;
        
        // Verify it's no longer active
        let new_active_contracts = contract_storage.get_active_contracts()?;
        println!("Active contracts after update: {}", new_active_contracts.len());
    }
    
    // Display all sawmills
    let all_sawmills = sawmill_storage.get_sawmills();
    println!("All sawmills: {}", all_sawmills.len());
    for (id, sawmill) in &all_sawmills {
        println!("  {} ({})", sawmill.name, id);
    }
    
    println!("Database operations completed successfully");
    
    Ok(())
}

fn initialize_database(db_path: &str) -> Result<()> {
    let conn = rusqlite::Connection::open(db_path)?;
    
    // Create all tables in the correct order
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
