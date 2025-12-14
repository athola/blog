#!/bin/bash

# Script to test the application with production-like configuration
# This mimics DigitalOcean App Platform environment variables locally

set -e

echo "Testing production-like configuration..."
echo "======================================="

# Detect environment and set HOST_IP accordingly
detect_host_ip() {
    # Check if we're in WSL2
    if grep -q Microsoft /proc/version 2>/dev/null; then
        # WSL2 environment - use WSL IP
        HOST_IP=$(ip addr | grep -E "inet.*scope global" | grep -E "172\.|10\." | awk '{print $2}' | cut -d'/' -f1 | head -1)
        echo "WSL2 environment detected, using IP: $HOST_IP"
    # Check if we're in Docker (GitHub Actions or similar)
    elif [ -f /.dockerenv ] || grep -q docker /proc/1/cgroup 2>/dev/null; then
        # Inside Docker - use host.docker.internal or service name
        if ping -c 1 host.docker.internal >/dev/null 2>&1; then
            HOST_IP="host.docker.internal"
        else
            # In Docker Compose or similar, try to use service name
            HOST_IP="surrealdb"
        fi
        echo "Docker environment detected, using: $HOST_IP"
    # Check if we're on a cloud provider with metadata service
    elif curl -s --connect-timeout 1 http://169.254.169.254 >/dev/null 2>&1; then
        # Cloud environment - use environment variable or localhost
        HOST_IP="${SURREAL_HOST:-127.0.0.1:8000}"
        echo "Cloud environment detected, using SURREAL_HOST: $HOST_IP"
    # Default for local testing
    else
        # Try common localhost equivalents
        if ping -c 1 127.0.0.1 >/dev/null 2>&1; then
            HOST_IP="127.0.0.1"
        elif ping -c 1 localhost >/dev/null 2>&1; then
            HOST_IP="localhost"
        else
            HOST_IP="${SURREAL_HOST:-127.0.0.1}"
        fi
        echo "Local environment detected, using: $HOST_IP"
    fi
}

# Check if SurrealDB is running
if ! pgrep -f "surreal start" >/dev/null; then
    # Check if it's accessible remotely (e.g., in CI/CD)
    echo "SurrealDB not running locally. Checking remote connection..."
    if [ -n "$SURREAL_HOST" ]; then
        echo "Using SURREAL_HOST from environment: $SURREAL_HOST"
    else
        echo "ERROR: SurrealDB is not running and SURREAL_HOST not set!"
        echo "Start SurrealDB with: ~/.surrealdb/surreal start --log info --user root --pass root memory --bind 0.0.0.0:8000"
        echo "Or set SURREAL_HOST environment variable for remote database"
        exit 1
    fi
else
    echo "✓ SurrealDB is running locally"
fi

# Detect environment and get host IP
detect_host_ip

# If HOST_IP doesn't include port, add it
if [[ ! $HOST_IP == *:* ]] && [ -z "$SURREAL_HOST" ]; then
    HOST_IP="$HOST_IP:8000"
fi

# Use SURREAL_HOST if it's set, otherwise use detected HOST_IP
FINAL_HOST="${SURREAL_HOST:-$HOST_IP}"

echo "Final SURREAL_HOST will be: $FINAL_HOST"

# Test connection to SurrealDB
echo "Testing SurrealDB connection..."
if curl -s --connect-timeout 2 http://${FINAL_HOST}/health >/dev/null 2>&1; then
    echo "✓ SurrealDB is accessible at http://$FINAL_HOST"
else
    echo "✗ Cannot connect to SurrealDB at http://$FINAL_HOST"
    echo "Please check if SurrealDB is running and accessible"
    exit 1
fi

# Set production environment variables
export RUST_ENV=production
export RUST_LOG=info
export LEPTOS_SITE_ADDR=127.0.0.1:8080
export LEPTOS_SITE_ROOT=site
export LEPTOS_HASH_FILES=true
export SURREAL_PROTOCOL=http
export SURREAL_HOST=$FINAL_HOST
export SURREAL_NS="${SURREAL_NS:-rustblog}"
export SURREAL_DB="${SURREAL_DB:-rustblog}"
export SURREAL_ROOT_USER="${SURREAL_ROOT_USER:-root}"
export SURREAL_ROOT_PASS="${SURREAL_ROOT_PASS:-root}"

echo ""
echo "Environment variables set:"
echo "--------------------------"
echo "RUST_ENV=$RUST_ENV"
echo "SURREAL_HOST=$SURREAL_HOST"
echo "SURREAL_NS=$SURREAL_NS"
echo "SURREAL_DB=$SURREAL_DB"

echo ""
echo "Building and running application..."
echo "---------------------------------"

# Build the application
if [ ! -f "target/release/server" ]; then
    echo "Building release binary..."
    cargo leptos build --release
else
    echo "Release binary exists"
fi

# Run the application
echo ""
echo "Starting server with production configuration..."
echo "Server will be available at: http://127.0.0.1:8080"
echo "Press Ctrl+C to stop"
echo ""

# Run in foreground
exec target/release/server