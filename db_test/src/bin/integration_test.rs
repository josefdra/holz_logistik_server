// db_test/src/bin/integration_test.rs
use db_test::local_storage::core_local_storage::CoreLocalStorage;
use db_test::local_storage::contract::contract_local_storage::{Contract, ContractLocalStorage};
use db_test::local_storage::location::location_local_storage::{Location, LocationLocalStorage};
use db_test::local_storage::sawmill::sawmill_local_storage::{Sawmill, SawmillLocalStorage};
use db_test::local_storage::user::user_local_storage::{User, UserLocalStorage};
use db_test::local_storage::note::note_local_storage::{Note, NoteLocalStorage};
use db_test::local_storage::shipment::shipment_local_storage::{Shipment, ShipmentLocalStorage};

use chrono::{Duration, Utc};
use std::sync::Arc;
use std::{fs, path::Path};

fn main() {
    println!("Starting integration test...");
    
    // Create a test database
    let db_path = "test_integration.db";
    if Path::new(db_path).exists() {
        fs::remove_file(db_path).expect("Failed to remove existing test database");
    }
    
    // Initialize database and storage
    db_test::initialize_database(db_path).expect("Failed to initialize database");
    let core_storage = Arc::new(CoreLocalStorage::new(db_path).expect("Failed to create core storage"));
    
    let user_storage = UserLocalStorage::new(core_storage.clone()).expect("Failed to create user storage");
    let sawmill_storage = SawmillLocalStorage::new(core_storage.clone()).expect("Failed to create sawmill storage");
    let contract_storage = ContractLocalStorage::new(core_storage.clone()).expect("Failed to create contract storage");
    let location_storage = LocationLocalStorage::new(core_storage.clone()).expect("Failed to create location storage");
    let note_storage = NoteLocalStorage::new(core_storage.clone()).expect("Failed to create note storage");
    let shipment_storage = ShipmentLocalStorage::new(core_storage.clone()).expect("Failed to create shipment storage");
    
    // Test 1: Create and retrieve users
    println!("Test 1: Creating users...");
    let user1 = User::new("John Smith".to_string(), 1);
    let user2 = User::new("Jane Doe".to_string(), 2);
    
    user_storage.save_user(&user1).expect("Failed to save user1");
    user_storage.save_user(&user2).expect("Failed to save user2");
    
    let one_hour_ago = Utc::now() - Duration::hours(1);
    let users = user_storage.get_user_updates_by_date(one_hour_ago).expect("Failed to get users");
    
    println!("  Retrieved {} users", users.len());
    assert!(users.len() >= 2);
    
    // Test 2: Create and retrieve sawmills
    println!("Test 2: Creating sawmills...");
    let sawmill1 = Sawmill::new("Sawmill Alpha".to_string());
    let sawmill2 = Sawmill::new("Sawmill Beta".to_string());
    
    sawmill_storage.save_sawmill(&sawmill1).expect("Failed to save sawmill1");
    sawmill_storage.save_sawmill(&sawmill2).expect("Failed to save sawmill2");
    
    let sawmills = sawmill_storage.get_sawmill_updates_by_date(one_hour_ago).expect("Failed to get sawmills");
    
    println!("  Retrieved {} sawmills", sawmills.len());
    assert!(sawmills.len() >= 2);
    
    // Test 3: Create and retrieve contracts
    println!("Test 3: Creating contracts...");
    let mut contract1 = Contract::new("Pine Wood Supply".to_string());
    contract1.available_quantity = 1000.0;
    contract1.booked_quantity = 0.0;
    
    let mut contract2 = Contract::new("Oak Wood Supply".to_string());
    contract2.available_quantity = 500.0;
    contract2.booked_quantity = 0.0;
    
    contract_storage.save_contract(&contract1).expect("Failed to save contract1");
    contract_storage.save_contract(&contract2).expect("Failed to save contract2");
    
    let contracts = contract_storage.get_contract_updates_by_date(one_hour_ago).expect("Failed to get contracts");
    
    println!("  Retrieved {} contracts", contracts.len());
    assert!(contracts.len() >= 2);
    
    // Test 4: Create and retrieve locations with sawmill relationships
    println!("Test 4: Creating locations with sawmill relationships...");
    let mut location1 = Location::new(contract1.id.clone());
    location1.latitude = 47.123;
    location1.longitude = 8.456;
    location1.partie_nr = "PART-001".to_string();
    location1.initial_quantity = 500.0;
    location1.current_quantity = 500.0;
    location1.sawmill_ids = vec![sawmill1.id.clone()];
    
    let mut location2 = Location::new(contract2.id.clone());
    location2.latitude = 46.789;
    location2.longitude = 9.012;
    location2.partie_nr = "PART-002".to_string();
    location2.initial_quantity = 250.0;
    location2.current_quantity = 250.0;
    location2.sawmill_ids = vec![sawmill1.id.clone(), sawmill2.id.clone()];
    location2.oversize_sawmill_ids = vec![sawmill2.id.clone()];
    
    location_storage.save_location(&location1).expect("Failed to save location1");
    location_storage.save_location(&location2).expect("Failed to save location2");
    
    let locations = location_storage.get_location_updates_by_date(one_hour_ago).expect("Failed to get locations");
    
    println!("  Retrieved {} locations", locations.len());
    assert!(locations.len() >= 2);
    
    // Verify location sawmill relationships
    let retrieved_location = location_storage.get_location_by_id(&location2.id).expect("Failed to get location2");
    assert_eq!(retrieved_location.sawmill_ids.len(), 2);
    assert_eq!(retrieved_location.oversize_sawmill_ids.len(), 1);
    
    // Test 5: Create notes
    println!("Test 5: Creating notes...");
    let note1 = Note::new("Note for contract1".to_string(), user1.id.clone());
    let note2 = Note::new("Note for location2".to_string(), user2.id.clone());
    
    note_storage.save_note(&note1).expect("Failed to save note1");
    note_storage.save_note(&note2).expect("Failed to save note2");
    
    let notes = note_storage.get_note_updates_by_date(one_hour_ago).expect("Failed to get notes");
    
    println!("  Retrieved {} notes", notes.len());
    assert!(notes.len() >= 2);
    
    // Test 6: Create shipments and verify impact on contract quantities
    println!("Test 6: Creating shipments...");
    let shipment1 = Shipment::new(
        100.0, // quantity
        10.0,  // oversize_quantity
        50,    // piece_count
        user1.id.clone(),
        contract1.id.clone(),
        sawmill1.id.clone(),
        location1.id.clone()
    );
    
    shipment_storage.save_shipment(&shipment1).expect("Failed to save shipment1");
    
    // Update contract shipped quantity (in a real app, this would be part of a transaction)
    let mut updated_contract1 = contract1.clone();
    updated_contract1.shipped_quantity += shipment1.quantity;
    contract_storage.save_contract(&updated_contract1).expect("Failed to update contract1");
    
    // Update location current quantity
    let mut updated_location1 = location1.clone();
    updated_location1.current_quantity -= shipment1.quantity;
    location_storage.save_location(&updated_location1).expect("Failed to update location1");
    
    // Verify updates
    let shipments = shipment_storage.get_shipments_by_date(one_hour_ago).expect("Failed to get shipments");
    println!("  Retrieved {} shipments", shipments.len());
    assert!(shipments.len() >= 1);
    
    let updated_contracts = contract_storage.get_contract_updates_by_date(one_hour_ago).expect("Failed to get updated contracts");
    let retrieved_contract = updated_contracts.iter().find(|c| c.id == contract1.id).unwrap();
    assert_eq!(retrieved_contract.shipped_quantity, 100.0);
    
    let retrieved_location = location_storage.get_location_by_id(&location1.id).expect("Failed to get updated location1");
    assert_eq!(retrieved_location.current_quantity, 400.0);
    
    println!("All integration tests passed!");
    
    // Clean up
    fs::remove_file(db_path).expect("Failed to remove test database");
}
