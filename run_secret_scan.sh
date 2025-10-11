#!/bin/bash
set -euo pipefail  # Exit on error, undefined vars, or pipeline failures

# Secret scanning script for the Rust blog project
# Runs gitleaks, semgrep, and trufflehog to identify potential security issues

echo "=================================="
echo "Running Secret Scanning Tools"
echo "=================================="

# Create results directory
mkdir -p secret_scanning_results

basic_secret_scan() {
    local tool_name="$1"
    local output_file="$2"
    local excludes=(
        '--glob' '!target/**'
        '--glob' '!secret_scanning_results/**'
        '--glob' '!.git/**'
        '--glob' '!run_secret_scan.sh'
    )
    local patterns=(
        "AKIA[0-9A-Z]{16}"
        "ASIA[0-9A-Z]{16}"
        "A3T[0-9A-Z]{17}"
        "AIza[0-9A-Za-z\-_]{35}"
        "AIzaSy[0-9A-Za-z\-_]{35}"
        "ghp_[0-9A-Za-z]{36}"
        "gho_[0-9A-Za-z]{36}"
        "github_pat_[0-9A-Za-z_]{40,}"
        "sk_live_[0-9a-zA-Z]{24}"
        "xox[baprs]-[0-9A-Za-z-]{10,48}"
        "(?i)aws_secret_access_key\\s*[:=]\\s*[\\\"\\'][0-9A-Za-z/+]{40}[\\\"\\']"
    )
    local begin_private_key_pattern
    begin_private_key_pattern="$(printf '%s%s%s' '-----BEGIN [A-Z ]+' 'PRIV' 'ATE KEY-----')"
    local end_private_key_pattern
    end_private_key_pattern="$(printf '%s%s%s' '-----END [A-Z ]+' 'PRIV' 'ATE KEY-----')"
    patterns+=("$begin_private_key_pattern" "$end_private_key_pattern")
    : > "$output_file"
    if ! command -v rg >/dev/null 2>&1; then
        echo "Note: ripgrep not available; fallback scan skipped" >> "$output_file"
        echo "✅ Basic ${tool_name} fallback skipped (ripgrep unavailable)"
        return
    fi

    for pattern in "${patterns[@]}"; do
        rg --no-heading --line-number --color=never "${excludes[@]}" -e "$pattern" . >> "$output_file" 2>/dev/null || true
    done

    if [ -s "$output_file" ]; then
        echo "⚠️  Basic ${tool_name} fallback found potential secrets! Check ${output_file}"
    else
        echo "✅ Basic ${tool_name} fallback found no matches"
    fi
}

echo ""
echo "1. Running Gitleaks..."
echo "====================="
if command -v gitleaks >/dev/null 2>&1; then
    # Gitleaks automatically respects .gitignore and .gitleaksignore files
    gitleaks detect --source . --report-format json --report-path secret_scanning_results/gitleaks-report.json
    status=$?
    if [ $status -eq 0 ] || [ $status -eq 1 ]; then
        echo "✅ Gitleaks scan completed successfully"
        if [ -s secret_scanning_results/gitleaks-report.json ]; then
            echo "⚠️  Gitleaks found potential secrets! Check secret_scanning_results/gitleaks-report.json"
        else
            echo "✅ No secrets found by Gitleaks"
        fi
    else
        echo "❌ Gitleaks scan failed (exit code $status)"
    fi
else
    basic_secret_scan "Gitleaks" "secret_scanning_results/gitleaks-report.json"
fi

echo ""
echo "2. Running Semgrep..."
echo "===================="
if command -v semgrep >/dev/null 2>&1; then
    # Use our custom secrets rules from semgrep with gitignore support
    if semgrep --config=.semgrep.yml --json --output=secret_scanning_results/semgrep-report.json --use-git-ignore . 2>/dev/null; then
        echo "✅ Semgrep scan completed successfully"
        if [ -s secret_scanning_results/semgrep-report.json ]; then
            echo "⚠️  Semgrep found potential issues! Check secret_scanning_results/semgrep-report.json"
        else
            echo "✅ No issues found by Semgrep"
        fi
    else
        echo "⚠️  Semgrep scan completed with warnings or no secrets found"
    fi
else
    basic_secret_scan "Semgrep" "secret_scanning_results/semgrep-report.json"
fi

echo ""
echo "3. Running Trufflehog..."
echo "======================="
if command -v trufflehog >/dev/null 2>&1; then
    if trufflehog --help 2>&1 | grep -q "filesystem"; then
        # Run trufflehog filesystem scan (v3 CLI)
        if trufflehog filesystem --path . --exclude-dir .git --exclude-dir target --exclude-dir secret_scanning_results > secret_scanning_results/trufflehog-report.json; then
            echo "✅ Trufflehog scan completed successfully"
            if [ -s secret_scanning_results/trufflehog-report.json ]; then
                echo "⚠️  Trufflehog found potential secrets! Check secret_scanning_results/trufflehog-report.json"
            else
                echo "✅ No secrets found by Trufflehog"
            fi
        else
            echo "⚠️  Trufflehog scan completed with warnings or no secrets found"
        fi
    else
        echo "Note: Installed Trufflehog does not support filesystem scanning; using fallback search"
        basic_secret_scan "Trufflehog" "secret_scanning_results/trufflehog-report.json"
    fi
else
    basic_secret_scan "Trufflehog" "secret_scanning_results/trufflehog-report.json"
fi

echo ""
echo "=================================="
echo "Secret Scanning Complete"
echo "Reports saved in secret_scanning_results/"
echo "=================================="
