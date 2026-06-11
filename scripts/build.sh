#!/bin/bash
set -e

echo "Building all Strelligence contracts..."

cargo build --target wasm32v1-none --release --package recurring-registry
echo "✓ recurring-registry built"

cargo build --target wasm32v1-none --release --package metadata-registry
echo "✓ metadata-registry built"

cargo build --target wasm32v1-none --release --package automation-rules
echo "✓ automation-rules built"

echo ""
echo "All contracts built. WASM files in target/wasm32v1-none/release/"
ls target/wasm32v1-none/release/*.wasm