#!/usr/bin/env bash
set -euo pipefail

echo "Building SWC plugin for wasm32-wasip1..."
cargo build --release --target wasm32-wasip1

WASM_FILE="target/wasm32-wasip1/release/swc_jest_coverage_nestjs_plugin.wasm"

if [ -f "$WASM_FILE" ]; then
    cp "$WASM_FILE" ./swc_jest_coverage_nestjs_plugin.wasm
    echo "Build complete: swc_jest_coverage_nestjs_plugin.wasm ($(du -h swc_jest_coverage_nestjs_plugin.wasm | cut -f1))"
else
    echo "ERROR: WASM file not found at $WASM_FILE"
    exit 1
fi
