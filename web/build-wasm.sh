#!/bin/bash

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Please install it with:"
    echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build the WASM module with debug info
echo "Building WASM module with debug info..."
cd ../crates/seal-cli || exit 1
wasm-pack build --target web --debug --out-dir ../../web/src/wasm

echo "WASM module built successfully!"