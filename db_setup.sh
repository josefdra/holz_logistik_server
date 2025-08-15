#!/bin/bash

# Setup script for creating the database and user with tenant input

# Create directories
mkdir -p databases

# User settings - username remains fixed but ID will be UUID
USER_NAME="driver"

# Prompt for tenant name
echo -n "Enter tenant name [test]: "
read input_tenant
TENANT=${input_tenant:-"test"}

DB_PATH="databases/${TENANT}.db"

# Create database schema
cat > create_table.sql << EOL
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 10000;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    lastEdit INTEGER NOT NULL,
    role INTEGER NOT NULL,
    name TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Contracts table
CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY NOT NULL,
    done INTEGER NOT NULL,
    lastEdit INTEGER NOT NULL,
    title TEXT NOT NULL,
    additionalInfo TEXT NOT NULL,
    startDate INTEGER NOT NULL,
    endDate INTEGER NOT NULL,
    availableQuantity REAL NOT NULL,
    bookedQuantity REAL NOT NULL,
    shippedQuantity REAL NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Sawmills table
CREATE TABLE IF NOT EXISTS sawmills (
    id TEXT PRIMARY KEY NOT NULL,
    lastEdit INTEGER NOT NULL,
    name TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Locations table
CREATE TABLE IF NOT EXISTS locations (
    id TEXT PRIMARY KEY NOT NULL,
    done INTEGER NOT NULL,
    started INTEGER NOT NULL,
    lastEdit INTEGER NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    partieNr TEXT NOT NULL,
    date INTEGER NOT NULL,
    additionalInfo TEXT NOT NULL,
    initialQuantity REAL NOT NULL,
    initialOversizeQuantity REAL NOT NULL,
    initialPieceCount INTEGER NOT NULL,
    currentQuantity REAL NOT NULL,
    currentOversizeQuantity REAL NOT NULL,
    currentPieceCount INTEGER NOT NULL,
    contractId TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Location-Sawmill Junction table
CREATE TABLE IF NOT EXISTS locationSawmillJunction (
    locationId TEXT NOT NULL,
    sawmillId TEXT NOT NULL,
    isOversize INTEGER NOT NULL,
    PRIMARY KEY (locationId, sawmillId, isOversize),
    FOREIGN KEY (locationId) REFERENCES locations(id) ON DELETE CASCADE,
    FOREIGN KEY (sawmillId) REFERENCES sawmills(id) ON DELETE CASCADE
);

-- Notes table
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY NOT NULL,
    lastEdit INTEGER NOT NULL,
    text TEXT NOT NULL,
    userId TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Photos table
CREATE TABLE IF NOT EXISTS photos (
    id TEXT PRIMARY KEY NOT NULL,
    lastEdit INTEGER NOT NULL,
    photoFile BLOB NOT NULL,
    locationId TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);

-- Shipments table
CREATE TABLE IF NOT EXISTS shipments (
    id TEXT PRIMARY KEY NOT NULL,
    lastEdit INTEGER NOT NULL,
    quantity REAL NOT NULL,
    oversizeQuantity REAL NOT NULL,
    pieceCount INTEGER NOT NULL,
    additionalInfo TEXT,
    userId TEXT NOT NULL,
    contractId TEXT NOT NULL,
    sawmillId TEXT NOT NULL,
    locationId TEXT NOT NULL,
    arrivalAtServer INTEGER NOT NULL,
    deleted INTEGER DEFAULT 0
);
EOL

# Create the database and initialize schema
sqlite3 "$DB_PATH" < create_table.sql

# Check if josef user exists
JOSEF_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE name='driver';")

# Create a test user with UUID only if no josef user exists
if [ "$JOSEF_EXISTS" -eq "0" ]; then
    # Get current timestamp in seconds since epoch (UTC integer format)
    UTC_TIMESTAMP=$(date +%s)
    UUID=$(uuidgen || (cat /proc/sys/kernel/random/uuid 2>/dev/null) || (python3 -c "import uuid; print(uuid.uuid4())") || echo "error-generating-uuid")

    if [ "$UUID" = "error-generating-uuid" ]; then
        echo "Error: Could not generate UUID. Please install uuidgen, ensure /proc/sys/kernel/random/uuid exists, or install Python."
        exit 1
    fi

    sqlite3 "$DB_PATH" << EOL
    INSERT INTO users (id, name, role, lastEdit, arrivalAtServer) 
    VALUES ('$UUID', 'driver', 0, $UTC_TIMESTAMP, $UTC_TIMESTAMP);
EOL
    echo "Created new driver user with ID: $UUID"
    USER_ID=$UUID
else
    # Get existing josef user ID
    USER_ID=$(sqlite3 "$DB_PATH" "SELECT id FROM users WHERE name='driver' LIMIT 1;")
    echo "driver user already exists with ID: $USER_ID"
fi

# Update or create .env file
if [ -f .env ]; then
  # Update existing .env file
  grep -v "^TENANT=" .env > .env.tmp
  echo "TENANT=$TENANT" >> .env.tmp
  grep -v "^USER_ID=" .env.tmp > .env
  echo "USER_ID=$USER_ID" >> .env
  grep -v "^API_KEY=" .env > .env.tmp
  echo "API_KEY=${TENANT}-${USER_ID}" >> .env.tmp
  mv .env.tmp .env
else
  # Create new .env file
  cat > .env << EOL
PORT=8080
TENANT=$TENANT
USER_ID=$USER_ID
API_KEY=${TENANT}-${USER_ID}
EOL
fi

# Print connection information
echo "========================================"
echo "Database setup complete!"
echo "Database path: $DB_PATH"
echo "Tenant: $TENANT"
echo "User ID (UUID): $USER_ID"
echo ""
echo "API Key for connection: ${TENANT}-${USER_ID}"
echo "Updated .env file"
echo "========================================"

# Clean up
rm create_table.sql

# SQL query mode
echo ""
echo "Entering SQL query mode. Type your queries or 'exit' to quit."
echo ""

while true; do
    echo -n "SQL> "
    read -r query
    
    if [ "$query" = "exit" ]; then
        echo "Exiting. You can now start your server with: cargo run"
        break
    elif [ -n "$query" ]; then
        echo "Executing: $query"
        sqlite3 -header -column "$DB_PATH" "$query"
        echo ""
    fi
done
