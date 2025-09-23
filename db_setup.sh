#!/bin/bash

# Setup script for creating the database and user with tenant input

# Create directories
mkdir -p databases

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
	ownerInformation TEXT,
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
	userId TEXT NOT NULL,
	contractId TEXT NOT NULL,
	sawmillId TEXT NOT NULL,
	locationId TEXT NOT NULL,
	arrivalAtServer INTEGER NOT NULL,
	deleted INTEGER DEFAULT 0,
	additionalInfo Text
);
EOL

# Create the database and initialize schema
sqlite3 "$DB_PATH" < create_table.sql

# Check if users exist
DRIVER_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE name='driver';")
BOSS_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE name='boss';")
ADMIN_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE name='admin';")

# Get current timestamp in seconds since epoch (UTC integer format)
UTC_TIMESTAMP=$(date +%s)

# Create a test user with UUID only if no user exists
if [ "$DRIVER_EXISTS" -eq "0" ]; then
	DRIVER_UUID=$(uuidgen || (cat /proc/sys/kernel/random/uuid 2>/dev/null) || (python3 -c "import uuid; print(uuid.uuid4())") || echo "error-generating-uuid")

	sqlite3 "$DB_PATH" << EOL
	INSERT INTO users (id, name, role, lastEdit, arrivalAtServer) 
	VALUES ('$DRIVER_UUID', 'driver', 0, $UTC_TIMESTAMP, $UTC_TIMESTAMP);
EOL
	echo "Created new driver user with ID: $DRIVER_UUID"
else
	# Get existing user ID
	DRIVER_UUID=$(sqlite3 "$DB_PATH" "SELECT id FROM users WHERE name='driver' LIMIT 1;")
	echo "driver user already exists with ID: $DRIVER_UUID"
fi

if [ "$BOSS_EXISTS" -eq "0" ]; then
	BOSS_UUID=$(uuidgen || (cat /proc/sys/kernel/random/uuid 2>/dev/null) || (python3 -c "import uuid; print(uuid.uuid4())") || echo "error-generating-uuid")

	sqlite3 "$DB_PATH" << EOL
	INSERT INTO users (id, name, role, lastEdit, arrivalAtServer) 
	VALUES ('$BOSS_UUID', 'boss', 1, $UTC_TIMESTAMP, $UTC_TIMESTAMP);
EOL
	echo "Created new boss user with ID: $BOSS_UUID"
else
	# Get existing user ID
	BOSS_UUID=$(sqlite3 "$DB_PATH" "SELECT id FROM users WHERE name='boss' LIMIT 1;")
	echo "boss user already exists with ID: $BOSS_UUID"
fi

if [ "$ADMIN_EXISTS" -eq "0" ]; then
	ADMIN_UUID=$(uuidgen || (cat /proc/sys/kernel/random/uuid 2>/dev/null) || (python3 -c "import uuid; print(uuid.uuid4())") || echo "error-generating-uuid")

	sqlite3 "$DB_PATH" << EOL
	INSERT INTO users (id, name, role, lastEdit, arrivalAtServer) 
	VALUES ('$ADMIN_UUID', 'admin', 2, $UTC_TIMESTAMP, $UTC_TIMESTAMP);
EOL
	echo "Created new admin user with ID: $ADMIN_UUID"
else
	# Get existing user ID
	ADMIN_UUID=$(sqlite3 "$DB_PATH" "SELECT id FROM users WHERE name='admin' LIMIT 1;")
	echo "admin user already exists with ID: $ADMIN_UUID"
fi

# Update or create .env file
if [ -f .env ]; then
  # Update existing .env file
  grep -v "^TENANT=" .env > .env.tmp
  echo "TENANT=$TENANT" >> .env.tmp
  grep -v "^DRIVER_ID=" .env.tmp > .env
  echo "DRIVER_ID=$DRIVER_UUID" >> .env
  grep -v "^DRIVER_API_KEY=" .env > .env.tmp
  echo "DRIVER_API_KEY=${TENANT}-${DRIVER_UUID}" >> .env.tmp
  grep -v "^BOSS_ID=" .env.tmp > .env
  echo "BOSS_ID=$BOSS_UUID" >> .env
  grep -v "^BOSS_API_KEY=" .env > .env.tmp
  echo "BOSS_API_KEY=${TENANT}-${BOSS_UUID}" >> .env.tmp
  grep -v "^ADMIN_ID=" .env.tmp > .env
  echo "ADMIN_ID=$ADMIN_UUID" >> .env
  grep -v "^ADMIN_API_KEY=" .env > .env.tmp
  echo "ADMIN_API_KEY=${TENANT}-${ADMIN_UUID}" >> .env.tmp
  mv .env.tmp .env
else
  # Create new .env file
	cat > .env << EOL
PORT=4000
TENANT=$TENANT
DRIVER_ID=$DRIVER_UUID
API_KEY=${TENANT}-${DRIVER_UUID}
BOSS_ID=$BOSS_UUID
API_KEY=${TENANT}-${BOSS_UUID}
ADMIN_ID=$ADMIN_UUID
API_KEY=${TENANT}-${ADMIN_UUID}
EOL
fi

# Print connection information
echo "========================================"
echo "Database setup complete!"
echo "Database path: $DB_PATH"
echo "Tenant: $TENANT"
echo ""
echo "Driver ID (UUID): $DRIVER_UUID"
echo "API Key for connection: ${TENANT}-${DRIVER_UUID}"
echo ""
echo "Boss ID (UUID): $BOSS_UUID"
echo "API Key for connection: ${TENANT}-${BOSS_UUID}"
echo ""
echo "Admin ID (UUID): $ADMIN_UUID"
echo "API Key for connection: ${TENANT}-${ADMIN_UUID}"
echo ""
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
