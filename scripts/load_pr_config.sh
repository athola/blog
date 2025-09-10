#!/bin/bash

# Helper script to load PR size configuration from YAML
# Usage: source scripts/load_pr_config.sh

set -euo pipefail

# Path to the configuration file
CONFIG_FILE=".github/config/pr-size-limits.yml"

# Function to extract values from YAML using basic parsing
# (Avoids dependency on yq/jq for simple cases)
get_yaml_value() {
    local key="$1"
    local file="$2"
    
    # Use awk to extract the value for a given key, handling comments
    awk -F': ' -v key="$key" '
    $1 ~ key { 
        gsub(/^[ \t]+/, "", $2)  # Remove leading whitespace
        gsub(/[ \t]*#.*$/, "", $2)  # Remove comments
        gsub(/[ \t]+$/, "", $2)  # Remove trailing whitespace
        print $2
        exit
    }' "$file"
}

# Load configuration values
if [[ -f "$CONFIG_FILE" ]]; then
    echo "Loading PR size configuration from $CONFIG_FILE"
    
    # Extract limit values
    export MAX_PR_SIZE=$(get_yaml_value "max_pr_size" "$CONFIG_FILE")
    export IDEAL_PR_SIZE=$(get_yaml_value "ideal_pr_size" "$CONFIG_FILE")
    export GOOD_PR_SIZE=$(get_yaml_value "good_pr_size" "$CONFIG_FILE")
    
    echo "Loaded configuration:"
    echo "  MAX_PR_SIZE: $MAX_PR_SIZE"
    echo "  IDEAL_PR_SIZE: $IDEAL_PR_SIZE" 
    echo "  GOOD_PR_SIZE: $GOOD_PR_SIZE"
else
    echo "Warning: Configuration file $CONFIG_FILE not found, using defaults"
    export MAX_PR_SIZE=2000
    export IDEAL_PR_SIZE=500
    export GOOD_PR_SIZE=1500
fi