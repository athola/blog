#!/bin/bash
# SSH tunnel wrapper for SurrealDB connectivity.
# Establishes a persistent tunnel to the SurrealDB droplet via autossh,
# then starts the blog application with SURREAL_ADDRESS pointing to the tunnel.

TUNNEL_HOST="${TUNNEL_HOST:-}"
TUNNEL_PORT="${TUNNEL_PORT:-22}"
TUNNEL_USER="${TUNNEL_USER:-root}"
TUNNEL_KEY="/app/secrets/tunnel_key"
REMOTE_PORT="${REMOTE_PORT:-8000}"
LOCAL_PORT="${LOCAL_PORT:-9000}"

echo "=== Blog Tunnel Wrapper ==="

# If no tunnel host configured, skip tunnel and run app directly
if [ -z "$TUNNEL_HOST" ]; then
    echo "No TUNNEL_HOST set, starting app without tunnel"
    exec /app/blog
fi

# Write SSH key from environment to file
if [ -n "${TUNNEL_KEY_SECRET:-}" ]; then
    mkdir -p /app/secrets
    # Use printf to avoid shell interpretation of key content
    printf '%s\n' "$TUNNEL_KEY_SECRET" > "$TUNNEL_KEY"
    chmod 600 "$TUNNEL_KEY"
    unset TUNNEL_KEY_SECRET
    echo "SSH key written to $TUNNEL_KEY"
else
    echo "ERROR: TUNNEL_KEY_SECRET not set"
    exec /app/blog
fi

# Create SSH directory for appuser
mkdir -p ~/.ssh 2>/dev/null || true

echo "Starting SSH tunnel: localhost:${LOCAL_PORT} -> ${TUNNEL_HOST}:${REMOTE_PORT}"

# Start autossh with persistent SSH tunnel
# AUTOSSH_GATETIME=0: don't wait before first connection (required for non-interactive)
export AUTOSSH_GATETIME=0
autossh -f -N \
    -i "$TUNNEL_KEY" \
    -L "${LOCAL_PORT}:localhost:${REMOTE_PORT}" \
    -p "$TUNNEL_PORT" \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o ServerAliveInterval=15 \
    -o ServerAliveCountMax=3 \
    -o ExitOnForwardFailure=yes \
    -o ConnectTimeout=10 \
    "${TUNNEL_USER}@${TUNNEL_HOST}" 2>&1

autossh_exit=$?
if [ $autossh_exit -ne 0 ]; then
    echo "WARNING: autossh exited with code $autossh_exit, starting app without tunnel"
    exec /app/blog
fi

echo "autossh started, waiting for tunnel..."

# Wait for tunnel to become healthy
MAX_HEALTH_WAIT=30
elapsed=0
while [ $elapsed -lt $MAX_HEALTH_WAIT ]; do
    if curl -sf "http://localhost:${LOCAL_PORT}/health" > /dev/null 2>&1; then
        echo "Tunnel healthy after ${elapsed}s"
        break
    fi
    sleep 2
    elapsed=$((elapsed + 2))
done

if [ $elapsed -ge $MAX_HEALTH_WAIT ]; then
    echo "WARNING: Tunnel not healthy after ${MAX_HEALTH_WAIT}s"
fi

# Override SURREAL_ADDRESS for the app to use the tunnel
export SURREAL_ADDRESS="http://localhost:${LOCAL_PORT}"
echo "Starting app with SURREAL_ADDRESS=${SURREAL_ADDRESS}"

exec /app/blog
