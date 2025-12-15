#!/bin/bash

# Script to run the blog container with production-like configuration
# This mimics DigitalOcean App Platform environment variables locally

set -e

echo "Starting production-like deployment test..."
echo "=========================================="

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

# Build the container if not already built
if command -v docker >/dev/null 2>&1; then
    echo "Building Docker image..."
    docker build -t blog:production .

    # Stop any existing container
    echo "Stopping any existing blog container..."
    docker stop blog-production 2>/dev/null || true
    docker rm blog-production 2>/dev/null || true

    # Detect host IP
    detect_host_ip

    # In production, SURREAL_HOST should include the port
    if [[ ! $HOST_IP == *:* ]]; then
        HOST_IP="$HOST_IP:8000"
    fi

    echo "Using SURREAL_HOST: $HOST_IP"

    # Run the container with production-like environment variables
    echo "Starting container with production configuration..."
    docker run -d \
        --name blog-production \
        -p 8080:8080 \
        -e RUST_ENV=production \
        -e RUST_LOG=info \
        -e LEPTOS_SITE_ADDR=0.0.0.0:8080 \
        -e LEPTOS_SITE_ROOT=site \
        -e LEPTOS_HASH_FILES=true \
        -e SURREAL_PROTOCOL=http \
        -e SURREAL_HOST=$HOST_IP \
        -e SURREAL_NS=rustblog \
        -e SURREAL_DB=rustblog \
        -e SURREAL_ROOT_USER=root \
        -e SURREAL_ROOT_PASS=root \
        blog:production

    echo ""
    echo "Container started!"
    echo "=================="
    echo "Container logs (last 20 lines):"
    echo "--------------------------------"
    docker logs --tail 20 blog-production

    echo ""
    echo "Waiting for application to be ready..."
    for i in {1..30}; do
        if curl -s http://127.0.0.1:8080 >/dev/null 2>&1; then
            echo "Application is ready at http://127.0.0.1:8080"
            echo ""
            echo "Live container logs (follow with 'docker logs -f blog-production'):"
            docker logs --tail 5 blog-production
            exit 0
        fi
        echo -n "."
        sleep 1
    done

    echo ""
    echo "Application failed to start within 30 seconds"
    echo "Full container logs:"
    docker logs blog-production
    exit 1
else
    echo "Docker not available. For local testing, use test-production-config.sh"
    echo "For production deployment on DigitalOcean:"
    echo "1. Set these environment variables in the App Platform:"
    echo "   - SURREAL_HOST=your-database-host:8000"
    echo "   - SURREAL_NS=your-namespace"
    echo "   - SURREAL_DB=your-database"
    echo "   - SURREAL_USERNAME=your-db-user"
    echo "   - SURREAL_PASSWORD=your-db-password"
    echo ""
    echo "2. The Docker image will use these automatically"
fi