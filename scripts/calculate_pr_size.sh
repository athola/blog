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

# Adaptive timeout based on estimated PR size
set_adaptive_timeout() {
    local estimated_files=${1:-100}
    
    # Base timeout of 30 seconds
    local base_timeout=30
    
    # Add 1 second per 100 files estimated
    local additional_timeout=$((estimated_files / 100))
    
    # Cap at 5 minutes for very large PRs
    local max_timeout=300
    
    export GIT_TIMEOUT=$((base_timeout + additional_timeout))
    if [[ "$GIT_TIMEOUT" -gt "$max_timeout" ]]; then
        export GIT_TIMEOUT=$max_timeout
    fi
    
    echo "Set adaptive timeout to ${GIT_TIMEOUT}s for estimated $estimated_files files"
}

# Set initial timeout
export GIT_TIMEOUT=30

# Memory and performance limits
export MAX_FILES_TO_PROCESS=10000
export MAX_DIFF_SIZE_BYTES=$((50 * 1024 * 1024))  # 50MB limit
export STREAMING_THRESHOLD=1000  # Process files in batches above this count

# Define exclusion patterns as arrays for better performance
declare -a DIRECTORY_EXCLUSIONS=(
    "target/*"
    "*/pkg/*"
    "*/dist/*"
    "*/generated/*"
    "*/node_modules/*"
)

declare -a FILE_EXCLUSIONS=(
    "*.lock"
    "*.min.js"
    "*.min.css"
    "*_generated.*"
    "package-lock.json"
    "yarn.lock"
    "Cargo.lock"
)

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

# Function to check if file should be excluded
is_excluded_file() {
    local filename="$1"
    
    # Check directory exclusions
    for pattern in "${DIRECTORY_EXCLUSIONS[@]}"; do
        case "$filename" in
            $pattern)
                return 0
                ;;
        esac
    done
    
    # Check file exclusions
    for pattern in "${FILE_EXCLUSIONS[@]}"; do
        case "$filename" in
            $pattern)
                return 0
                ;;
        esac
    done
    
    return 1
}

# Validate git repository and SHAs
validate_git_environment() {
    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        echo "Error: Not in a git repository" >&2
        return 1
    fi
    
    # Validate base SHA exists and is a valid commit
    if ! git cat-file -e "$BASE_SHA" 2>/dev/null; then
        echo "Error: Base SHA '$BASE_SHA' is not a valid commit" >&2
        return 1
    fi
    
    # Validate head SHA exists and is a valid commit
    if ! git cat-file -e "$HEAD_SHA" 2>/dev/null; then
        echo "Error: Head SHA '$HEAD_SHA' is not a valid commit" >&2
        return 1
    fi
    
    # Check if SHAs are different (avoid comparing same commit)
    if [[ "$BASE_SHA" == "$HEAD_SHA" ]]; then
        echo "Warning: Base and head SHAs are identical, no changes to analyze" >&2
        return 2  # Special return code for no changes
    fi
    
    return 0
}

# Validate environment before proceeding
echo "Validating git environment..."
validate_result=$(validate_git_environment)
case $? in
    0) echo "✅ Git environment validated" ;;
    1) exit 1 ;;
    2) 
        # No changes case - return empty results
        echo "No changes detected between commits"
        if [[ "$OUTPUT_FORMAT" == "json" ]]; then
            echo '{"total_lines": 0, "additions": 0, "deletions": 0, "changed_files": 0}'
        else
            [[ -n "${GITHUB_OUTPUT:-}" ]] && {
                echo "total-lines=0" >> "$GITHUB_OUTPUT"
                echo "additions=0" >> "$GITHUB_OUTPUT" 
                echo "deletions=0" >> "$GITHUB_OUTPUT"
                echo "changed-files=0" >> "$GITHUB_OUTPUT"
            } || {
                echo "TOTAL_LINES=0"
                echo "ADDITIONS=0"
                echo "DELETIONS=0" 
                echo "CHANGED_FILES=0"
            }
        fi
        exit 0
        ;;
esac

