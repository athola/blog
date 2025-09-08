#!/bin/bash

# Script to run integration tests sequentially to avoid resource conflicts

echo "Running server integration tests sequentially..."

# List of test functions to run
TESTS=(
    "test_server_startup_and_connectivity"
    "test_page_navigation"
    "test_navigation_elements"
    "test_page_specific_content"
    "test_static_asset_serving"
    "test_server_performance"
    "test_error_handling"
    "test_complete_development_workflow"
)

# Counter for passed/failed tests
PASSED=0
FAILED=0

# Run each test individually
for TEST in "${TESTS[@]}"; do
    echo "----------------------------------------"
    echo "Running test: $TEST"
    echo "----------------------------------------"
    
    # Run the test with a timeout
    timeout 120 cargo test --workspace --test server_integration_tests "$TEST" -- --nocapture
    
    # Check the exit code
    if [ $? -eq 0 ]; then
        echo "✅ PASSED: $TEST"
        ((PASSED++))
    else
        echo "❌ FAILED: $TEST"
        ((FAILED++))
    fi
    
    # Wait a bit between tests to ensure cleanup
    echo "Waiting for cleanup..."
    sleep 5
    
    # Kill any remaining processes
    pkill -f "cargo leptos" 2>/dev/null || true
    pkill -f "make.*watch" 2>/dev/null || true
    pkill -f "tailwindcss" 2>/dev/null || true
    pkill -f "wasm-bindgen" 2>/dev/null || true
    
    # Wait a bit more for processes to terminate
    sleep 2
done

# Print summary
echo "========================================"
echo "TEST SUMMARY"
echo "========================================"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Total:  $((${PASSED} + ${FAILED}))"

if [ $FAILED -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed."
    exit 1
fi