#!/usr/bin/env bash
set -euo pipefail

# Validate CI workflow environment variables
# Ensures production container tests include all required env vars
# This catches missing env vars before they break CI

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "[validate-ci-env]: Checking CI workflow production environment variables..."

WORKFLOW_FILE=".github/workflows/docker-build-test.yml"

if [ ! -f "$WORKFLOW_FILE" ]; then
    echo -e "${YELLOW}Warning: $WORKFLOW_FILE not found${NC}"
    echo "Skipping CI env validation..."
    exit 0
fi

# Required environment variables for production mode
# These must be present when RUST_ENV=production is set
REQUIRED_PRODUCTION_VARS=(
    "SURREAL_NS"
    "SURREAL_DB"
    "LEPTOS_SITE_ADDR"
    "SMTP_HOST"
    "SMTP_USER"
    "SMTP_PASSWORD"
)

# At least one set of database credentials must be present
DB_CRED_SETS=(
    "SURREAL_ROOT_USER:SURREAL_ROOT_PASS"
    "SURREAL_NAMESPACE_USER:SURREAL_NAMESPACE_PASS"
    "SURREAL_USERNAME:SURREAL_PASSWORD"
)

ERRORS=0

# Check if production mode is used in docker run commands
if grep -q "RUST_ENV=production" "$WORKFLOW_FILE"; then
    echo "[validate-ci-env]: Found RUST_ENV=production in workflow"

    # Extract the docker run block (simplified check)
    # Look for env vars being passed to docker run
    DOCKER_RUN_SECTION=$(grep -A 50 "docker run" "$WORKFLOW_FILE" | head -60)

    # Check each required variable
    for var in "${REQUIRED_PRODUCTION_VARS[@]}"; do
        # Check for -e VAR= or -e "$VAR" pattern
        if ! echo "$DOCKER_RUN_SECTION" | grep -qE "(${var}=|-e.*\\\$.*${var}|TEST_${var})"; then
            echo -e "${RED}Error: Missing required production env var: $var${NC}"
            ERRORS=$((ERRORS + 1))
        fi
    done

    # Check for at least one set of database credentials
    HAS_DB_CREDS=0
    for cred_pair in "${DB_CRED_SETS[@]}"; do
        USER_VAR="${cred_pair%%:*}"
        PASS_VAR="${cred_pair##*:}"

        if echo "$DOCKER_RUN_SECTION" | grep -qE "(${USER_VAR}|TEST_${USER_VAR})" && \
           echo "$DOCKER_RUN_SECTION" | grep -qE "(${PASS_VAR}|TEST_${PASS_VAR})"; then
            HAS_DB_CREDS=1
            break
        fi
    done

    if [ $HAS_DB_CREDS -eq 0 ]; then
        echo -e "${RED}Error: No database credentials found in docker run command${NC}"
        echo "       Need one of: ROOT_USER/PASS, NAMESPACE_USER/PASS, or USERNAME/PASSWORD"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "[validate-ci-env]: No RUST_ENV=production found in workflow (development mode)"
fi

if [ $ERRORS -gt 0 ]; then
    echo ""
    echo -e "${RED}[validate-ci-env]: Found $ERRORS error(s)${NC}"
    echo "Production container tests must include all required env vars."
    echo "See server/src/security.rs validate_production_env() for requirements."
    exit 1
fi

echo -e "${GREEN}[validate-ci-env]: All production environment variables present${NC}"
exit 0
