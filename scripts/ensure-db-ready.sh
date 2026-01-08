#!/bin/bash

# Database readiness script for CI/CD environments
# This script ensures the database is running and initialized properly

set -euo pipefail

# Configuration
SURREAL_HOST=${SURREAL_HOST:-127.0.0.1:8000}
SURREAL_PROTOCOL=${SURREAL_PROTOCOL:-http}
SURREAL_ROOT_USER=${SURREAL_ROOT_USER:-root}
SURREAL_ROOT_PASS=${SURREAL_ROOT_PASS:-root}
SURREAL_NS=${SURREAL_NS:-rustblog}
SURREAL_DB=${SURREAL_DB:-rustblog}
SURREAL_NAMESPACE_USER=${SURREAL_NAMESPACE_USER:-}
SURREAL_NAMESPACE_PASS=${SURREAL_NAMESPACE_PASS:-}
DB_FILE=${DB_FILE:-rustblog.db}
PID_DIR=${PID_DIR:-target/tmp}

echo "Ensuring database is ready..."
echo "Host: $SURREAL_PROTOCOL://$SURREAL_HOST"
echo "Namespace: $SURREAL_NS"
echo "Database: $SURREAL_DB"

# Function to check if database is running
is_database_running() {
    # Check for surreal processes
    if pgrep -f "surreal start" >/dev/null 2>&1; then
        return 0
    fi
    
    # Check if database is responding
    if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST/version" >/dev/null 2>&1; then
        return 0
    fi
    
    return 1
}

# Function to start database
start_database() {
    echo "Starting database..."

    # Check if surreal is available
    if ! command -v surreal >/dev/null 2>&1; then
        echo "Error: SurrealDB not found. Please install SurrealDB first."
        echo "Visit: https://surrealdb.com/install or run 'make install-surrealdb'"
        return 1
    fi

    # Show SurrealDB version
    echo "SurrealDB version: $(surreal --version 2>/dev/null || echo "unknown")"
    
    mkdir -p "$PID_DIR"

    # Clean up any existing processes
    "$(dirname "$0")/stop-db.sh" || true
    
    # Clean up database file if it exists
    if [ -f "$DB_FILE" ] || [ -d "$DB_FILE" ]; then
        echo "Removing existing database file..."
        rm -rf "$DB_FILE"
    fi
    
    # Start SurrealDB
    echo "Starting SurrealDB server..."
    env SURREAL_EXPERIMENTAL_GRAPHQL=true \
        surreal start \
            --log info \
            --user "$SURREAL_ROOT_USER" \
            --pass "$SURREAL_ROOT_PASS" \
            --bind 127.0.0.1:8000 \
            "surrealkv:$DB_FILE" &
    
    # Store the process ID
    DB_PID=$!
    PID_FILE=$(mktemp "$PID_DIR/blog_db_pid.XXXXXX")
    echo "$DB_PID" > "$PID_FILE"
    echo "$DB_PID" > "$PID_DIR/blog_db_pid.latest"
    
    # Wait for database to be ready
    local max_attempts=30
    local attempt=1
    
    echo "Waiting for database to start..."
    while [ $attempt -le $max_attempts ]; do
        if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST/version" >/dev/null 2>&1; then
            echo "Database started successfully"
            return 0
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            echo "Failed to start database within $max_attempts attempts"
            return 1
        fi
        
        echo "Waiting for database... ($attempt/$max_attempts)"
        sleep 1
        attempt=$((attempt + 1))
    done
}

# Function to initialize database
initialize_database() {
    echo "Initializing database..."

    # Try to create the root user
    if echo "DEFINE USER IF NOT EXISTS $SURREAL_ROOT_USER ON ROOT PASSWORD '$SURREAL_ROOT_PASS' ROLES OWNER;" | surreal sql --endpoint "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --username "$SURREAL_ROOT_USER" --password "$SURREAL_ROOT_PASS" 2>/dev/null; then
        echo "Root user created successfully"
    else
        echo "Root user may already exist or was created differently"
    fi

    # Create namespace
    echo "DEFINE NAMESPACE IF NOT EXISTS $SURREAL_NS;" | surreal sql --endpoint "$SURREAL_PROTOCOL://$SURREAL_HOST" \
        --username "$SURREAL_ROOT_USER" --password "$SURREAL_ROOT_PASS" 2>/dev/null || echo "Namespace may already exist"

    # Create database
    echo "DEFINE DATABASE IF NOT EXISTS $SURREAL_DB;" | surreal sql --endpoint "$SURREAL_PROTOCOL://$SURREAL_HOST" --namespace "$SURREAL_NS" \
        --username "$SURREAL_ROOT_USER" --password "$SURREAL_ROOT_PASS" 2>/dev/null || echo "Database may already exist"

    # Create namespace-level user if credentials are provided
    if [ -n "$SURREAL_NAMESPACE_USER" ] && [ -n "$SURREAL_NAMESPACE_PASS" ]; then
        echo "Creating namespace-level user: $SURREAL_NAMESPACE_USER"
        echo "DEFINE USER IF NOT EXISTS $SURREAL_NAMESPACE_USER ON NAMESPACE PASSWORD '$SURREAL_NAMESPACE_PASS' ROLES OWNER;" | surreal sql --endpoint "$SURREAL_PROTOCOL://$SURREAL_HOST" --namespace "$SURREAL_NS" \
            --username "$SURREAL_ROOT_USER" --password "$SURREAL_ROOT_PASS" 2>/dev/null || echo "Namespace user may already exist or creation failed"
    fi

    echo "Database initialization completed"
}

# Main logic
if is_database_running; then
    echo "Database is already running"
    
    # Test connectivity
    if curl -s --connect-timeout 2 --max-time 5 "$SURREAL_PROTOCOL://$SURREAL_HOST/version" >/dev/null 2>&1; then
        echo "Database is responding"
        initialize_database
    else
        echo "Database process is running but not responding, restarting..."
        start_database
        initialize_database
    fi
else
    echo "Database is not running, starting it..."
    start_database
    initialize_database
fi

echo "Database is ready and initialized!"
