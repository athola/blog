#!/usr/bin/env bash
# Token discipline check — fails on arbitrary color values in component code.
#
# Direction D (site-redesign-0.2.0) requires all colors to flow from
# Tailwind v4 @theme tokens (style/tailwind.css). This guard catches
# inline `bg-[#hex]`, `text-[#hex]`, etc. creeping into Rust component
# code where they should be `bg-paper`, `text-accent`, etc.
#
# As of T20 (Sprint 2 lint clear), the allow-list is empty: every Rust
# source file under app/src and frontend/src must reference @theme
# tokens, with zero arbitrary color values. New violations fail the
# guard immediately.
#
# Usage: scripts/lint_tokens.sh
# Exit 0 on success, 1 on violation.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Allow-list — keep empty after Sprint 2 T20. Add a temporary entry only
# for active in-flight refactors and remove it as the file is migrated.
ALLOWED_FILES=()

# Search pattern: bg-[#hex], text-[#hex], border-[#hex], from-[#hex],
# to-[#hex], via-[#hex], ring-[#hex], outline-[#hex], decoration-[#hex],
# accent-[#hex], shadow-[#hex], stroke-[#hex], fill-[#hex]
PATTERN='(bg|text|border|from|to|via|ring|outline|decoration|accent|shadow|stroke|fill)-\[#[0-9a-fA-F]{3,8}'

# Search Rust source under app/src/ and frontend/src/
ROOTS=("app/src" "frontend/src")
EXIT=0

for root in "${ROOTS[@]}"; do
    if [[ ! -d "$root" ]]; then
        continue
    fi
    while IFS= read -r match; do
        rel="${match#$REPO_ROOT/}"
        rel="${rel%%:*}"
        skip=0
        for allowed in "${ALLOWED_FILES[@]+"${ALLOWED_FILES[@]}"}"; do
            if [[ "$rel" == "$allowed" ]]; then
                skip=1
                break
            fi
        done
        if [[ $skip -eq 0 ]]; then
            echo "$match" >&2
            EXIT=1
        fi
    done < <(grep -rEHn "$PATTERN" "$root" --include='*.rs' 2>/dev/null || true)
done

if [[ $EXIT -ne 0 ]]; then
    echo "" >&2
    echo "✗ Found arbitrary color values in component code." >&2
    echo "  Replace bg-[#hex], text-[#hex], etc. with @theme tokens" >&2
    echo "  (e.g. bg-paper, text-accent, border-rule)." >&2
    echo "  Spec: docs/specification.md §2.1, Appendix A token migration map." >&2
    exit 1
fi

echo "✓ Token discipline holds — no arbitrary color values found."
exit 0
