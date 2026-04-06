# LWC Chart Engine

A high-performance charting engine and trading terminal built with Rust, Tauri, and Lightweight Charts. This project features a robust data-processing backend implemented in Rust (exposed via Maturin for Python integration) and a professional-grade GUI using Tauri.

## Project Structure

- `chart_backend/`: The core data-processing engine. A Rust library that provides high-performance data manipulation, indicator calculation, and synchronization using Polars. Exposed as a Python module using Maturin.
- `chart_engine/`: The GUI shell. A Tauri application that hosts the `src-frontend` (HTML/CSS/JS) and interacts with the Rust backend.
- `chart_engine/src-frontend/`: The interactive charting interface built on top of [Lightweight Charts](https://github.com/tradingview/lightweight-charts).

## Prerequisites

Ensure you have the following installed on your system:

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install) (Cargo included).
- **Node.js**: [Install Node.js](https://nodejs.org/en) (for Tauri frontend dependencies).
- **Python 3.9+**: For building/using the backend as a library.
- **Maturin**: Install via pip: `pip install maturin`.
- **Tauri dependencies (Linux)**: Follow the [Tauri Linux setup guide](https://tauri.app/v1/guides/getting-started/prerequisites#linux).

## Build Instructions

### 1. Installation

Navigate to the `chart_engine` directory and install the required frontend dependencies. The core backend logic (from `chart_backend`) is built automatically by the Tauri compiler.

```bash
cd chart_engine
npm install
```

### 2. Run the Application

The application uses **Tauri** (which internally calls **Cargo**) to build and run the GUI. 

#### Development Mode
# can take 5 to 10 minutes to build depending upon system resources
To run the application with hot-reloading for both the Rust backend and JS frontend:

```bash
cd chart_engine
npm run tauri dev
```

### 3. Building the Native Binary

You can compile the application into a single, optimized native binary using either the Tauri CLI (recommended) or Cargo directly.

### Option 1: Using Tauri CLI (Standard)
This is the standard way to build the application, bundling both the frontend assets and the Rust code into a production-ready installer or binary.

```bash
cd chart_engine
npm run tauri build
```

### Option 2: Using Cargo Directly (Advanced)
If you only want to compile the Rust binary without the full Tauri bundling process (useful for testing production performance during dev):

```bash
cd chart_engine/src-tauri
cargo build --release
```

> [!NOTE]
> The resulting binary from Option 2 will rely on the assets being correctly path-mapped in your `tauri.conf.json`. For a complete distribution, always use Option 1.

## Performance Optimization

For maximum execution speed, especially when handling large datasets or high-frequency charting, you should build the binary with hardware-specific optimizations.

### 1. Build for Maximum Target Speed
Run the build with the `target-cpu=native` flag to allow the compiler to leverage All available CPU features (like AVX-512):

```bash
cd chart_engine
RUSTFLAGS="-C target-cpu=native" npm run tauri build
```

### 2. Runtime Performance Settings
The application is pre-configured in `chart_engine/src-tauri/Cargo.toml` with:
- **LTO (Link-Time Optimization)**: Enabled for full-binary visibility.
- **Panic Strategy**: Set to `abort` for reduced size and overhead.

### 3. Finding the Production Binary
The final, high-speed binary will be located at:
`chart_engine/src-tauri/target/release/tauri-app` (on Linux)
`chart_engine/src-tauri/target/release/tauri-app.exe` (on Windows)

## Features

- **High Performance**: Data handling powered by Rust and Polars for near-instant indicator calculations.
- **Modern UI**: Vertical-docked toolbar, slide-out menus, and professional dark-mode aesthetics.
- **Synchronization**: Multi-chart timezone synchronization and timestamp alignment.
- **Custom Indicators**: Extensible indicator system leveraging Rust's speed.

## Development Workflow

1. **Modifying Logic**: Edit files in `chart_backend/src` and `chart_engine/src-tauri/src`.
2. **Modifying UI**: Edit files in `chart_engine/src-frontend`.
3. **Debugging**: Use `npm run tauri dev` and inspect the console for frontend errors or Rust panic logs in the terminal.

---

*Authored by amit vaidya*
