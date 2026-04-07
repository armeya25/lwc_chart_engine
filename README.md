# 🚀 LWC Chart Engine

A high-performance charting engine built with **Rust**, **Tauri**, and **Lightweight Charts**. This library provides a seamless, non-blocking Python API for streaming and visualizing large datasets via **Polars**.

## 💎 Features
- **High Performance**: Native Rust backend for low-latency data streaming.
- **Aggressive Optimization**: Binary size reduced from 79MB to **4.7MB** (via Thin LTO, UPX, and dependency pruning).
- **Embedded UI**: Minified frontend assets bundled directly into the distribution.
- **Python Integration**: First-class support for **Polars DataFrames**.

![Static Chart Example](screenshots/static.png)

## 🏗 Build Requirements

To build the optimized package from source, ensure you have the following installed:

### ⚙ Toolchains
- **Rust**: Latest stable (cargo, rustc).
- **Python**: 3.12+ (or 3.13 for the provided wheel).
- **Node.js**: Needed for frontend asset minification (`esbuild`).

### 🛠 Tools & Utilities
- **Maturin**: `pip install maturin` (for building the Python extension).
- **UPX**: Required for binary compression.
  - **Ubuntu/Debian**: `sudo apt install upx-ucl`
  - **Arch Linux**: `sudo pacman -S upx`
  - **macOS**: `brew install upx`
  - *Note: Alternatively, place the `upx` binary directly in the project root if it is not in your PATH.*

### 🐧 System Dependencies (Linux)

#### Ubuntu / Debian:
```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Arch Linux:
```bash
sudo pacman -S gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg
```

### 🍎 System Dependencies (macOS)
Install the Xcode Command Line Tools:
```bash
xcode-select --install
```

### 🪟 System Dependencies (Windows)
1. **Visual Studio 2022**: Install the [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. **WebView2**: Most recent Windows versions include this by default. If not, install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

## 📦 Installation

Install the production-ready wheel from the `wheels/` directory:

```bash
# 1. Install dependencies
pip install -r requirements.txt

# 2. Install the wheel
pip install wheels/chart_engine-0.2.9-cp313-cp313-linux_x86_64.whl
```

## 🛠 Usage

```python
import polars as pl
from chart_engine import Chart
import time

# 1. Initialize and Show Chart
chart = Chart(title="Chart Engine - SubChart Test")
chart.show()  # Launch the Tauri window

# 2. Configure Layout and Series
subcharts = chart.set_layout("single")
series = subcharts[0].create_candlestick_series(name="BTC/USD")

# 3. Load and Set Data (Polars DataFrame)
# Assumes a parquet file exists at data/1d.parquet
df = pl.read_parquet("data/1d.parquet").tail(100)
series.set_data(df)

print("✅ Data series set successfully. Window should be open.")

# Keep the script alive to see the window
#time.sleep(10) not needed
```



## 🏗 Build Pipeline

We provide a consolidated build script `create-wheels.sh` that automates the entire optimization and packaging lifecycle.

### Build Optimized Wheel:
```bash
./create-wheels.sh
```

### What happens in the pipeline:
1. **Frontend**: Minifies JS/CSS using `esbuild` and bundles them into the binary.
2. **Backend**: Compiles Rust with `opt-level="z"`, `strip=true`, and `Thin LTO`.
3. **Compression**: Applies `UPX --best --lzma` to internal binaries.
4. **Packaging**: Generates a lean manylinux wheel, unpacks it for internal secondary compression, and repacks with maximum ZIP compression.

## 📊 Optimization Audit

Our latest build achieved a **~94% reduction** in distribution size compared to standard builds:

| Component | Optimized Size | Reduction |
| :--- | :--- | :--- |
| **Standalone Binary** | **0.9 MB** | **96%** |
| **Python Extension** | **8.2 MB** | **71%** |
| **Final Wheel (.whl)** | **4.7 MB** | **94%** |

---
*Built with ❤️ by the Antigravity Team.*
*build by amit vaidya*