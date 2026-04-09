# 🚀 LWC Chart Engine

A high-performance charting engine built with **Rust**, **Tauri**, and **Lightweight Charts**. This library provides a seamless, non-blocking Python API for streaming and visualizing large datasets via **Polars**.

**<u>[installable wheels for Os are in wheels/ folder](wheels/)</u>**

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
./helpers/dev.sh
```

### 📦 Production Build
To generate a distribution-ready wheel:
```bash
./helpers/create-wheels.sh
```

### 📊 Examples
Dive into the `examples/` directory to see the full capabilities:
- **[static_charts.py](file:///home/armeya/Documents/lwc_chart_engine/examples/static_charts.py)**: Basic rendering of historical OHLC data.
- **[live_chart_emulation.py](file:///home/armeya/Documents/lwc_chart_engine/examples/live_chart_emulation.py)**: Real-time data streaming and auto-scrolling.
- **[drawing_tools.py](file:///home/armeya/Documents/lwc_chart_engine/examples/drawing_tools.py)**: Programmatic Trendlines, Rays, Fibonacci, and Boxes.
- **[multi_chart_layouts.py](file:///home/armeya/Documents/lwc_chart_engine/examples/multi_chart_layouts.py)**: Building complex workspaces with 2, 3, or 4 subcharts.
- **[paper_trading.py](file:///home/armeya/Documents/lwc_chart_engine/examples/paper_trading.py)**: Backend programmatic execution and TP/SL visual management.
- **[ui_customization.py](file:///home/armeya/Documents/lwc_chart_engine/examples/ui_customization.py)**: Full control over tooltips, watermarks, timezones, and legends.

## 📚 Documentation

For a full list of methods, configuration options, and advanced drawing logic, see the **[API Documentation](docs/api.md)**.

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