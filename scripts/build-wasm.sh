#!/usr/bin/env bash
# Builds the Rust core + LBX importer to WebAssembly and generates the JS bindings
# into web/src/wasm (used by the web and desktop builds).
# Requires: rustup target wasm32-unknown-unknown, wasm-bindgen-cli matching Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/.."

profile="${1:-release}"
want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/, ""); print; exit}' Cargo.lock)
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen-cli $want is required (found: ${have:-none})." >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

flag=$([ "$profile" = "release" ] && echo "--release" || echo "")
cargo build -p aoo-wasm --target wasm32-unknown-unknown $flag
wasm-bindgen --target web --out-dir web/src/wasm --out-name aoo \
  "target/wasm32-unknown-unknown/$profile/aoo_wasm.wasm"
if command -v wasm-opt >/dev/null && [ "$profile" = "release" ]; then
  wasm-opt -Os --enable-bulk-memory --enable-nontrapping-float-to-int \
    web/src/wasm/aoo_bg.wasm -o web/src/wasm/aoo_bg.wasm
fi
ls -l web/src/wasm
