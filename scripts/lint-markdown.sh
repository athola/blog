#!/usr/bin/env bash
set -euo pipefail

if ! command -v npx >/dev/null 2>&1; then
  echo "markdownlint: npx is required. Install Node.js/npm first." >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
CACHE_DIR="$REPO_ROOT/.cache/npm"
mkdir -p "$CACHE_DIR"
export NPM_CONFIG_CACHE="$CACHE_DIR"

CMD=(npx --yes markdownlint-cli2@0.15.0)
PATTERNS=(
  "**/*.md"
  "!target/**"
  "!node_modules/**"
  "!book/**"
  "!public/**"
  "!.cache/**"
  "!writing-clearly-and-concisely/**"
)

echo "running markdownlint-cli2..."
"${CMD[@]}" "${PATTERNS[@]}"
