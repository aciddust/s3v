#!/usr/bin/env bash
set -euo pipefail

bash scripts/bump.sh "${1:-}"
next=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//')

echo ""
echo "Verifying v$next across files..."
ok=true
for f in src-tauri/tauri.conf.json package.json; do
  fv=$(grep '"version"' "$f" | head -1 | sed 's/.*: "//;s/".*//')
  if [ "$fv" = "$next" ]; then echo "  ✓ $f → $fv"
  else echo "  ✗ $f → $fv (expected $next)"; ok=false; fi
done
fv=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/.*= "//;s/".*//')
if [ "$fv" = "$next" ]; then echo "  ✓ src-tauri/Cargo.toml → $fv"
else echo "  ✗ src-tauri/Cargo.toml → $fv (expected $next)"; ok=false; fi
fv=$(grep "APP_VERSION" src/lib/constants.ts | sed "s/.*= '//;s/'.*//")
if [ "$fv" = "$next" ]; then echo "  ✓ src/lib/constants.ts → $fv"
else echo "  ✗ src/lib/constants.ts → $fv (expected $next)"; ok=false; fi

if [ "$ok" = "false" ]; then echo "Version mismatch. Aborting."; exit 1; fi

echo ""
echo "Committing & tagging v$next..."
git add -A
git commit -m "chore: bump v$next"
git tag -a "v$next" -m "Release v$next"
git push origin main "v$next"
echo ""
echo "✓ Released v$next — CI build triggered"
