#!/usr/bin/env bash

# Backfill deterministic activity IDs by re-writing existing rows so their
# primary keys follow the activity:post-<slug> pattern expected by the
# newsletter sync job.

set -euo pipefail

if ! command -v surreal >/dev/null 2>&1; then
  echo "error: surreal CLI not found in PATH" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for JSON parsing" >&2
  exit 1
fi

SURREAL_PROTOCOL="${SURREAL_PROTOCOL:-http}"
SURREAL_HOST="${SURREAL_HOST:-127.0.0.1:8000}"
SURREAL_NS="${SURREAL_NS:-rustblog}"
SURREAL_DB="${SURREAL_DB:-rustblog}"
SURREAL_USER="${SURREAL_USER:-${SURREAL_ROOT_USER:-root}}"
SURREAL_PASS="${SURREAL_PASS:-${SURREAL_ROOT_PASS:-root}}"
BASE_URL_PREFIX="${ACTIVITY_BASE_URL_PREFIX:-https://alexthola.com/post/}"

conn_url="${SURREAL_PROTOCOL}://${SURREAL_HOST}"

run_sql() {
  local query=$1
  surreal sql \
    --conn "$conn_url" \
    --ns "$SURREAL_NS" \
    --db "$SURREAL_DB" \
    --user "$SURREAL_USER" \
    --pass "$SURREAL_PASS" \
    --multi \
    "$query" >/dev/null
}

echo "Fetching existing activity records from ${conn_url} (${SURREAL_NS}/${SURREAL_DB})..."
raw=$(surreal sql \
  --conn "$conn_url" \
  --ns "$SURREAL_NS" \
  --db "$SURREAL_DB" \
  --user "$SURREAL_USER" \
  --pass "$SURREAL_PASS" \
  --json \
  "SELECT id, content, tags, source, created_at FROM activity;")

records=()
mapfile -t records < <(echo "$raw" | jq -r '.[0].result // [] | .[] | @base64')

if [[ ${#records[@]} -eq 0 ]]; then
  echo "No activity records found. Nothing to backfill."
  exit 0
fi

migrated=0
skipped=0
failed=0

decode() {
  echo "$1" | base64 --decode
}

slug_from_source() {
  local source=$1
  if [[ -z "$source" || $source != ${BASE_URL_PREFIX}* ]]; then
    return 1
  fi
  local slug=${source#${BASE_URL_PREFIX}}
  slug=${slug%%[?#]*}
  slug=${slug%%/*}
  slug=${slug// /-}
  slug=$(echo "$slug" | tr '[:upper:]' '[:lower:]')
  slug=${slug//[^a-z0-9_-]/-}
  slug=$(echo "$slug" | sed 's/-\{2,\}/-/g; s/^-//; s/-$//')
  if [[ -z "$slug" ]]; then
    return 1
  fi
  echo "$slug"
}

for encoded in "${records[@]}"; do
  record_json=$(decode "$encoded")
  current_id=$(echo "$record_json" | jq -r '.id // empty')
  source=$(echo "$record_json" | jq -r '.source // empty')

  if [[ -z "$current_id" ]]; then
    echo "Skipping record with missing id" >&2
    ((skipped++))
    continue
  fi

  slug=$(slug_from_source "$source" || true)
  if [[ -z "$slug" ]]; then
    echo "Skipping $current_id (no newsletter-friendly source URL)"
    ((skipped++))
    continue
  fi

  new_key="post-${slug}"
  new_id="activity:${new_key}"

  if [[ "$current_id" == "$new_id" ]]; then
    ((skipped++))
    continue
  fi

  payload=$(echo "$record_json" | jq -c 'del(.id)')
  if [[ -z "$payload" ]]; then
    echo "Failed to serialize payload for $current_id" >&2
    ((failed++))
    continue
  fi

  upsert_query=$(cat <<SURQL
UPSERT type::record("activity", "${new_key}") CONTENT ${payload};
SURQL
  )

  delete_query="DELETE ${current_id};"

  if run_sql "${upsert_query}${delete_query}"; then
    echo "Migrated ${current_id} -> ${new_id}"
    ((migrated++))
  else
    echo "Failed to migrate ${current_id}" >&2
    ((failed++))
  fi
done

echo "Backfill complete: migrated=${migrated}, skipped=${skipped}, failed=${failed}" 

if [[ $failed -gt 0 ]]; then
  exit 1
fi
