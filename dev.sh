#!/bin/bash
set -e

# Change to the root directory
cd "$(dirname "$0")"

# 1. Source virtual environment
if [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
fi

echo "🚀 Starting fast development build..."

# 2. Build and install the Python extension (Debug mode)
# This is much faster than --release and installs directly into your venv
maturin develop --features python-bridge

# 3. Build the standalone Tauri binary (Debug mode)
echo "📂 Compiling standalone binary..."
cargo build --manifest-path src/src-tauri/Cargo.toml --features python-bridge

# 4. Copy to the location expected by chart.py
cp src/src-tauri/target/debug/chart_engine_lib src/chart_engine/chart_engine
chmod +x src/chart_engine/chart_engine

echo "✅ Dev build complete! You can now run your scripts."
