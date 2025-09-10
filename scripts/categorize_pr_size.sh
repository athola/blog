#!/bin/bash

# Shared PR size categorization script
# Provides consistent categorization across workflows
#
# Usage: ./scripts/categorize_pr_size.sh <total_lines> [output_format]
#
# Arguments:
#   total_lines: Total lines changed in PR
#   output_format: "env" (default) or "json"
#
# Outputs:
#   env format: Sets GitHub Actions output variables
#   json format: Prints JSON object with category info

set -euo pipefail

# Validate arguments
if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <total_lines> [output_format]" >&2
    exit 1
fi

TOTAL_LINES="$1"
OUTPUT_FORMAT="${2:-env}"

# Validate total_lines is a number
if ! [[ "$TOTAL_LINES" =~ ^[0-9]+$ ]]; then
    echo "Error: total_lines must be a positive integer" >&2
    exit 1
fi

# Validate output format
if [[ "$OUTPUT_FORMAT" != "env" && "$OUTPUT_FORMAT" != "json" ]]; then
    echo "Error: output_format must be 'env' or 'json'" >&2
    exit 1
fi

# Load configuration
source "$(dirname "$0")/load_pr_config.sh"

# Categorize based on thresholds
if [ "$TOTAL_LINES" -le "$IDEAL_PR_SIZE" ]; then
    CATEGORY="ideal"
    STATUS="success"
    MESSAGE="✅ Ideal PR size ($TOTAL_LINES lines)"
    EMOJI="✅"
    LABEL="✅ Ideal"
elif [ "$TOTAL_LINES" -le "$GOOD_PR_SIZE" ]; then
    CATEGORY="good"
    STATUS="success"
    MESSAGE="🟡 Good PR size ($TOTAL_LINES lines)"
    EMOJI="🟡"
    LABEL="🟡 Good"
elif [ "$TOTAL_LINES" -le "$MAX_PR_SIZE" ]; then
    CATEGORY="large"
    STATUS="neutral"
    MESSAGE="⚠️ Large PR size ($TOTAL_LINES lines) - consider splitting"
    EMOJI="⚠️"
    LABEL="⚠️ Large"
else
    CATEGORY="too-large"
    STATUS="failure"
    MESSAGE="❌ PR too large ($TOTAL_LINES lines) - must be split"
    EMOJI="❌"
    LABEL="❌ Too Large"
fi

echo "PR Size: $TOTAL_LINES lines -> Category: $CATEGORY"

# Output results based on format
if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    # JSON output
    cat << EOF
{
  "category": "$CATEGORY",
  "status": "$STATUS", 
  "message": "$MESSAGE",
  "emoji": "$EMOJI",
  "label": "$LABEL",
  "total_lines": $TOTAL_LINES
}
EOF
else
    # GitHub Actions environment output
    if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
        echo "category=$CATEGORY" >> "$GITHUB_OUTPUT"
        echo "status=$STATUS" >> "$GITHUB_OUTPUT"
        echo "message=$MESSAGE" >> "$GITHUB_OUTPUT"
        echo "emoji=$EMOJI" >> "$GITHUB_OUTPUT"
        echo "label=$LABEL" >> "$GITHUB_OUTPUT"
    else
        # Fallback for local testing
        echo "CATEGORY=$CATEGORY"
        echo "STATUS=$STATUS"
        echo "MESSAGE=$MESSAGE"
        echo "EMOJI=$EMOJI"
        echo "LABEL=$LABEL"
    fi
fi