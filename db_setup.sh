#!/bin/bash

# Setup script for creating the first database and user with tenant input and SQL query support

# Create directories
mkdir -p databases

# User settings - username remains fixed but ID will be UUID
USER_NAME="Josef"

# Prompt for tenant name
echo -n "Enter tenant name [test]: "
read input_tenant
TENANT=${input_tenant:-"test"}

DB_PATH="databases/${TENANT}.db"

# Create database schema
cat > create_table.sql << EOL
PRAGMA foreign_keys = ON;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role INTEGER NOT NULL DEFAULT 0,
    lastEdit TEXT NOT NULL
);
EOL

# Create the database and initialize schema
sqlite3 "$DB_PATH" < create_table.sql

# Create a test user with UUID
NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
UUID=$(uuidgen || (cat /proc/sys/kernel/random/uuid 2>/dev/null) || (python3 -c "import uuid; print(uuid.uuid4())") || echo "error-generating-uuid")

if [ "$UUID" = "error-generating-uuid" ]; then
    echo "Error: Could not generate UUID. Please install uuidgen, ensure /proc/sys/kernel/random/uuid exists, or install Python."
    exit 1
fi

sqlite3 "$DB_PATH" << EOL
INSERT OR REPLACE INTO users (id, name, role, lastEdit) 
VALUES ('$UUID', 'Josef', 2, '$NOW');
EOL

# Save UUID to environment variable for reference
USER_ID=$UUID

# Update or create .env file
if [ -f .env ]; then
  # Update existing .env file
  grep -v "^TENANT=" .env > .env.tmp
  echo "TENANT=$TENANT" >> .env.tmp
  grep -v "^USER_ID=" .env.tmp > .env
  echo "USER_ID=$UUID" >> .env
  grep -v "^API_KEY=" .env > .env.tmp
  echo "API_KEY=${TENANT}-${UUID}" >> .env.tmp
  mv .env.tmp .env
else
  # Create new .env file
  cat > .env << EOL
PORT=8080
TENANT=$TENANT
USER_ID=$UUID
API_KEY=${TENANT}-${UUID}
EOL
fi

# Print connection information
echo "========================================"
echo "Database setup complete!"
echo "Database path: $DB_PATH"
echo "Tenant: $TENANT"
echo "User ID (UUID): $UUID"
echo ""
echo "API Key for connection: ${TENANT}-${UUID}"
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
