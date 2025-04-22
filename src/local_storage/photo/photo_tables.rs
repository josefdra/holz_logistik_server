/// Provides constants and utilities for working with
/// the "photos" database table.
pub struct PhotoTable;

impl PhotoTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "photos";

    /// The column name for the primary key identifier of a photo.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for the timestamp when a photo was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for storing the binary photo data
    pub const COLUMN_PHOTO: &'static str = "photoFile";

    /// The column name for storing the location id of the photo.
    pub const COLUMN_LOCATION_ID: &'static str = "locationId";

    /// The column name for the deleted status.
    pub const COLUMN_DELETED: &'static str = "deleted";

    /// SQL statement for creating the photos table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} TEXT NOT NULL,
                {} BLOB NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL,
                FOREIGN KEY ({}) REFERENCES locations(id)
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_PHOTO,
            Self::COLUMN_LOCATION_ID,
            Self::COLUMN_DELETED,
            Self::COLUMN_LOCATION_ID
        )
    }
}
