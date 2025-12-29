#!/usr/bin/env bash
set -euo pipefail

wasm-pack build \
  --target web \
  --out-dir ../pigmora-frontend/src/wasm/pkg \
  --release
