#!/bin/bash

# Helper script to load PR size configuration from YAML
# Usage: source scripts/load_pr_config.sh

set -euo pipefail

# Path to the configuration file
CONFIG_FILE=".github/config/pr-size-limits.yml"

# Function to validate numeric value
validate_numeric() {
    local value="$1"
    local name="$2"
    
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
        echo "Error: $name must be a positive integer, got: $value" >&2
        return 1
    fi
    
    if [[ "$value" -lt 1 ]]; then
        echo "Error: $name must be positive, got: $value" >&2
        return 1
    fi
    
    if [[ "$value" -gt 50000 ]]; then
        echo "Warning: $name is very large ($value), consider if this is intentional" >&2
    fi
    
    return 0
}

# Function to extract values from YAML using robust parsing
get_yaml_value() {
    local key="$1"
    local file="$2"
    
    if [[ ! -f "$file" ]]; then
        echo "Error: Configuration file $file not found" >&2
        return 1
    fi
    
    # More robust awk parsing to handle various YAML formats
    local value
    value=$(awk -v key="^[[:space:]]*${key}[[:space:]]*:" '
    $0 ~ key { 
        # Extract everything after the colon
        sub(/^[[:space:]]*[^:]+:[[:space:]]*/, "")
        # Remove inline comments
        sub(/[[:space:]]*#.*$/, "")
        # Remove quotes if present
        gsub(/^["\047]|["\047]$/, "")
        # Remove trailing whitespace
        gsub(/[[:space:]]+$/, "")
        if (length($0) > 0) {
            print $0
            exit
        }
    }' "$file")
    
    if [[ -z "$value" ]]; then
        echo "Error: Could not find key '$key' in $file" >&2
        return 1
    fi
    
    echo "$value"
}

# Load configuration values with validation
load_and_validate_config() {
    if [[ -f "$CONFIG_FILE" ]]; then
        echo "Loading PR size configuration from $CONFIG_FILE"
        
        # Extract limit values with error handling
        local max_size ideal_size good_size
        
        if ! max_size=$(get_yaml_value "max_pr_size" "$CONFIG_FILE"); then
            echo "Error: Failed to load max_pr_size" >&2
            return 1
        fi
        
        if ! ideal_size=$(get_yaml_value "ideal_pr_size" "$CONFIG_FILE"); then
            echo "Error: Failed to load ideal_pr_size" >&2
            return 1
        fi
        
        if ! good_size=$(get_yaml_value "good_pr_size" "$CONFIG_FILE"); then
            echo "Error: Failed to load good_pr_size" >&2
            return 1
        fi
        
        # Validate all values are numeric and reasonable
        if ! validate_numeric "$max_size" "max_pr_size" || \
           ! validate_numeric "$ideal_size" "ideal_pr_size" || \
           ! validate_numeric "$good_size" "good_pr_size"; then
            echo "Error: Configuration validation failed" >&2
            return 1
        fi
        
        # Validate logical ordering
        if [[ "$ideal_size" -gt "$good_size" ]]; then
            echo "Error: ideal_pr_size ($ideal_size) cannot be greater than good_pr_size ($good_size)" >&2
            return 1
        fi
        
        if [[ "$good_size" -gt "$max_size" ]]; then
            echo "Error: good_pr_size ($good_size) cannot be greater than max_pr_size ($max_size)" >&2
            return 1
        fi
        
        # Export validated values
        export MAX_PR_SIZE="$max_size"
        export IDEAL_PR_SIZE="$ideal_size"
        export GOOD_PR_SIZE="$good_size"
        
        echo "✅ Loaded and validated configuration:"
        echo "  IDEAL_PR_SIZE: $IDEAL_PR_SIZE (≤ ideal)"
        echo "  GOOD_PR_SIZE: $GOOD_PR_SIZE (≤ good)" 
        echo "  MAX_PR_SIZE: $MAX_PR_SIZE (≤ maximum allowed)"
        
    else
        echo "⚠️ Configuration file $CONFIG_FILE not found, using defaults"
        export MAX_PR_SIZE=2000
        export IDEAL_PR_SIZE=500
        export GOOD_PR_SIZE=1500
        
        echo "Using default configuration:"
        echo "  IDEAL_PR_SIZE: $IDEAL_PR_SIZE"
        echo "  GOOD_PR_SIZE: $GOOD_PR_SIZE" 
        echo "  MAX_PR_SIZE: $MAX_PR_SIZE"
    fi
}

# Security validation - ensure we're in a safe environment
validate_environment() {
    # Check if we're running in a CI environment
    if [[ -z "${CI:-}" && -z "${GITHUB_ACTIONS:-}" ]]; then
        echo "Warning: Not running in CI environment, ensure this is intentional" >&2
    fi
    
    # Validate we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        echo "Error: Not in a git repository" >&2
        return 1
    fi
    
    # Ensure no dangerous environment variables are set
    local dangerous_vars=("LD_PRELOAD" "LD_LIBRARY_PATH" "DYLD_INSERT_LIBRARIES")
    for var in "${dangerous_vars[@]}"; do
        if [[ -n "${!var:-}" ]]; then
            echo "Warning: Dangerous environment variable $var is set" >&2
        fi
    done
    
    return 0
}

# Perform environment validation
echo "🛡️  Validating environment security..."
if ! validate_environment; then
    echo "Error: Environment validation failed" >&2
    exit 1
fi

# Load configuration
if ! load_and_validate_config; then
    echo "Error: Failed to load PR size configuration, exiting" >&2
    exit 1
fi