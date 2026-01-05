#!/bin/bash

set -euo pipefail

echo "Stopping database server..."

PID_DIR=${PID_DIR:-target/tmp}

# Stop tracked PIDs first (supports parallel runs)
shopt -s nullglob
for pidfile in "$PID_DIR"/blog_db_pid.*; do
	[ -f "$pidfile" ] || continue
	pid=$(cat "$pidfile" 2>/dev/null || true)
	if [ -n "${pid:-}" ] && kill -0 "$pid" 2>/dev/null; then
		kill "$pid" 2>/dev/null || true
		for _ in $(seq 1 30); do
			kill -0 "$pid" 2>/dev/null || break
			sleep 0.2
		done
		kill -9 "$pid" 2>/dev/null || true
	fi
	rm -f "$pidfile" 2>/dev/null || true

done

# Fallback: best-effort stop by pattern (avoid pkill -f)
for pat in "surreal start" "surrealkv"; do
	pids=$(pgrep -f "$pat" 2>/dev/null || true)
	if [ -n "${pids:-}" ]; then
		kill $pids 2>/dev/null || true
	fi
done

echo "Database server stopped"
