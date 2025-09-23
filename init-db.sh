#!/bin/bash

# Database initialization script for SurrealDB 2.3.7
# This script handles the idempotent creation of root users using proper SurrealQL syntax

set -e

# Configuration from environment variables with defaults
SURREAL_HOST="${SURREAL_HOST:-127.0.0.1:8000}"
SURREAL_PROTOCOL="${SURREAL_PROTOCOL:-http}"
SURREAL_ROOT_USER="${SURREAL_ROOT_USER:-root}"
SURREAL_ROOT_PASS="${SURREAL_ROOT_PASS:-root}"
SURREAL_NS="${SURREAL_NS:-rustblog}"
SURREAL_DB="${SURREAL_DB:-rustblog}"

echo "Initializing SurrealDB database..."
echo "Host: $SURREAL_PROTOCOL://$SURREAL_HOST"
echo "Namespace: $SURREAL_NS"
echo "Database: $SURREAL_DB"

# Function to create root user idempotently
create_root_user() {
    local user="$1"
    local pass="$2"
    echo "Creating root user '$user'..."
    
    # Use IF NOT EXISTS for idempotent user creation at ROOT level
    if surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --query "DEFINE USER IF NOT EXISTS $user ON ROOT PASSWORD '$pass' ROLES OWNER" \
        --user "$user" --pass "$pass" 2>/dev/null; then
        echo "Successfully created root user '$user'"
        return 0
    else
        echo "Failed to create root user '$user' (may already exist or insufficient permissions)"
        return 1
    fi
}

# Function to create namespace and database
create_namespace_and_database() {
    echo "Creating namespace '$SURREAL_NS' and database '$SURREAL_DB'..."
    
    # Create namespace if it doesn't exist
    surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --query "DEFINE NAMESPACE IF NOT EXISTS $SURREAL_NS;" \
        --user "$SURREAL_ROOT_USER" --pass "$SURREAL_ROOT_PASS" 2>/dev/null || true
    
    # Create database if it doesn't exist
    surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" --ns "$SURREAL_NS" \
        --query "DEFINE DATABASE IF NOT EXISTS $SURREAL_DB;" \
        --user "$SURREAL_ROOT_USER" --pass "$SURREAL_ROOT_PASS" 2>/dev/null || true
    
    echo "Namespace and database setup completed"
}

# Wait for database to be ready
echo "Waiting for database to be ready..."
i=1
while [ $i -le 30 ]; do
    if curl -s "$SURREAL_PROTOCOL://$SURREAL_HOST/health" >/dev/null 2>&1; then
        echo "Database is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Database did not become ready within 30 seconds"
        exit 1
    fi
    echo "Waiting for database... ($i/30)"
    sleep 1
    i=$((i + 1))
done

# Try to create the root user (this is idempotent)
if create_root_user "$SURREAL_ROOT_USER" "$SURREAL_ROOT_PASS"; then
    echo "Root user setup completed successfully"
else
    echo "Could not create root user, but continuing anyway..."
    echo "   This might be normal if the database was initialized with different credentials"
fi

# Create namespace and database
create_namespace_and_database

echo "Database initialization completed!"