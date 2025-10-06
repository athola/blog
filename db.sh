#!/bin/sh

# Start SurrealDB with the configured root credentials so subsequent auth succeeds
ROOT_USER=${SURREAL_ROOT_USER:-root}
ROOT_PASS=${SURREAL_ROOT_PASS:-root}
env SURREAL_EXPERIMENTAL_GRAPHQL=true \
    surreal start \
        --log info \
        --user "$ROOT_USER" \
        --pass "$ROOT_PASS" \
        --bind 127.0.0.1:8000 \
        surrealkv:rustblog.db &

# Store the process ID
DB_PID=$!

# Wait for database to be ready
echo "Waiting for database to start..."
i=1
while [ $i -le 30 ]; do
    if curl -s http://127.0.0.1:8000/health >/dev/null 2>&1; then
        echo "Database is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Database did not start within 30 seconds"
        exit 1
    fi
    echo "Waiting for database... ($i/30)"
    sleep 1
    i=$((i + 1))
done

# Run initialization script to create namespace and database
echo "Running database initialization..."
./init-db.sh

# Bring the database process to foreground
wait $DB_PID
