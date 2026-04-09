# Changelog

All notable changes to the **LWC Chart Engine** will be documented in this file.

## [0.5.5] - 2026-04-09

### 🚀 Core Improvements & CI/CD
- **Summary**: Cross-Platform Build System.
- **Details**: Replaced POSIX-style shell scripts in `package.json` with a robust Node.js `build-frontend.js` script to resolve CI/CD failures on Windows runners.
- **Summary**: Environment Resilience.
- **Details**: Standardized all build and upload scripts to use the `dist/` directory for unified artifact management across local and GitHub environments.
- **Summary**: Force-Push Synchronization.
- **Details**: Updated `upload_to_git.sh` to a streamlined force-push model for faster synchronization and explicit remote state mirroring.

### 🎯 UI & API Enhancements
- **Summary**: Active Pane Highlighting.
- **Details**: Added visual focus indicators (glow/border) that follow interaction in multi-chart layouts.
- **Summary**: Automatic Data Type Fallbacks.
- **Details**: Built resiliency into the frontend to automatically map `close` prices to `value` fields for line/area series when provided with OHLC data.
- **Summary**: Live Marker Timestamping.
- **Details**: Fixed a crash in `trader_execute` by providing automatic real-time timestamping for markers when no explicit time is provided.
- **Summary**: Initialization Handshaking.
- **Details**: Added a deferred command queue in the frontend to ensure no data is lost during the startup "cold-start" phase.

## [0.5.2] - 2026-04-09

### 🚀 Core Improvements / Features
- **Summary**: Full Drawing Tool Suite.
- **Details**: Implemented a comprehensive set of interactive tools including Trendlines, Fibonacci Retracement, Supply Zone Boxes, Long/Short (Risk/Reward) positions, Extended Rays, and Horizontal Resistance lines.
- **Summary**: Interactive Marker API.
- **Details**: Added support for programmatic and interactive markers with alignment-aware coordinate matching.

### 🎯 UI & API Enhancements
- **Summary**: Reactive Resize Integration.
- **Details**: Implemented `ResizeObserver` for all chart cells to resolve "blank chart" rendering issues during multi-pane layout transitions.
- **Summary**: Active Pane Highlighting.
- **Details**: Added visual focus indicators (glow/border) that follow interaction in multi-chart layouts.
- **Summary**: Automatic Data Type Fallbacks.
- **Details**: Built resiliency into the frontend to automatically map `close` prices to `value` fields for line/area series when provided with OHLC data.
- **Summary**: Expanded API Examples.
- **Details**: Created and refined demonstration scripts (`drawing_tools.py`, `multi_chart_layouts.py`, `paper_trading.py`) for better feature discoverability.
- **Summary**: Fixed Python-Rust Bridge Conflicts.
- **Details**: Resolved `TypeError` in `Series.add_marker` where logic was double-passing arguments to the underlying bridge.

### 🛠 Build & Workflow Optimizations
- **Summary**: Robust Build Pipeline.
- **Details**: Refactored `create-wheels.sh` with subshells and relative path sanitization to prevent directory mismatch errors.
- **Summary**: ESM Consolidation.
- **Details**: Fixed module-level naming conflicts and centralized UI-related loading logic into the correct ES modules.
- **Summary**: Standardized CI/CD Distribution.
- **Details**: Aligned GitHub Actions and local scripts to use `dist/` as the primary artifact directory.

### ⚙ Internal Refactoring
- **Summary**: Rust Backend Modernization.
- **Details**: Resolved PyO3 deprecation warnings by updating `new_bound` to `new` across the time utilities module.

## [0.5.1] - 2026-04-09

### 🛠 Build & Workflow Optimizations
- **Summary**: Build Regression Fixes.
- **Details**: Resolved ESM export issues in `ui.js` and ensured all plugin managers are correctly exported for `esbuild`.
- **Summary**: CI/CD Pipeline Stability.
- **Details**: Standardized use of the `wheels/` directory in GitHub Actions and modernized the frontend build step.

