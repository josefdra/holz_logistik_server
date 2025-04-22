/// Provides constants and utilities for working with
/// the "shipments" database table.
pub struct ShipmentTable;

impl ShipmentTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "shipments";

    /// The column name for the primary key identifier of a shipment.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for storing when a shipment was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for storing the quantity of the shipment.
    pub const COLUMN_QUANTITY: &'static str = "quantity";

    /// The column name for storing the oversize quantity of the shipment.
    pub const COLUMN_OVERSIZE_QUANTITY: &'static str = "oversizeQuantity";

    /// The column name for storing the piece count of the shipment.
    pub const COLUMN_PIECE_COUNT: &'static str = "pieceCount";

    /// The column name for storing the user id of the shipment.
    pub const COLUMN_USER_ID: &'static str = "userId";

    /// The column name for storing the contract id of the shipment.
    pub const COLUMN_CONTRACT_ID: &'static str = "contractId";

    /// The column name for storing the sawmill id of the shipment.
    pub const COLUMN_SAWMILL_ID: &'static str = "sawmillId";

    /// The column name for storing the location id of the shipment.
    pub const COLUMN_LOCATION_ID: &'static str = "locationId";

    /// The column name for the deleted status.
    pub const COLUMN_DELETED: &'static str = "deleted";

    /// SQL statement for creating the shipments table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} TEXT NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL,
                {} INTEGER NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL,
                FOREIGN KEY ({}) REFERENCES users(id),
                FOREIGN KEY ({}) REFERENCES contracts(id),
                FOREIGN KEY ({}) REFERENCES sawmills(id),
                FOREIGN KEY ({}) REFERENCES locations(id)
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_QUANTITY,
            Self::COLUMN_OVERSIZE_QUANTITY,
            Self::COLUMN_PIECE_COUNT,
            Self::COLUMN_USER_ID,
            Self::COLUMN_CONTRACT_ID,
            Self::COLUMN_SAWMILL_ID,
            Self::COLUMN_LOCATION_ID,
            Self::COLUMN_DELETED,
            Self::COLUMN_USER_ID,
            Self::COLUMN_CONTRACT_ID,
            Self::COLUMN_SAWMILL_ID,
            Self::COLUMN_LOCATION_ID
        )
    }
}
