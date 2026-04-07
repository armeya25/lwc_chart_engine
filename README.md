# 🚀 LWC Chart Engine

A high-performance charting engine built with **Rust**, **Tauri**, and **Lightweight Charts**. This library provides a seamless, non-blocking Python API for streaming and visualizing large datasets via **Polars**.

![Static Chart Example](screenshots/static.png)

## 💎 Features

- **High Performance**: Native Rust backend for low-latency data streaming.
- **Embedded UI**: Minified frontend assets bundled directly into the distribution.
- **Python Integration**: First-class support for **Polars DataFrames**.

## 🚀 Quick Start

We provide scripts to automate the build and installation process. Choose the one that fits your needs:

### 🛠 Development Build
For a fast development cycle (installs in your current environment):
```bash
./dev.sh
```

### 📦 Production Build
To generate a distribution-ready wheel:
```bash
./create-wheels.sh
```

## 📊 Usage & Examples

Detailed usage examples can be found in the [examples/](examples/) directory:

- [static_charts.py](examples/static_charts.py): Basic usage with Polars DataFrames.
- [live_chart_emulation.py](examples/live_chart_emulation.py): Real-time data streaming simulation.

## 🏗 Prerequisites

To build the package from source, ensure you have the following toolchains and dependencies installed.

### ⚙ Toolchains
- **Rust**: Latest stable (cargo, rustc).
- **Python**: 3.12+ (or 3.13 for the provided wheel).
- **Node.js**: Needed for frontend asset minification (`esbuild`).
- **Maturin**: `pip install maturin` (for building the Python extension).

### 🐧 Linux Dependencies
**Ubuntu / Debian:**
```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

**Arch Linux:**
```bash
sudo pacman -S gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg
```

### 🍎 macOS Dependencies
Install the Xcode Command Line Tools:
```bash
xcode-select --install
```

### 🪟 Windows Dependencies
1. **Visual Studio 2022**: Install the [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. **WebView2**: Most recent Windows versions include this by default. If not, install the [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

---
*build by amit vaidya*