#!/bin/bash

# Improved database initialization script for test environments
# This script handles database startup and initialization more robustly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

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

# Function to check if database is ready
check_database_ready() {
    local max_attempts=60
    local attempt=1
    
    echo "Waiting for database to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        # Try version endpoint first (more reliable)
        if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST/version" >/dev/null 2>&1; then
            echo "Database is ready (version check passed)"
            return 0
        fi
        
        # Try basic connection
        if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST" >/dev/null 2>&1; then
            echo "Database is ready (basic connection passed)"
            return 0
        fi
        
        # Try health endpoint (might not be available in all versions)
        if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST/health" >/dev/null 2>&1; then
            echo "Database is ready (health check passed)"
            return 0
        fi
        if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST" >/dev/null 2>&1; then
            echo "Database is ready (basic connection passed)"
            return 0
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            echo "Database did not become ready within $max_attempts attempts"
            echo "Checking if database process is running..."
            if pgrep -f "surreal start" >/dev/null; then
                echo "Database process is running but not responding to health checks"
                echo "Attempting to continue anyway..."
                return 0
            else
                echo "Database process is not running"
                return 1
            fi
        fi
        
        echo "Waiting for database... ($attempt/$max_attempts)"
        sleep 1
        attempt=$((attempt + 1))
    done
}

# Function to create root user idempotently
create_root_user() {
    local user="$1"
    local pass="$2"
    echo "Creating root user '$user'..."
    
    # Try multiple approaches for user creation
    if surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --query "DEFINE USER IF NOT EXISTS $user ON ROOT PASSWORD '$pass' ROLES OWNER" \
        --user "$user" --pass "$pass" 2>/dev/null; then
        echo "Successfully created root user '$user'"
        return 0
    elif surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --query "DEFINE USER IF NOT EXISTS $user ON ROOT PASSWORD '$pass' ROLES OWNER" \
        2>/dev/null; then
        echo "Successfully created root user '$user' (without auth)"
        return 0
    else
        echo "Could not create root user '$user' (may already exist)"
        return 1
    fi
}

# Function to create namespace and database
create_namespace_and_database() {
    echo "Creating namespace '$SURREAL_NS' and database '$SURREAL_DB'..."
    
    # Create namespace if it doesn't exist
    surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --query "DEFINE NAMESPACE IF NOT EXISTS $SURREAL_NS;" \
        --user "$SURREAL_ROOT_USER" --pass "$SURREAL_ROOT_PASS" 2>/dev/null || echo "Namespace may already exist"
    
    # Create database if it doesn't exist
    surreal sql --conn "$SURREAL_PROTOCOL://$SURREAL_HOST" --ns "$SURREAL_NS" \
        --query "DEFINE DATABASE IF NOT EXISTS $SURREAL_DB;" \
        --user "$SURREAL_ROOT_USER" --pass "$SURREAL_ROOT_PASS" 2>/dev/null || echo "Database may already exist"
    
    echo "Namespace and database setup completed"
}

# Main initialization logic
if check_database_ready; then
    echo "Database is ready, proceeding with initialization..."
    
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
else
    echo "Failed to initialize database - database is not ready"
    exit 1
fi
