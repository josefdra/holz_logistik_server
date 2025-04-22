/// Provides constants and utilities for working with
/// the "sawmills" database table.
pub struct SawmillTable;

impl SawmillTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "sawmills";

    /// The column name for the primary key identifier of a sawmill.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for storing when a sawmill was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for storing the sawmill's name.
    pub const COLUMN_NAME: &'static str = "name";

    /// The column name for the deleted status.
    pub const COLUMN_DELETED: &'static str = "deleted";

    /// SQL statement for creating the sawmills table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_NAME,
            Self::COLUMN_DELETED
        )
    }
}
