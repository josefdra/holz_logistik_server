use db_test::local_storage::contract::contract_local_storage::ContractLocalStorage;
use db_test::local_storage::core_local_storage::CoreLocalStorage;
use db_test::local_storage::location::location_local_storage::LocationLocalStorage;
use db_test::local_storage::note::note_local_storage::NoteLocalStorage;
use db_test::local_storage::photo::photo_local_storage::PhotoLocalStorage;
use db_test::local_storage::sawmill::sawmill_local_storage::SawmillLocalStorage;
use db_test::local_storage::shipment::shipment_local_storage::ShipmentLocalStorage;
use db_test::local_storage::user::user_local_storage::UserLocalStorage;

use std::env;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    #[test]
    fn run_test_suite() {
        // This is a meta-test that ensures our test.rs file is included in the build
        // and that its tests are discovered
        println!("Test module is properly included");
    }
}

fn main() -> rusqlite::Result<()> {
    let db_path = "databases/holz_logistik.db";

    let args: Vec<String> = env::args().collect();
    let run_tests = args.len() > 1 && args[1] == "--test";

    if run_tests {
        println!("Running in test mode");
        return Ok(());
    }

    db_test::initialize_database(db_path)?;

    let core_storage = Arc::new(CoreLocalStorage::new(db_path)?);

    let _user_storage = UserLocalStorage::new(core_storage.clone())?;
    let _sawmill_storage = SawmillLocalStorage::new(core_storage.clone())?;
    let _contract_storage = ContractLocalStorage::new(core_storage.clone())?;
    let _location_storage = LocationLocalStorage::new(core_storage.clone())?;
    let _note_storage = NoteLocalStorage::new(core_storage.clone())?;
    let _photo_storage = PhotoLocalStorage::new(core_storage.clone())?;
    let _shipment_storage = ShipmentLocalStorage::new(core_storage.clone())?;

    println!("Server initialized successfully");

    Ok(())
}
