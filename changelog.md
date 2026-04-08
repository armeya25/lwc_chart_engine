# Changelog

All notable changes to the **LWC Chart Engine** will be documented in this file.

## [0.3.5] - 2026-04-08

### 🛠 Build & Workflow Optimizations
- **Cross-Platform Packaging**: Replaced system-level `zip` and `unzip` dependencies with robust Python-based implementations, resolving "command not found" errors on Windows CI runners.
- **CI/CD Pipeline Fixes**: 
  - Resolved linker errors on macOS and Windows by properly isolating the `python-bridge` feature.
  - Fixed YAML syntax and indentation errors in the `build_wheels.yml` workflow.
  - Eliminated redundant repackaging steps for improved pipeline efficiency.
- **Improved Windows CI**: Rewrote the frontend build stage to execute directly via Bash, ensuring compatibility for Unix-style commands on Windows runners.
- **Official GitHub Environment**: Migrated all automated actions to their latest versions (**v6**) with native Node.js 24 support.
- **Enhanced Identification**: Automated wheel renaming to include OS suffixes (`-windows`, `-linux`, `-macos`) for easier distribution management.

## [0.3.0] - 2026-04-07

### 🚀 Core Improvements / Features
- **API Documentation Overhaul**: Completely rewrote `api.md` to provide full technical coverage.
  - Replaced all generic `**kwargs` with explicit, named parameters and types.
  - Added detailed documentation for `Chart` constructor, `Series.set_data`, and `SubChart` classes.
  - Documented all drawing tools including Trends, Rays, Fibonacci, and Boxes with their specific visual options.

### 🛠 Build & Workflow Optimizations
- **Automated Multi-Platform CI/CD**: Implemented a GitHub Actions workflow to build highly-compressed Python wheels for Windows, Linux, and macOS.
  - Integrated **UPX compression** into the automated build pipeline for lightweight distributions.
  - Automated "commit-back" logic to synchronize built wheels directly into the repository's `wheels/` folder.
- **Global Version Synchronization**: Synchronized version `0.3.0` across `pyproject.toml`, `Cargo.toml`, and build scripts.

## [0.2.9] - 2026-04-07

### 🎯 UI & API Enhancements
- **Programmatic Layout Control**: Hidden the layout selection toolbar from the JS frontend by default to provide a cleaner initial interface.
- **New Visibility API**: Added `chart.enable_layout_toolbar()` and `chart.disable_layout_toolbar()` to the Python API.
- **Backend Synchronization**: Integrated layout toolbar state into the Rust `Chart` struct for consistent cross-platform state management.

## [0.2.8] - 2026-04-07

### 🚀 Core Improvements / Features
- **Rust Paper Trading Engine (v2)**: Finalized the high-performance Rust implementation for position, PnL, and TP/SL tracking.
  - Resolved `AttributeError: Chart has no attribute 'trader'` by ensuring correct PyO3 registration.
  - Implemented `set_tooltip`, `enable_tooltip`, and `disable_tooltip` with centralized state in Rust.
- **Stable Lifecycle Management**: 
  - Added `chart.show()` to prevent Python script exit while the window is active.
  - Integrated `atexit` for reliable Rust process termination.

### 🛠 Build & Workflow Optimizations
- **Enhanced `dev.sh`**: Added native support for `uv` environments (`maturin develop --uv`) for ultra-fast builds.
- **Source Cleanliness**: Added post-build automatic cleanup of temporary staging binaries in `create-wheels.sh`.

### 🎯 UI & API Enhancements
- **Custom Visuals**: Removed the permanent "LIVE" indicator from the header for a cleaner Look-and-Feel.
- **Dynamic Tooltips**: Tooltips are now conditionally hidden based on user preference, with state managed by the backend.

## [0.2.7] - 2026-04-07

### 🚀 Core Improvements (Rust Migration)
- **High-Performance Paper Trader**: Migrated the entire paper trading engine from Python to Rust.
  - Position state, PnL calculations, and TP/SL monitoring now run at host-speed in the backend.
  - Reduced Python-to-UI overhead for high-frequency price updates.
- **Unified Trader API**: Exposed `PaperTrader` and `Position` classes via PyO3 for direct inspection and manipulation.

### 🛠 Build & Workflow Optimizations
- **New Fast Dev Workflow**: Created `dev.sh` to support rapid iteration.
  - Uses `maturin develop --uv` for instantaneous environment updates.
  - Skips heavy UPX compression and packaging overhead.
- **Production Build Fixes**: 
  - Resolved UPX "corrupted file" errors by adjusting Rust `strip` settings to `debuginfo`.
  - Added automated post-build cleanup to `create-wheels.sh` for a pristine source directory.
  - Fixed `pip` command visibility issues in `uv`-based virtual environments.

### 🎯 UI & API Enhancements
- **Clean UI**: Removed the legacy "LIVE" status indicator from the frontend header for a more professional aesthetic.
- **Intelligent Tooltips**: Tooltips are now disabled by default. 
  - Added `chart.enable_tooltip()` and `chart.disable_tooltip()` methods to the Python API.
  - State is now centrally managed in the Rust backend.
- **Stability & Persistence**:
  - Added `chart.show(block=True)` to keep the Python script alive while the window is active.
  - Integrated `atexit` handlers to ensure the Rust `chart_engine` process is gracefully terminated when the Python script ends.

---
*maintained by amit vaidya*
