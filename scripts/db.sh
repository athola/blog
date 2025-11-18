#!/bin/bash

# Improved SurrealDB startup script for test environments
# This script handles database startup more robustly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

# Configuration
ROOT_USER=${SURREAL_ROOT_USER:-root}
ROOT_PASS=${SURREAL_ROOT_PASS:-root}
DB_FILE=${DB_FILE:-rustblog.db}
SURREAL_HOST=${SURREAL_HOST:-127.0.0.1:8000}

# Clean up any existing database processes
echo "Cleaning up existing database processes..."
pkill -f "surreal start" 2>/dev/null || true
pkill -f "surrealkv" 2>/dev/null || true
sleep 2

# Clean up database file if it exists and we want a fresh start
if [ -f "$DB_FILE" ] || [ -d "$DB_FILE" ]; then
    echo "Removing existing database file..."
    rm -rf "$DB_FILE"
fi

# Start SurrealDB with improved configuration
echo "Starting SurrealDB..."
echo "Host: $SURREAL_HOST"
echo "User: $ROOT_USER"
echo "Database file: $DB_FILE"

# Start SurrealDB in background
env SURREAL_EXPERIMENTAL_GRAPHQL=true \
    surreal start \
        --log info \
        --user "$ROOT_USER" \
        --pass "$ROOT_PASS" \
        --bind 127.0.0.1:8000 \
        "surrealkv:$DB_FILE" &

# Store the process ID
DB_PID=$!

# Give the database a moment to start
sleep 3

# Function to check if database is ready
check_database_ready() {
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for database to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        # Check if process is still running
        if ! kill -0 $DB_PID 2>/dev/null; then
            echo "Database process died"
            return 1
        fi
        
        # Try version endpoint first (more reliable)
        if curl -s --connect-timeout 2 --max-time 5 "http://$SURREAL_HOST/version" >/dev/null 2>&1; then
            echo "Database is ready (version check passed)"
            return 0
        fi
        
        # Try basic connection
        if curl -s --connect-timeout 2 --max-time 5 "http://$SURREAL_HOST" >/dev/null 2>&1; then
            echo "Database is ready (basic connection passed)"
            return 0
        fi
        
        # Try health endpoint (might not be available in all versions)
        if curl -s --connect-timeout 2 --max-time 5 "http://$SURREAL_HOST/health" >/dev/null 2>&1; then
            echo "Database is ready (health check passed)"
            return 0
        fi
        if curl -s --connect-timeout 2 --max-time 5 "http://$SURREAL_HOST/version" >/dev/null 2>&1; then
            echo "Database is ready (version check passed)"
            return 0
        fi
        
        # Try basic connection
        if curl -s --connect-timeout 2 --max-time 5 "http://$SURREAL_HOST" >/dev/null 2>&1; then
            echo "Database is ready (basic connection passed)"
            return 0
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            echo "Database did not become ready within $max_attempts attempts"
            echo "Database process status:"
            ps aux | grep $DB_PID || echo "Process not found"
            return 1
        fi
        
        echo "Waiting for database... ($attempt/$max_attempts)"
        sleep 1
        attempt=$((attempt + 1))
    done
}

# Wait for database to be ready
if check_database_ready; then
    echo "Database started successfully"
    
    # Run initialization script
    echo "Running database initialization..."
    if [ -f "${SCRIPT_DIR}/init-db-improved.sh" ]; then
        "${SCRIPT_DIR}/init-db-improved.sh"
    elif [ -f "${SCRIPT_DIR}/init-db.sh" ]; then
        "${SCRIPT_DIR}/init-db.sh"
    else
        echo "Warning: No initialization script found"
    fi
    
    echo "Database startup and initialization completed!"
    
    # Keep the process running in foreground
    wait $DB_PID
else
    echo "Failed to start database"
    kill $DB_PID 2>/dev/null || true
    exit 1
fi
