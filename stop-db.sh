#!/bin/bash

echo "Stopping database server..."
pkill -f "surreal" 2>/dev/null || true
sleep 1
pkill -f "surreal" 2>/dev/null || true
echo "Database server stopped"