# Get diff stats with timeout and comprehensive error handling
echo "Getting diff statistics (timeout: ${GIT_TIMEOUT}s)..."
echo "Comparing $HEAD_SHA with base $BASE_SHA"

# Check if diff will be too large before processing
check_diff_size() {
    echo "Checking diff size to prevent memory issues..." >&2
    
    # Get basic diff stats first (lightweight)
    local stats_output
    if ! stats_output=$(timeout 10 git diff --shortstat "$BASE_SHA"..."$HEAD_SHA" 2>/dev/null); then
        echo "Warning: Could not get diff statistics, proceeding with caution" >&2
        return 0
    fi
    
    # Parse the shortstat output to get file count
    local files_changed
    files_changed=$(echo "$stats_output" | awk '{print $1}' | grep -o '[0-9]*' || echo "0")
    
    # Set adaptive timeout based on file count
    set_adaptive_timeout "$files_changed"
    
    if [[ "$files_changed" -gt "$MAX_FILES_TO_PROCESS" ]]; then
        echo "Error: Too many files changed ($files_changed > $MAX_FILES_TO_PROCESS)" >&2
        echo "This PR is too large to process safely. Consider splitting it." >&2
        return 1
    fi
    
    echo "✅ Diff size check passed: $files_changed files changed" >&2
    return 0
}

# Try git diff with timeout and multiple fallback strategies
get_diff_with_fallback() {
    local diff_output
    
    # Check if the diff will be too large first
    if ! check_diff_size; then
        return 1
    fi
    
    # Primary attempt: Use timeout with diff-filter for added/modified files
    echo "Attempting primary diff with timeout and filter..." >&2
    if diff_output=$(timeout "$GIT_TIMEOUT" git diff --numstat --diff-filter=AM "$BASE_SHA"..."$HEAD_SHA" 2>/dev/null); then
        # Check if output is reasonable size
        local output_size=${#diff_output}
        if [[ "$output_size" -gt "$MAX_DIFF_SIZE_BYTES" ]]; then
            echo "Warning: Diff output is very large (${output_size} bytes), truncating..." >&2
            diff_output=$(echo "$diff_output" | head -n "$MAX_FILES_TO_PROCESS")
        fi
        echo "$diff_output"
        return 0
    fi
    
    # Fallback 1: Try without timeout (in case timeout command is not available)
    echo "Primary diff failed, trying without timeout..." >&2
    if diff_output=$(git diff --numstat --diff-filter=AM "$BASE_SHA"..."$HEAD_SHA" 2>/dev/null); then
        echo "$diff_output"
        return 0
    fi
    
    # Fallback 2: Try without diff-filter but with size limits
    echo "Trying diff without filter but with size limits..." >&2
    if diff_output=$(timeout "$GIT_TIMEOUT" git diff --numstat "$BASE_SHA"..."$HEAD_SHA" 2>/dev/null | head -n "$MAX_FILES_TO_PROCESS"); then
        if [[ -n "$diff_output" ]]; then
            echo "$diff_output"
            return 0
        fi
    fi
    
    # Fallback 3: Try different commit syntax with limits
    echo "Trying alternative commit syntax with limits..." >&2
    if diff_output=$(timeout "$GIT_TIMEOUT" git diff --numstat "$BASE_SHA" "$HEAD_SHA" 2>/dev/null | head -n "$MAX_FILES_TO_PROCESS"); then
        if [[ -n "$diff_output" ]]; then
            echo "$diff_output"
            return 0
        fi
    fi
    
    return 1
}

if ! DIFF_OUTPUT=$(get_diff_with_fallback); then
    echo "Error: All git diff strategies failed. Possible causes:" >&2
    echo "  - Network issues or repository corruption" >&2
    echo "  - Invalid commit SHAs" >&2
    echo "  - Insufficient permissions" >&2
    echo "  - Merge conflicts in the range" >&2
    exit 1
fi

# Calculate total lines changed (additions + deletions)
ADDITIONS=0
DELETIONS=0
CHANGED_FILES=0

# Enhanced binary file detection
is_binary_file() {
    local filename="$1"
    local added="$2"
    local deleted="$3"
    
    # Primary check: git diff marks binary files with "-"
    if [[ "$added" == "-" || "$deleted" == "-" ]]; then
        return 0
    fi
    
    # Secondary check: file extension patterns for known binary types
    case "${filename,,}" in
        *.jpg|*.jpeg|*.png|*.gif|*.bmp|*.ico|*.svg)
            return 0 ;;
        *.pdf|*.doc|*.docx|*.xls|*.xlsx|*.ppt|*.pptx)
            return 0 ;;
        *.zip|*.tar|*.gz|*.bz2|*.7z|*.rar)
            return 0 ;;
        *.exe|*.dll|*.so|*.dylib|*.bin)
            return 0 ;;
        *.mp3|*.mp4|*.avi|*.mov|*.wav|*.flac)
            return 0 ;;
        *.wasm|*.woff|*.woff2|*.ttf|*.otf)
            return 0 ;;
        *.jar|*.war|*.ear|*.class)
            return 0 ;;
        *)
            return 1 ;;
    esac
}