## [0.5.0] - 2026-04-09

### 🚀 Core Improvements / Features
- **Summary**: Modular Testing Infrastructure.
- **Details**: Established a comprehensive `tests/` suite with `pytest` integration to ensure project stability and API reliability.

### 🛠 Build & Workflow Optimizations
- **Summary**: Modern ESM Frontend Bundling.
- **Details**: Refactored the frontend build from manual file concatenation to a modern ESM-based bundling process using `esbuild` and an `entry.js` point.
- **Summary**: Global Project Structure Migration.
- **Details**: Reorganized helper scripts into the `helpers/` directory and synchronized all relative paths across the build pipeline.

### 🎯 UI & API Enhancements
- **Summary**: New Performance Stress Test Suite.
- **Details**: Added `benchmarks/stream_bench.py` capable of stress-testing the engine with high-volume data pushes (19k candles).

### ⚙ Internal Refactoring
- **Summary**: Enhanced Frontend Plugin Architecture.
- **Details**: Reorganized the frontend plugin structure by moving `bandPlugin.js` to `js/plugins/` for better maintainability.
- **Summary**: Project Governance & Compliance.
- **Details**: Added MIT License, `CONTRIBUTING.md`, and `ruff` linting to establish professional code standards.

## [0.4.1] - 2026-04-09

### 🛠 Build & Workflow Optimizations
- **Summary**: Automated Git Tag Maintenance.
- **Details**: Implemented logic in `zz_upload_git.sh` to automatically prune older tags, keeping only the 3 most recent versions to prevent repository bloat.

## [0.4.0] - 2026-04-09

### 🚀 Core Improvements / Features
- **Summary**: Added native support for `manylinux` platform tags.
- **Details**: Resolved PyPI `linux_x86_64` tag rejection by implementing `manylinux_2_39` (local) and `auto` (CI) compatibility modes in `maturin`.

### 🛠 Build & Workflow Optimizations
- **Summary**: Global Version Synchronization.
- **Details**: Synchronized version `0.4.0` across `pyproject.toml`, `src/src-tauri/Cargo.toml`, and `create-wheels.sh`.
- **Summary**: Enhanced CI/CD for PyPI.
- **Details**: Updated GitHub Actions to automatically detect and apply the correct `manylinux` tag for binary distribution.


## [0.3.10] - 2026-04-08

### 🛠 Build & Workflow Optimizations
- **Summary**: Integrated Master Sync Script into Project Maintenance.
- **Details**: Established `zz_upload_git.sh` as the official single source of truth for project versioning in the `changelog_maintenance` documentation.

## [0.3.9] - 2026-04-08

### 🚀 Core Improvements / Features
- **Summary**: Implemented "Master Sync Engine" Script.
- **Details**: Created `zz_upload_git.sh` to centralize absolute version control and repository synchronization, including automated git stashing/rebasing.

### 🛠 Build & Workflow Optimizations
- **Summary**: Optimized GitHub Actions Triggers.
- **Details**: Eliminated redundant workflow runs by restricting CI/CD building to version tags only.
- **Summary**: Enhanced Workflow Robustness.
- **Details**: Added `git pull --rebase` to the automated wheel commit step in GHA to prevent "non-fast-forward" push rejections.

## [0.3.8] - 2026-04-08

### 🛠 Build & Workflow Optimizations
- **Summary**: Resolved GitHub Actions workflow validation errors.
- **Details**: Removed the `environment` block from `publish_pypi` which was causing the `Value 'pypi' is not valid` error due to OIDC mismatch.
- **Summary**: Global Version Synchronization.
- **Details**: Synchronized version `0.3.8` across `pyproject.toml`, `src/src-tauri/Cargo.toml`, and `create-wheels.sh`.

## [0.3.7] - 2026-04-08

### 🛠 Build & Workflow Optimizations
- **Summary**: Enhanced CI Observability and Package Metadata.
- **Details**: Finalized `pyproject.toml` metadata and improved pipeline transparency with structure inspections.

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
