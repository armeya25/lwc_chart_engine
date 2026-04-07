#!/bin/bash
set -e

# Change to the root directory
cd "$(dirname "$0")"

# 1. Source virtual environment
if [ -f ".venv/bin/activate" ]; then
    source .venv/bin/activate
fi

echo "🚀 Starting fast development build..."

# 1.5. Build minified frontend
echo "📦 Building minified frontend..."
(cd src && npm run build:frontend) > /dev/null 2>&1

# 2. Build and install the Python extension
echo "📦 Building debug wheel..."
# Redirect noise to dev null, but keep output on error
maturin build --out wheels --features python-bridge > /tmp/maturin_build.log 2>&1 || { cat /tmp/maturin_build.log; exit 1; }

# Install the wheel quietly
echo "📥 Installing wheel..."
uv pip install wheels/chart_engine-0.3.5*.whl --force-reinstall --quiet

# 3. Build the standalone Tauri binary
echo "📂 Compiling standalone binary..."
cargo build --manifest-path src/src-tauri/Cargo.toml --features python-bridge > /tmp/cargo_build.log 2>&1 || { cat /tmp/cargo_build.log; exit 1; }

# 4. Copy to the location expected by chart.py
cp src/src-tauri/target/debug/chart_engine_lib src/chart_engine/chart_engine
chmod +x src/chart_engine/chart_engine

echo "✅ Dev build complete! You can now run your scripts."
