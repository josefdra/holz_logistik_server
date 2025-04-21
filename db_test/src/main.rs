use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    id: i32,
    name: String,
    age: i32,
    last_edit: Option<String>, // ISO 8601 date format like "2023-04-01T14:30:00Z"
}

fn main() -> Result<()> {
    // Open a connection to the SQLite database
    let mut conn = Connection::open("my_database.db")?;

    // Create the table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS people (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            age   INTEGER,
            last_edit TEXT
        )",
        [],
    )?;

    println!("Table created successfully");

    // JSON string to use throughout the examples
    let json_str = r#"
    [
        {"id": 1, "name": "Alice", "age": 30, "last_edit": "2023-01-15T10:30:00Z"},
        {"id": 2, "name": "Bob", "age": 25, "last_edit": "2023-03-20T14:45:00Z"},
        {"id": 3, "name": "Carol", "age": 35, "last_edit": "2022-11-05T09:15:00Z"},
        {"id": 4, "name": "Dave", "age": 40, "last_edit": "2023-02-10T16:20:00Z"}
    ]
    "#;

    // Example 1: Reading from JSON and inserting into the database (using structs)
    {
        println!("\nExample 1: Reading from JSON and inserting into the database (using structs)");
        
        // Deserialize JSON into Vec<Person>
        let people: Vec<Person> = serde_json::from_str(json_str)
            .expect("Failed to parse JSON");

        // Insert the data using UPSERT
        for person in &people {
            conn.execute(
                "INSERT INTO people (id, name, age, last_edit) VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(id) DO UPDATE SET 
                    name = excluded.name,
                    age = excluded.age,
                    last_edit = excluded.last_edit",
                params![person.id, person.name, person.age, person.last_edit],
            )?;
        }

        println!("Data from JSON inserted successfully");
    }

    // Example 2: Querying all data and converting to JSON
    {
        println!("\nExample 2: Querying all data and converting to JSON");
        
        // Query all people
        let mut stmt = conn.prepare("SELECT id, name, age, last_edit FROM people ORDER BY id")?;
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                age: row.get(2)?,
                last_edit: row.get(3)?,
            })
        })?;

        // Collect the results into a Vec<Person>
        let mut all_people = Vec::new();
        for person_result in person_iter {
            all_people.push(person_result?);
        }

        // Serialize to JSON
        let json_output = serde_json::to_string_pretty(&all_people)
            .expect("Failed to serialize to JSON");

        // Print the JSON
        println!("All people as JSON:");
        println!("{}", json_output);

        // Optionally, write to a file
        fs::write("people.json", &json_output)
            .expect("Failed to write JSON to file");
        println!("JSON data written to people.json");
    }

    // Example 3: Query with filtering and return as JSON
    {
        println!("\nExample 3: Query with filtering and return as JSON");
        
        let min_age = 30;
        let mut stmt = conn.prepare("SELECT id, name, age, last_edit FROM people WHERE age >= ? ORDER BY age")?;
        let filtered_iter = stmt.query_map([min_age], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                age: row.get(2)?,
                last_edit: row.get(3)?,
            })
        })?;

        // Collect the filtered results
        let mut filtered_people = Vec::new();
        for person_result in filtered_iter {
            filtered_people.push(person_result?);
        }

        // Serialize to JSON
        let filtered_json = serde_json::to_string_pretty(&filtered_people)
            .expect("Failed to serialize filtered results to JSON");

        // Print the filtered JSON
        println!("People age >= {} as JSON:", min_age);
        println!("{}", filtered_json);
    }

    // Example 4: Sorting by lastEdit (oldest first)
    {
        println!("\nExample 4: Sorting by lastEdit (oldest first)");
        
        let mut stmt = conn.prepare("SELECT id, name, age, last_edit FROM people ORDER BY last_edit ASC")?;
        let sorted_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                age: row.get(2)?,
                last_edit: row.get(3)?,
            })
        })?;

        // Collect and print the sorted results
        let mut sorted_people = Vec::new();
        for person_result in sorted_iter {
            let person = person_result?;
            sorted_people.push(person);
        }

        // Serialize to JSON
        let sorted_json = serde_json::to_string_pretty(&sorted_people)
            .expect("Failed to serialize sorted results to JSON");

        println!("People sorted by last_edit (oldest first):");
        println!("{}", sorted_json);
    }

    // Example 5: Direct JSON processing without full deserialization
    {
        println!("\nExample 5: Direct JSON processing without full struct deserialization");
        
        // Parse JSON into a generic Value
        let json_value: serde_json::Value = serde_json::from_str(json_str)
            .expect("Failed to parse JSON as generic Value");
        
        // Process directly from the Value without fully deserializing to Person structs
        if let serde_json::Value::Array(people_array) = json_value {
            // Create a new transaction
            let tx = conn.transaction()?;
            
            for person_value in people_array {
                // Extract fields directly from the generic JSON Value
                let id = person_value["id"].as_i64().unwrap_or(0) as i32;
                let name = person_value["name"].as_str().unwrap_or("Unknown");
                let age = person_value["age"].as_i64().unwrap_or(0) as i32;
                let last_edit = person_value["last_edit"].as_str();
                
                // Insert directly without creating a Person struct
                tx.execute(
                    "INSERT OR IGNORE INTO people (id, name, age, last_edit) VALUES (?1, ?2, ?3, ?4)",
                    params![id, name, age, last_edit],
                )?;
            }
            
            tx.commit()?;
            println!("Inserted data directly from JSON without full struct deserialization");
        }
    }
    
    // Example 6: Direct JSON generation from database without struct
    {
        println!("\nExample 6: Direct JSON generation without full struct serialization");
        
        // Query the database
        let mut stmt = conn.prepare("SELECT id, name, age, last_edit FROM people ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            // Create a serde_json::Value directly
            let obj = serde_json::json!({
                "id": row.get::<_, i32>(0)?,
                "name": row.get::<_, String>(1)?,
                "age": row.get::<_, i32>(2)?,
                "last_edit": row.get::<_, Option<String>>(3)?
            });
            Ok(obj)
        })?;
        
        // Collect the JSON objects
        let mut json_array = Vec::new();
        for row_result in rows {
            json_array.push(row_result?);
        }
        
        // Convert to a JSON string
        let direct_json = serde_json::to_string_pretty(&json_array)
            .expect("Failed to serialize direct JSON results");
        
        println!("JSON generated directly without Person struct serialization:");
        println!("{}", direct_json);
    }

    Ok(())
}
