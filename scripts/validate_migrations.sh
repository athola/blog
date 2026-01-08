#!/usr/bin/env bash
set -euo pipefail

# Validate SurrealDB migration syntax
# This script checks for the SurrealDB CLI and validates migration files

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "[validate-migrations]: Checking migration syntax..."

# Find SurrealDB CLI - prefer ~/.surrealdb (local project installation)
SURREAL_CMD=""
LOCAL_SURREAL="$HOME/.surrealdb/surreal"

if [ -f "$LOCAL_SURREAL" ]; then
    SURREAL_CMD="$LOCAL_SURREAL"
elif command -v surreal >/dev/null 2>&1; then
    SURREAL_CMD="surreal"
else
    echo -e "${YELLOW}Warning: SurrealDB CLI not found${NC}"
    echo "Install it with: make install-surrealdb"
    echo "Skipping migration validation..."
    exit 0
fi

# Check version and provide guidance if mismatched
VERSION=$($SURREAL_CMD --version 2>&1 | grep -oP '\d+\.\d+\.\d+(\.[a-z0-9.]+)?' || echo "unknown")
echo "[validate-migrations]: Using SurrealDB $VERSION at: $SURREAL_CMD"

# Check if there are any migration files to validate
if ! compgen -G "migrations/*.surql" > /dev/null; then
    echo "[validate-migrations]: No migration files found, skipping validation"
    exit 0
fi

# Validate migration syntax
if $SURREAL_CMD validate "migrations/*.surql" 2>&1; then
    echo -e "${GREEN}[validate-migrations]: All migration files have valid syntax${NC}"
    exit 0
else
    echo -e "${RED}[validate-migrations]: Migration syntax validation failed${NC}"
    echo "Run 'make install-surrealdb' to install the SurrealDB CLI (v2.4.0)"
    exit 1
fi
