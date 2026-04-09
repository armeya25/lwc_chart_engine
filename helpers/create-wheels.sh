#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Change to the project root directory
cd "$(dirname "$0")/.."

VERSION="0.5.0"
PACKAGE_NAME="chart_engine"
SOURCE_DIR="src/chart_engine"

echo "🚀 Starting maturin-based development build for ${PACKAGE_NAME} v${VERSION}..."

# 0. Virtual Environment Check
if [ ! -d ".venv" ]; then
    echo "🐍 .venv/ not found. Attempting to create a new virtual environment..."
    if python3 -m venv .venv 2>/dev/null; then
        echo "✅ .venv created successfully."
    elif python3 -m venv --without-pip .venv 2>/dev/null; then
        echo "✅ .venv created successfully (without pip fallback)."
    else
        echo "⚠️ .venv could not be created (python3-venv missing)."
        rm -rf .venv
        echo "💡 Continuing with your current environment."
    fi
fi

if [ -f ".venv/bin/activate" ]; then
    echo "🐍 Using virtual environment in .venv/"
    source .venv/bin/activate
fi

# Ensure core build dependencies are installed
echo "📦 Ensuring build dependencies (maturin, polars) are up-to-date..."
# Use python3 -m pip instead of direct pip
python3 -m pip install --quiet --upgrade pip maturin polars || echo "⚠️ Could not update pip/maturin/polars automatically."

# 0. Cleanup old artifacts
echo "🧹 Clearing previous build artifacts..."
rm -rf wheels
mkdir -p wheels
rm -f src/chart_engine/chart_engine
rm -f src/chart_engine/chart_engine_lib*.so
rm -rf src/chart_engine/__pycache__

# 1. Install UI dependencies
# 1. Install UI dependencies
echo "📦 Synchronizing UI dependencies..."
cd src && npm install && cd ..

# 2. Build minified frontend
echo "📦 Building minified frontend..."
cd src && npm run build:frontend && cd ..

# 3. Build via Maturin (Consolidated Build)
echo "🛠 Building optimized Python extension and standalone binary..."
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# First, ensure binaries are built so we can compress them before packaging
echo "📂 Compiling and preparing binaries for packaging..."
echo "Current Directory: $(pwd)"
ls -l src/src-tauri/Cargo.toml || echo "❌ Error: Cannot find Cargo.toml at $(pwd)/src/src-tauri/Cargo.toml"
cargo build --release --manifest-path src/src-tauri/Cargo.toml --features python-bridge

# Standalone binary
BPATH="src/src-tauri/target/release/chart_engine"
if [ -f "$BPATH" ]; then
    cp "$BPATH" "src/chart_engine/chart_engine"
    chmod +x "src/chart_engine/chart_engine"
    # Skip early packing; we'll do it once inside the wheel
    echo "✅ Standalone binary ready for packaging."
else
    echo "❌ Error: Could not find built binary at $BPATH"
    exit 1
fi

# Shared library (Optional for packaging, but good for local)
LPATH="src/src-tauri/target/release/libchart_engine_lib.so"
if [ ! -f "$LPATH" ]; then
    LPATH=$(find src/src-tauri/target/release/deps -name "libchart_engine_lib.so" | head -n 1)
fi
if [ -f "$LPATH" ]; then
    cp "$LPATH" "src/chart_engine/chart_engine_lib.so"
fi

# Build the wheel (.whl) for distribution
echo "📦 Generating production wheel (lightweight)..."
mkdir -p wheels
rm -f src/chart_engine/chart_engine_lib.so # Never include manually copied libs in the wheel
maturin build --release --features python-bridge --out wheels --manylinux 2_39

# High Compression Phase
echo "🗜 Starting High Compression phase for the .whl..."
WHEEL_FILE=$(ls wheels/*.whl | head -n 1)
if [ -f "$WHEEL_FILE" ]; then
    TMP_DIR=$(mktemp -d)
    echo "📂 Unpacking wheel to $TMP_DIR..."
    python3 -m zipfile -e "$WHEEL_FILE" "$TMP_DIR"
    
    echo "📂 Restoring executable permissions before compression..."
    find "$TMP_DIR" -type f -name "*.so" -exec chmod +x {} +
    find "$TMP_DIR" -type f -name "chart_engine" -exec chmod +x {} +

    echo "⚡ High-compressing internal binaries (UPX)..."
    # Use -type f to skip directories and --force just in case
    find "$TMP_DIR" -type f -name "*.so" -exec ./upx --best --lzma --force {} + || echo "⚠️ Internal UPX failed for .so"
    find "$TMP_DIR" -type f -name "chart_engine" -exec ./upx --best --lzma --force {} + || echo "⚠️ Internal UPX failed for binary"
    
    echo "✍ Updating RECORD file hashes and sizes..."
    # We need to update the SHA256 and size for all modified files in the RECORD file
    # This is a bit complex in bash, so we'll use a python one-liner to fix the RECORD file
    python3 -c "
import os, hashlib, base64
record_path = next(iter(path for path in [os.path.join(r, 'RECORD') for r, d, f in os.walk('$TMP_DIR')] if os.path.exists(path)), None)
if record_path:
    lines = []
    with open(record_path, 'r') as f:
        for line in f:
            parts = line.strip().split(',')
            if len(parts) >= 3:
                rel_path = parts[0]
                full_path = os.path.join('$TMP_DIR', rel_path)
                if os.path.exists(full_path) and not rel_path.endswith('RECORD'):
                    size = os.path.getsize(full_path)
                    with open(full_path, 'rb') as bf:
                        sha = base64.urlsafe_b64encode(hashlib.sha256(bf.read()).digest()).decode().rstrip('=')
                    parts[1] = f'sha256={sha}'
                    parts[2] = str(size)
            lines.append(','.join(parts))
    with open(record_path, 'w') as f:
        f.write('\n'.join(lines) + '\n')
"
    
    echo "📦 Repacking highly-compressed wheel..."
    WHEEL_NAME=$(basename "$WHEEL_FILE")
    WHEEL_OUT_DIR=$(realpath wheels)
    python3 -c "import zipfile, os;
with zipfile.ZipFile('$WHEEL_OUT_DIR/$WHEEL_NAME.new', 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as zf:
    for root, dirs, files in os.walk('$TMP_DIR'):
        for file in files:
            full_path = os.path.join(root, file)
            rel_path = os.path.relpath(full_path, '$TMP_DIR')
            # Preserve file permissions (external_attr)
            st = os.stat(full_path)
            zinfo = zipfile.ZipInfo.from_file(full_path, rel_path)
            zinfo.external_attr = (st.st_mode & 0xFFFF) << 16
            with open(full_path, 'rb') as f:
                zf.writestr(zinfo, f.read(), compress_type=zipfile.ZIP_DEFLATED)
"
    mv "$WHEEL_OUT_DIR/$WHEEL_NAME.new" "$WHEEL_FILE"
    rm -rf "$TMP_DIR"
    echo "✅ High compression complete."
fi

# Final Cleanup: Remove staging binaries
echo "🧹 Final cleanup of source directory..."
rm -f src/chart_engine/chart_engine
rm -f src/chart_engine/chart_engine_lib*.so

echo "✨ Build complete! Your highly-compressed wheel is in: wheels/"
