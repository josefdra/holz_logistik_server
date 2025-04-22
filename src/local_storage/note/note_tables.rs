/// Provides constants and utilities for working with
/// the "notes" database table.
pub struct NoteTable;

impl NoteTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "notes";

    /// The column name for the primary key identifier of a note.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for storing the timestamp when a note was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for storing the text of the comment.
    pub const COLUMN_TEXT: &'static str = "text";

    /// The column name for storing the id of the user that created the note.
    pub const COLUMN_USER_ID: &'static str = "userId";

    /// The column name for the deleted status.
    pub const COLUMN_DELETED: &'static str = "deleted";

    /// SQL statement for creating the notes table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL,
                FOREIGN KEY ({}) REFERENCES users(id)
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_TEXT,
            Self::COLUMN_USER_ID,
            Self::COLUMN_DELETED,
            Self::COLUMN_USER_ID
        )
    }
}
