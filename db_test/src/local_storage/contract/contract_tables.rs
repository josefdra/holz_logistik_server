/// Provides constants and utilities for working with
/// the "contracts" database table.
pub struct ContractTable;

impl ContractTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "contracts";

    /// The column name for the primary key identifier of a contract.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for the done status of the contract.
    pub const COLUMN_DONE: &'static str = "done";

    /// The column name for the timestamp when a contract was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for the title of the contract.
    pub const COLUMN_TITLE: &'static str = "title";

    /// The column name for storing the additional info of the contract.
    pub const COLUMN_ADDITIONAL_INFO: &'static str = "additionalInfo";

    /// The column name for the timestamp when a contract starts.
    pub const COLUMN_START_DATE: &'static str = "startDate";

    /// The column name for the timestamp when a contract ends.
    pub const COLUMN_END_DATE: &'static str = "endDate";

    /// The column name for storing the available quantity of the contract.
    pub const COLUMN_AVAILABLE_QUANTITY: &'static str = "availableQuantity";

    /// The column name for storing the booked quantity of the contract.
    pub const COLUMN_BOOKED_QUANTITY: &'static str = "bookedQuantity";

    /// The column name for storing the shipped quantity of the contract.
    pub const COLUMN_SHIPPED_QUANTITY: &'static str = "shippedQuantity";

    /// SQL statement for creating the contracts table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} INTEGER NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_DONE,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_TITLE,
            Self::COLUMN_ADDITIONAL_INFO,
            Self::COLUMN_START_DATE,
            Self::COLUMN_END_DATE,
            Self::COLUMN_AVAILABLE_QUANTITY,
            Self::COLUMN_BOOKED_QUANTITY,
            Self::COLUMN_SHIPPED_QUANTITY
        )
    }
}
