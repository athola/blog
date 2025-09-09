#!/bin/bash

# Secret scanning script for the Rust blog project
# Runs gitleaks, semgrep, and trufflehog to identify potential security issues

echo "=================================="
echo "Running Secret Scanning Tools"
echo "=================================="

# Create results directory
mkdir -p secret_scanning_results

echo ""
echo "1. Running Gitleaks..."
echo "====================="
gitleaks detect --source . --no-git --report-format json --report-path secret_scanning_results/gitleaks-report.json
if [ $? -eq 0 ] || [ $? -eq 1 ]; then
    echo "✅ Gitleaks scan completed successfully"
    if [ -s secret_scanning_results/gitleaks-report.json ]; then
        echo "⚠️  Gitleaks found potential secrets! Check secret_scanning_results/gitleaks-report.json"
    else
        echo "✅ No secrets found by Gitleaks"
    fi
else
    echo "❌ Gitleaks scan failed"
fi

echo ""
echo "2. Running Semgrep..."
echo "===================="
# Use our custom secrets rules from semgrep
semgrep --config=.semgrep.yml --json --output=secret_scanning_results/semgrep-report.json --exclude=secret_scanning_results/ --exclude=target/ --exclude=.git/ . 2>/dev/null
if [ $? -eq 0 ]; then
    echo "✅ Semgrep scan completed successfully"
    if [ -s secret_scanning_results/semgrep-report.json ]; then
        echo "⚠️  Semgrep found potential issues! Check secret_scanning_results/semgrep-report.json"
    else
        echo "✅ No issues found by Semgrep"
    fi
else
    echo "⚠️  Semgrep scan completed with warnings or no secrets found"
fi

echo ""
echo "3. Running Trufflehog..."
echo "======================="
# Run trufflehog with the correct syntax
trufflehog filesystem . --exclude-paths=secret_scanning_results/,target/,.git/ > secret_scanning_results/trufflehog-report.json
if [ $? -eq 0 ]; then
    echo "✅ Trufflehog scan completed successfully"
    if [ -s secret_scanning_results/trufflehog-report.json ]; then
        echo "⚠️  Trufflehog found potential secrets! Check secret_scanning_results/trufflehog-report.json"
    else
        echo "✅ No secrets found by Trufflehog"
    fi
else
    echo "⚠️  Trufflehog scan completed with warnings or no secrets found"
fi

echo ""
echo "=================================="
echo "Secret Scanning Complete"
echo "Reports saved in secret_scanning_results/"
echo "=================================="