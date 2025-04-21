/// Provides constants and utilities for working with
/// the "locations" database table.
pub struct LocationTable;

impl LocationTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "locations";

    /// The column name for the primary key identifier of a location.
    pub const COLUMN_ID: &'static str = "id";

    /// The column name for the done status of the location.
    pub const COLUMN_DONE: &'static str = "done";

    /// The column name for the started status of the location.
    pub const COLUMN_STARTED: &'static str = "started";

    /// The column name for the timestamp when a location was last modified.
    pub const COLUMN_LAST_EDIT: &'static str = "lastEdit";

    /// The column name for latitude of the location.
    pub const COLUMN_LATITUDE: &'static str = "latitude";

    /// The column name for longitude of the location.
    pub const COLUMN_LONGITUDE: &'static str = "longitude";

    /// The column name for storing the partie number of the location.
    pub const COLUMN_PARTIE_NR: &'static str = "partieNr";

    /// The column name for the date of a location.
    pub const COLUMN_DATE: &'static str = "date";

    /// The column name for storing additional information of the location.
    pub const COLUMN_ADDITIONAL_INFO: &'static str = "additionalInfo";

    /// The column name for storing the initial quantity of the location.
    pub const COLUMN_INITIAL_QUANTITY: &'static str = "initialQuantity";

    /// The column name for storing the initial oversize quantity of the location.
    pub const COLUMN_INITIAL_OVERSIZE_QUANTITY: &'static str = "initialOversizeQuantity";

    /// The column name for storing the initial piece count of the location.
    pub const COLUMN_INITIAL_PIECE_COUNT: &'static str = "initialPieceCount";

    /// The column name for storing the current quantity of the location.
    pub const COLUMN_CURRENT_QUANTITY: &'static str = "currentQuantity";

    /// The column name for storing the current oversize quantity of the location.
    pub const COLUMN_CURRENT_OVERSIZE_QUANTITY: &'static str = "currentOversizeQuantity";

    /// The column name for storing the current piece count of the location.
    pub const COLUMN_CURRENT_PIECE_COUNT: &'static str = "currentPieceCount";

    /// The column name for storing the contract id of the location.
    pub const COLUMN_CONTRACT_ID: &'static str = "contractId";

    /// SQL statement for creating the locations table with the defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT PRIMARY KEY NOT NULL,
                {} INTEGER NOT NULL,
                {} INTEGER NOT NULL,
                {} TEXT NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL,
                {} INTEGER NOT NULL,
                {} REAL NOT NULL,
                {} REAL NOT NULL,
                {} INTEGER NOT NULL,
                {} TEXT NOT NULL,
                FOREIGN KEY ({}) REFERENCES {}({})
            )",
            Self::TABLE_NAME,
            Self::COLUMN_ID,
            Self::COLUMN_DONE,
            Self::COLUMN_STARTED,
            Self::COLUMN_LAST_EDIT,
            Self::COLUMN_LATITUDE,
            Self::COLUMN_LONGITUDE,
            Self::COLUMN_PARTIE_NR,
            Self::COLUMN_DATE,
            Self::COLUMN_ADDITIONAL_INFO,
            Self::COLUMN_INITIAL_QUANTITY,
            Self::COLUMN_INITIAL_OVERSIZE_QUANTITY,
            Self::COLUMN_INITIAL_PIECE_COUNT,
            Self::COLUMN_CURRENT_QUANTITY,
            Self::COLUMN_CURRENT_OVERSIZE_QUANTITY,
            Self::COLUMN_CURRENT_PIECE_COUNT,
            Self::COLUMN_CONTRACT_ID,
            Self::COLUMN_CONTRACT_ID, ContractTable::TABLE_NAME, ContractTable::COLUMN_ID
        )
    }
}

/// Provides the junction between location and sawmill table
pub struct LocationSawmillJunctionTable;

impl LocationSawmillJunctionTable {
    /// The name of the database table
    pub const TABLE_NAME: &'static str = "locationSawmillJunction";

    /// The column name for the locationId.
    pub const COLUMN_LOCATION_ID: &'static str = "locationId";

    /// The column name for the sawmillId.
    pub const COLUMN_SAWMILL_ID: &'static str = "sawmillId";

    /// The column that stores if the relation is for oversize sawmills.
    pub const COLUMN_IS_OVERSIZE: &'static str = "isOversize";

    /// SQL statement for creating the locationSawmillJunction table with the
    /// defined schema.
    pub fn create_table() -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (
                {} TEXT NOT NULL,
                {} TEXT NOT NULL,
                {} INTEGER NOT NULL,
                PRIMARY KEY ({}, {}, {}),
                FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE CASCADE,
                FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE CASCADE
            )",
            Self::TABLE_NAME,
            Self::COLUMN_LOCATION_ID,
            Self::COLUMN_SAWMILL_ID,
            Self::COLUMN_IS_OVERSIZE,
            Self::COLUMN_LOCATION_ID, Self::COLUMN_SAWMILL_ID, Self::COLUMN_IS_OVERSIZE,
            Self::COLUMN_LOCATION_ID, LocationTable::TABLE_NAME, LocationTable::COLUMN_ID,
            Self::COLUMN_SAWMILL_ID, SawmillTable::TABLE_NAME, SawmillTable::COLUMN_ID
        )
    }
}
