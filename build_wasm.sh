#!/bin/bash
# Build script for WASM target

set -e

echo "🦀 Building iDos Games SDK for WebAssembly..."

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "📦 Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build the project
echo "🔨 Building..."
cargo build --release --target wasm32-unknown-unknown --features all

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "⚠️  wasm-bindgen-cli not found. Install it with:"
    echo "   cargo install wasm-bindgen-cli"
    exit 1
fi

# Generate bindings
echo "🔗 Generating WASM bindings..."
mkdir -p dist
wasm-bindgen \
    --out-dir ./dist \
    --target web \
    ./target/wasm32-unknown-unknown/release/*.wasm

# Optimize if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM..."
    for file in dist/*_bg.wasm; do
        wasm-opt -Oz -o "${file%.wasm}_opt.wasm" "$file"
        mv "${file%.wasm}_opt.wasm" "$file"
    done
else
    echo "💡 Tip: Install wasm-opt for smaller builds:"
    echo "   cargo install wasm-opt"
fi

echo "✅ Build complete! Output in ./dist/"
echo "🚀 Serve with: python3 -m http.server 8000 --directory dist"

