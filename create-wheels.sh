#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Change to the root directory
cd "$(dirname "$0")"

VERSION="0.2.0"
PACKAGE_NAME="chart_engine"
SOURCE_DIR="src/chart_engine"

echo "🚀 Starting maturin-based development build for ${PACKAGE_NAME} v${VERSION}..."

# 0. Cleanup old artifacts
echo "🧹 Cleaning up old binaries and libraries..."
rm -f src/chart_engine/chart_engine
rm -f src/chart_engine/chart_engine_lib*.so
rm -rf src/chart_engine/__pycache__

# 1. Install UI dependencies
if [ ! -d "src/node_modules" ]; then
    echo "📦 Node modules not found in src/. Installing UI dependencies..."
    cd src && npm install && cd ..
else
    echo "✅ UI dependencies found in src/node_modules."
fi

# 2. Build via Maturin (Consolidated Build)
echo "🛠 Building optimized Python extension and standalone binary..."
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# maturin develop --release will build both the .so and the bin in release mode
maturin develop --release --features python-bridge

# 3. Prepare the binary and library for the Python package
echo "📂 Synchronizing binary and library to package directory..."
BPATH="src/src-tauri/target/release/chart_engine_lib"

if [ -f "$BPATH" ]; then
    cp "$BPATH" "src/chart_engine/chart_engine"
    chmod +x "src/chart_engine/chart_engine"
    echo "✅ Standalone binary updated in src/chart_engine/chart_engine"
else
    echo "❌ Error: Could not find built binary at $BPATH"
    exit 1
fi

# Find the .so library
LPATH="src/src-tauri/target/release/libchart_engine_lib.so"
if [ ! -f "$LPATH" ]; then
    LPATH=$(find src/src-tauri/target/release/deps -name "libchart_engine_lib.so" | head -n 1)
fi

if [ -f "$LPATH" ]; then
    cp "$LPATH" "src/chart_engine/chart_engine_lib.so"
    echo "✅ Library updated in src/chart_engine/chart_engine_lib.so"
else
    echo "❌ Error: Could not find built library (.so)"
    exit 1
fi

echo "✨ Maturin develop complete! Package ${PACKAGE_NAME} is now installed in your environment."
echo "You can verify with: python3 test_install.py"
