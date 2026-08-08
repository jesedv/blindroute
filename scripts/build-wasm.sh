#!/usr/bin/env bash
# Build the WASM bundle into web/public/pkg, then rebuild the static site.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> cargo test (native)"
cargo test --workspace

echo "==> wasm-pack build (web target)"
wasm-pack build crates/ringcrypt-wasm --target web --out-dir ../web/public/pkg

echo "==> npm install + build site"
cd web
npm install
npm run build

echo "==> dist ready at web/dist  (deploy this folder)"
