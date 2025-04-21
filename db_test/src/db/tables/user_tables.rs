/// Provides constants and utilities for working with
/// the "users" database table.
pub struct UserTable;

impl UserTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "users";
  
    /// The column name for the primary key identifier of a user.
    pub const COLUMN_ID: &'static str = "id";
  
    /// The column name for storing the timestamp when a user was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";
  
    /// The column name for storing the user role (stored as INTEGER).
    pub const COLUMN_ROLE: &'static str = "role";
  
    /// The column name for storing the user's name.
    pub const COLUMN_NAME: &'static str = "name";
  
    /// SQL statement for creating the users table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL,
                {} TEXT NOT NULL
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_ROLE,
            Self::COLUMN_NAME
        )
    }
}
