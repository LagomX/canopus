#!/bin/bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
CONCURRENTLY="$ROOT/frontend/node_modules/.bin/concurrently"

# Install frontend deps if needed
if [ ! -f "$CONCURRENTLY" ]; then
  echo "Installing frontend dependencies..."
  cd "$ROOT/frontend" && pnpm install
fi

"$CONCURRENTLY" \
  --names "ollama,rust,next" \
  --prefix-colors "cyan,red,blue" \
  "ollama serve" \
  "cargo run --bin canopus-dashboard --manifest-path $ROOT/Cargo.toml" \
  "bash -c 'cd $ROOT/frontend && pnpm dev'"
