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
# Supports both base64-encoded (TUNNEL_KEY_B64) and raw (TUNNEL_KEY_SECRET) formats.
# DO App Platform may strip newlines from env vars, so base64 is preferred.
if [ -n "${TUNNEL_KEY_B64:-}" ]; then
    mkdir -p /app/secrets
    echo "$TUNNEL_KEY_B64" | base64 -d > "$TUNNEL_KEY"
    chmod 600 "$TUNNEL_KEY"
    unset TUNNEL_KEY_B64
    echo "SSH key decoded from base64 to $TUNNEL_KEY"
elif [ -n "${TUNNEL_KEY_SECRET:-}" ]; then
    mkdir -p /app/secrets
    # Use printf to write key content; trailing newline added explicitly
    printf '%s\n' "$TUNNEL_KEY_SECRET" > "$TUNNEL_KEY"
    chmod 600 "$TUNNEL_KEY"
    unset TUNNEL_KEY_SECRET
    echo "SSH key written to $TUNNEL_KEY"
else
    echo "ERROR: No TUNNEL_KEY_B64 or TUNNEL_KEY_SECRET set"
    exec /app/blog
fi

# Debug: verify key is valid
key_lines=$(wc -l < "$TUNNEL_KEY")
echo "Key file has $key_lines lines"
head -1 "$TUNNEL_KEY"

# Create SSH directory for appuser
mkdir -p ~/.ssh 2>/dev/null || true

echo "Starting SSH tunnel: localhost:${LOCAL_PORT} -> ${TUNNEL_HOST}:${REMOTE_PORT}"

# --- Preflight: test the port-forward directly (no remote command exec).
# This is compatible with keys restricted by `command="/usr/sbin/nologin"` +
# `permitopen="localhost:${REMOTE_PORT}"` (i.e. port-forward-only keys),
# which is the hardened posture we want. We open a short-lived forward on
# a scratch local port and probe SurrealDB /health through it.
PREFLIGHT_LOCAL=9001
PREFLIGHT_LOG=/tmp/preflight-ssh.log
echo "=== SSH PREFLIGHT BEGIN ==="
ssh -v -N -n \
    -i "$TUNNEL_KEY" \
    -p "$TUNNEL_PORT" \
    -L "${PREFLIGHT_LOCAL}:localhost:${REMOTE_PORT}" \
    -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null \
    -o BatchMode=yes \
    -o ConnectTimeout=10 \
    -o ExitOnForwardFailure=yes \
    "${TUNNEL_USER}@${TUNNEL_HOST}" > "$PREFLIGHT_LOG" 2>&1 &
preflight_pid=$!

preflight_ok=0
for i in 1 2 3 4 5 6 7 8 9 10; do
    sleep 1
    if curl -sf --max-time 2 "http://localhost:${PREFLIGHT_LOCAL}/health" > /dev/null 2>&1; then
        preflight_ok=1
        echo "[preflight] port-forward to SurrealDB verified after ${i}s"
        break
    fi
    if ! kill -0 "$preflight_pid" 2>/dev/null; then
        echo "[preflight] ssh process exited early after ${i}s"
        break
    fi
done

# Tear down preflight ssh; the real tunnel runs via autossh below.
kill "$preflight_pid" 2>/dev/null || true
wait "$preflight_pid" 2>/dev/null || true

sed 's/^/[preflight] /' "$PREFLIGHT_LOG" 2>/dev/null | tail -20
echo "=== SSH PREFLIGHT END (ok=${preflight_ok}) ==="

if [ "$preflight_ok" -ne 1 ]; then
    echo "ERROR: SSH preflight port-forward failed. Starting app without tunnel (expect HTTP 504 until next deploy)."
    exec /app/blog
fi

# Preflight succeeded -> real tunnel via autossh.
export AUTOSSH_GATETIME=0
autossh -f -N \
    -v \
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