# Streaming processor for large diffs to reduce memory usage
process_diff_streaming() {
    local file_count=0
    local binary_count=0
    local excluded_count=0
    local processed_count=0
    local batch_size=100
    local current_batch=0
    
    echo "Processing diff with streaming approach for memory efficiency..."
    
    # Process in batches to avoid memory issues
    while IFS=$'\t' read -r added deleted filename; do
        # Skip empty lines
        [[ -n "$filename" ]] || continue
        ((file_count++))
        ((current_batch++))
        
        # Progress indicator for large diffs
        if [[ $((current_batch % batch_size)) -eq 0 ]]; then
            echo "  Progress: processed $current_batch files (binary: $binary_count, excluded: $excluded_count, text: $processed_count)"
        fi
        
        # Enhanced binary file detection
        if is_binary_file "$filename" "$added" "$deleted"; then
            [[ $((binary_count % 50)) -eq 0 ]] && echo "  $filename: binary file (skipped)"
            ((binary_count++))
            continue
        fi
        
        # Check if file should be excluded using efficient function
        if is_excluded_file "$filename"; then
            [[ $((excluded_count % 50)) -eq 0 ]] && echo "  $filename: excluded (auto-generated)"
            ((excluded_count++))
            continue
        fi
        
        # Validate numeric values with enhanced error handling
        if [[ "$added" =~ ^[0-9]+$ ]] && [[ "$deleted" =~ ^[0-9]+$ ]]; then
            # Additional safety check for extremely large values
            if [[ "$added" -gt 1000000 ]] || [[ "$deleted" -gt 1000000 ]]; then
                echo "  $filename: suspicious file size (+$added -$deleted), treating as binary"
                ((binary_count++))
                continue
            fi
            
            ADDITIONS=$((ADDITIONS + added))
            DELETIONS=$((DELETIONS + deleted))
            CHANGED_FILES=$((CHANGED_FILES + 1))
            ((processed_count++))
            
            # Only show details for first few files to reduce output volume
            if [[ "$processed_count" -le 20 ]]; then
                echo "  $filename: +$added -$deleted"
            fi
        else
            echo "  $filename: invalid diff stats (added='$added', deleted='$deleted'), treating as binary"
            ((binary_count++))
        fi
        
        # Memory check - if we're processing too much, suggest splitting
        if [[ "$current_batch" -gt "$STREAMING_THRESHOLD" ]]; then
            echo "⚠️  Processing large number of files ($current_batch), this may take time..."
        fi
    done
    
    echo ""
    echo "✅ File processing summary:"
    echo "  Total files in diff: $file_count"
    echo "  Binary files skipped: $binary_count"
    echo "  Auto-generated files excluded: $excluded_count"
    echo "  Text files processed: $processed_count"
}

# Process diff output with memory-efficient streaming
if [[ -n "$DIFF_OUTPUT" ]]; then
    process_diff_streaming <<< "$DIFF_OUTPUT"
else
    echo "No changes detected in diff"
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