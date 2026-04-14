#!/usr/bin/env bash
set -euo pipefail

ARG="${1:-}"
current=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//')

if [[ "$ARG" == v=* ]]; then
  next="${ARG#v=}"
elif [[ "$ARG" == "patch" || "$ARG" == "minor" || "$ARG" == "major" ]]; then
  IFS='.' read -r ma mi pa <<< "$current"
  case "$ARG" in
    major) next="$((ma+1)).0.0" ;;
    minor) next="$ma.$((mi+1)).0" ;;
    patch) next="$ma.$mi.$((pa+1))" ;;
  esac
else
  echo "Usage: bun run bump patch|minor|major  or  bun run bump v=X.Y.Z"
  exit 1
fi

echo "$current → $next"
sed -i "s/\"version\": \".*\"/\"version\": \"$next\"/" src-tauri/tauri.conf.json
sed -i "s/\"version\": \".*\"/\"version\": \"$next\"/" package.json
sed -i "s/^version = \".*\"/version = \"$next\"/" src-tauri/Cargo.toml
sed -i "s/APP_VERSION = '.*'/APP_VERSION = '$next'/" src/lib/constants.ts
echo "→ v$next"
