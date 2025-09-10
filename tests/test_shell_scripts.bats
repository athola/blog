#!/usr/bin/env bats
#
# TDD Tests for Shell Scripts using BATS
# Following Red-Green-Refactor cycle and FIRST principles for bash scripts
#

# Setup function - runs before each test
setup() {
    # Create temporary directory for test isolation
    export TEST_TEMP_DIR="$(mktemp -d)"
    export PROJECT_ROOT="$(pwd)"
    
    # Create mock git repository structure
    cd "$TEST_TEMP_DIR"
    git init --quiet
    git config user.email "test@example.com"
    git config user.name "Test User"
    
    # Create some test files
    mkdir -p src tests
    echo "print('hello')" > src/main.py
    echo "def test_main(): pass" > tests/test_main.py
    echo "# Test Repo" > README.md
    
    # Create initial commit
    git add .
    git commit --quiet -m "Initial commit"
    
    # Create some changes for second commit
    echo "print('hello world')" > src/main.py
    echo "console.log('test');" > src/app.js
    git add .
    git commit --quiet -m "Add changes"
    
    # Get commit hashes for testing
    export BASE_SHA="$(git log --format="%H" -n 2 | tail -1)"
    export HEAD_SHA="$(git log --format="%H" -n 1)"
    
    cd "$PROJECT_ROOT"
}

# Teardown function - runs after each test
teardown() {
    rm -rf "$TEST_TEMP_DIR"
}

# RED: Test that load_pr_config.sh script exists and is executable
@test "load_pr_config.sh script exists and is executable" {
    [ -f "scripts/load_pr_config.sh" ]
    [ -x "scripts/load_pr_config.sh" ]
}

# RED: Test that calculate_pr_size.sh script exists and is executable
@test "calculate_pr_size.sh script exists and is executable" {
    [ -f "scripts/calculate_pr_size.sh" ]
    [ -x "scripts/calculate_pr_size.sh" ]
}

# RED: Test that categorize_pr_size.sh script exists and is executable
@test "categorize_pr_size.sh script exists and is executable" {
    [ -f "scripts/categorize_pr_size.sh" ]
    [ -x "scripts/categorize_pr_size.sh" ]
}

# RED: Test load_pr_config.sh with missing config file
@test "load_pr_config.sh handles missing config file gracefully" {
    # Temporarily move config file
    mv ".github/config/pr-size-limits.yml" ".github/config/pr-size-limits.yml.bak" 2>/dev/null || true
    
    run source scripts/load_pr_config.sh
    
    # Should succeed with default values
    [ "$status" -eq 0 ]
    
    # Should set default values
    [[ "$output" == *"using defaults"* ]]
    
    # Restore config file
    mv ".github/config/pr-size-limits.yml.bak" ".github/config/pr-size-limits.yml" 2>/dev/null || true
}

# RED: Test load_pr_config.sh with valid config file
@test "load_pr_config.sh loads configuration correctly" {
    run bash -c "source scripts/load_pr_config.sh && echo \$MAX_PR_SIZE"
    
    [ "$status" -eq 0 ]
    [ "$output" = "2000" ]
}

# RED: Test calculate_pr_size.sh with insufficient arguments
@test "calculate_pr_size.sh requires base and head SHA arguments" {
    run scripts/calculate_pr_size.sh
    
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

# RED: Test calculate_pr_size.sh with valid git repository
@test "calculate_pr_size.sh calculates PR size correctly" {
    cd "$TEST_TEMP_DIR"
    
    run "$PROJECT_ROOT/scripts/calculate_pr_size.sh" "$BASE_SHA" "$HEAD_SHA" "env"
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"TOTAL_LINES="* ]]
    [[ "$output" == *"ADDITIONS="* ]]
    [[ "$output" == *"DELETIONS="* ]]
}

# RED: Test calculate_pr_size.sh JSON output format
@test "calculate_pr_size.sh produces valid JSON output" {
    cd "$TEST_TEMP_DIR"
    
    run "$PROJECT_ROOT/scripts/calculate_pr_size.sh" "$BASE_SHA" "$HEAD_SHA" "json"
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"\"total_lines\":"* ]]
    [[ "$output" == *"\"additions\":"* ]]
    [[ "$output" == *"\"deletions\":"* ]]
    [[ "$output" == *"\"changed_files\":"* ]]
}

# RED: Test calculate_pr_size.sh with invalid output format
@test "calculate_pr_size.sh rejects invalid output format" {
    cd "$TEST_TEMP_DIR"
    
    run "$PROJECT_ROOT/scripts/calculate_pr_size.sh" "$BASE_SHA" "$HEAD_SHA" "invalid"
    
    [ "$status" -eq 1 ]
    [[ "$output" == *"output_format must be 'env' or 'json'"* ]]
}

# RED: Test categorize_pr_size.sh with insufficient arguments
@test "categorize_pr_size.sh requires total_lines argument" {
    run scripts/categorize_pr_size.sh
    
    [ "$status" -eq 1 ]
    [[ "$output" == *"Usage:"* ]]
}

# RED: Test categorize_pr_size.sh with ideal size
@test "categorize_pr_size.sh categorizes ideal size correctly" {
    run scripts/categorize_pr_size.sh 450 env
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=ideal"* ]]
    [[ "$output" == *"STATUS=success"* ]]
}

# RED: Test categorize_pr_size.sh with good size
@test "categorize_pr_size.sh categorizes good size correctly" {
    run scripts/categorize_pr_size.sh 800 env
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=good"* ]]
    [[ "$output" == *"STATUS=success"* ]]
}

