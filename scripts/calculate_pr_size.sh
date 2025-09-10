#!/bin/bash

# Shared PR size calculation script
# Eliminates duplication between pr-size-check.yml and pr-size-metrics.yml
# 
# Usage: ./scripts/calculate_pr_size.sh <base_sha> <head_sha> [output_format]
#
# Arguments:
#   base_sha: Base branch SHA for comparison
#   head_sha: Head branch SHA for comparison  
#   output_format: "env" (default) or "json"
#
# Outputs:
#   env format: Sets GitHub Actions output variables
#   json format: Prints JSON object with metrics

set -euo pipefail

# Validate arguments
if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <base_sha> <head_sha> [output_format]" >&2
    exit 1
fi

BASE_SHA="$1"
HEAD_SHA="$2"
OUTPUT_FORMAT="${3:-env}"

# Validate output format
if [[ "$OUTPUT_FORMAT" != "env" && "$OUTPUT_FORMAT" != "json" ]]; then
    echo "Error: output_format must be 'env' or 'json'" >&2
    exit 1
fi

echo "Comparing $HEAD_SHA with base $BASE_SHA"

# Get diff stats (additions and deletions)
DIFF_OUTPUT=$(git diff --numstat "$BASE_SHA"..."$HEAD_SHA")

# Calculate total lines changed (additions + deletions)
ADDITIONS=0
DELETIONS=0
CHANGED_FILES=0

# Handle empty diff case
if [[ -n "$DIFF_OUTPUT" ]]; then
    while IFS=$'\t' read -r added deleted filename; do
        # Skip binary files (marked with -)
        if [[ "$added" != "-" && "$deleted" != "-" ]]; then
            # Skip auto-generated files
            # - Files in target/ directory (Rust build artifacts)
            # - Lock files
            # - Generated frontend assets
            if [[ "$filename" == target/* ]] || \
               [[ "$filename" == *.lock ]] || \
               [[ "$filename" == */pkg/* ]] || \
               [[ "$filename" == */dist/* ]] || \
               [[ "$filename" == *.min.js ]] || \
               [[ "$filename" == *.min.css ]] || \
               [[ "$filename" == */generated/* ]] || \
               [[ "$filename" == *_generated.* ]]; then
                echo "  $filename: skipped (auto-generated file)"
            else
                ADDITIONS=$((ADDITIONS + added))
                DELETIONS=$((DELETIONS + deleted))
                CHANGED_FILES=$((CHANGED_FILES + 1))
                echo "  $filename: +$added -$deleted"
            fi
        else
            echo "  $filename: binary file"
        fi
    done <<< "$DIFF_OUTPUT"
fi

TOTAL_LINES=$((ADDITIONS + DELETIONS))

echo "PR Statistics:"
echo "  Files changed: $CHANGED_FILES"
echo "  Lines added: $ADDITIONS"
echo "  Lines deleted: $DELETIONS"
echo "  Total lines changed: $TOTAL_LINES"

# Output results based on format
if [[ "$OUTPUT_FORMAT" == "json" ]]; then
    # JSON output
    cat << EOF
{
  "total_lines": $TOTAL_LINES,
  "additions": $ADDITIONS,
  "deletions": $DELETIONS,
  "changed_files": $CHANGED_FILES
}
EOF
else
    # GitHub Actions environment output
    if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
        echo "total-lines=$TOTAL_LINES" >> "$GITHUB_OUTPUT"
        echo "additions=$ADDITIONS" >> "$GITHUB_OUTPUT"
        echo "deletions=$DELETIONS" >> "$GITHUB_OUTPUT"
        echo "changed-files=$CHANGED_FILES" >> "$GITHUB_OUTPUT"
    else
        # Fallback for local testing
        echo "TOTAL_LINES=$TOTAL_LINES"
        echo "ADDITIONS=$ADDITIONS"
        echo "DELETIONS=$DELETIONS"
        echo "CHANGED_FILES=$CHANGED_FILES"
    fi
fi