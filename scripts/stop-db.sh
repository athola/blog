#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

echo "Stopping database server..."
pkill -f "surreal" 2>/dev/null || true
sleep 1
pkill -f "surreal" 2>/dev/null || true
echo "Database server stopped"