# RED: Test categorize_pr_size.sh with large size
@test "categorize_pr_size.sh categorizes large size correctly" {
    run scripts/categorize_pr_size.sh 1800 env
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=large"* ]]
    [[ "$output" == *"STATUS=neutral"* ]]
}

# RED: Test categorize_pr_size.sh with too large size
@test "categorize_pr_size.sh categorizes too-large size correctly" {
    run scripts/categorize_pr_size.sh 2500 env
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=too-large"* ]]
    [[ "$output" == *"STATUS=failure"* ]]
}

# RED: Test categorize_pr_size.sh boundary values
@test "categorize_pr_size.sh handles boundary values correctly" {
    # Test exact boundary at 500
    run scripts/categorize_pr_size.sh 500 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=ideal"* ]]
    
    # Test just over boundary at 501
    run scripts/categorize_pr_size.sh 501 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=good"* ]]
    
    # Test exact boundary at 1500
    run scripts/categorize_pr_size.sh 1500 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=good"* ]]
    
    # Test just over boundary at 1501
    run scripts/categorize_pr_size.sh 1501 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=large"* ]]
    
    # Test exact boundary at 2000
    run scripts/categorize_pr_size.sh 2000 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=large"* ]]
    
    # Test just over boundary at 2001
    run scripts/categorize_pr_size.sh 2001 env
    [ "$status" -eq 0 ]
    [[ "$output" == *"CATEGORY=too-large"* ]]
}

# RED: Test categorize_pr_size.sh JSON output format
@test "categorize_pr_size.sh produces valid JSON output" {
    run scripts/categorize_pr_size.sh 800 json
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"\"category\": \"good\""* ]]
    [[ "$output" == *"\"status\": \"success\""* ]]
    [[ "$output" == *"\"total_lines\": 800"* ]]
}

# RED: Test categorize_pr_size.sh with invalid input
@test "categorize_pr_size.sh rejects non-numeric input" {
    run scripts/categorize_pr_size.sh "not_a_number" env
    
    [ "$status" -eq 1 ]
    [[ "$output" == *"must be a positive integer"* ]]
}

# RED: Test categorize_pr_size.sh with negative input
@test "categorize_pr_size.sh rejects negative input" {
    run scripts/categorize_pr_size.sh "-100" env
    
    [ "$status" -eq 1 ]
    [[ "$output" == *"must be a positive integer"* ]]
}

# RED: Test integration of calculate and categorize scripts
@test "calculate and categorize scripts work together" {
    cd "$TEST_TEMP_DIR"
    
    # Get PR size
    PR_SIZE_OUTPUT="$("$PROJECT_ROOT/scripts/calculate_pr_size.sh" "$BASE_SHA" "$HEAD_SHA" "json")"
    TOTAL_LINES="$(echo "$PR_SIZE_OUTPUT" | grep -o '"total_lines": [0-9]*' | grep -o '[0-9]*')"
    
    # Categorize the size
    run "$PROJECT_ROOT/scripts/categorize_pr_size.sh" "$TOTAL_LINES" "json"
    
    [ "$status" -eq 0 ]
    [[ "$output" == *"\"category\":"* ]]
    [[ "$output" == *"\"total_lines\": $TOTAL_LINES"* ]]
}

# RED: Test that scripts handle file exclusions correctly
@test "calculate_pr_size.sh excludes auto-generated files" {
    cd "$TEST_TEMP_DIR"
    
    # Create auto-generated files that should be excluded
    mkdir -p target/debug pkg dist
    echo "compiled_code" > target/debug/main.o
    echo "lock_content" > Cargo.lock
    echo "minified_js" > app.min.js
    echo "generated_file" > src/generated.rs
    
    git add .
    git commit --quiet -m "Add auto-generated files"
    
    NEW_HEAD_SHA="$(git log --format="%H" -n 1)"
    
    run "$PROJECT_ROOT/scripts/calculate_pr_size.sh" "$HEAD_SHA" "$NEW_HEAD_SHA" "env"
    
    [ "$status" -eq 0 ]
    # Output should show skipped files
    [[ "$output" == *"skipped (auto-generated file)"* ]]
}

# RED: Test error handling with invalid git references
@test "calculate_pr_size.sh handles invalid git references" {
    cd "$TEST_TEMP_DIR"
    
    run "$PROJECT_ROOT/scripts/calculate_pr_size.sh" "invalid_sha" "another_invalid_sha" "env"
    
    # Should fail gracefully
    [ "$status" -ne 0 ]
}

# RED: Test scripts work without GITHUB_OUTPUT environment variable
@test "scripts work in non-GitHub Actions environment" {
    unset GITHUB_OUTPUT
    
    run scripts/categorize_pr_size.sh 500 env
    
    [ "$status" -eq 0 ]
    # Should fall back to direct output
    [[ "$output" == *"CATEGORY=ideal"* ]]
}

# RED: Test configuration loading with malformed YAML
@test "load_pr_config.sh handles malformed configuration gracefully" {
    # Create temporary malformed config
    mkdir -p "$TEST_TEMP_DIR/.github/config"
    echo "invalid: yaml: content: [" > "$TEST_TEMP_DIR/.github/config/pr-size-limits.yml"
    
    cd "$TEST_TEMP_DIR"
    
    run source "$PROJECT_ROOT/scripts/load_pr_config.sh"
    
    # Should fall back to defaults
    [ "$status" -eq 0 ]
    [[ "$output" == *"using defaults"* ]]
}