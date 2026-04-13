
## [0.9.8] - 2026-04-13

### 🚀 Core Improvements / Features
- **Summary**: Precision Chart Framing.
- **Details**: Implemented dynamic price scale margin control via the Python API (`set_price_margins`), allowing users to eliminate large visual gaps or increase padding for professional layouts.

### 🛠 Build & Workflow Optimizations
- **Summary**: Mixed Project Packaging Fix.
- **Details**: Resolved a critical build flaw where Maturin would default to a pure-Rust wheel structure. Synchronized the build pipeline to correctly discover and bundle the `chart_engine` Python package alongside the native Rust components.
- **Summary**: CI/CD Verification Layer.
- **Details**: Integrated an automated audit step into GitHub Actions to verify wheel structure (presence of Python source and Rust extension) before publication.

### 🎯 UI & API Enhancements
- **Summary**: Dynamic Margin API.
- **Details**: Exposed `chart.set_price_margins(top, bottom)` to allow programmatic vertical padding adjustments on the fly.

## [0.9.7] - 2026-04-13

### 🚀 Core Improvements / Features
- **Summary**: Comprehensive Technical Indicator Expansion.
- **Details**: Completed the integration of all 20 technical indicators defined in the registry. Added support for **WMA, HMA, Keltner Channels, Donchian Channels, MFI, ROC, OBV, and ADL**.
- **Summary**: Robust Indicator Calculations.
- **Details**: Hardened range-based indicators (ADL, Stochastic, Williams %R, CCI) against division-by-zero errors in the Rust backend, ensuring stable rendering even during flat price ranges.

### 🎯 UI & API Enhancements
- **Summary**: Centered Trade Execution Panel.
- **Details**: Repositioned the execution panel to be centered horizontally at the top of the chart area for improved accessibility.
- **Summary**: Clean Legend UX.
- **Details**: Hidden vertical scrollbars in the indicator legend to maintain a clean aesthetic while preserving scroll functionality for dense indicator lists.

### ⚙ Internal Refactoring
- **Summary**: Modular Indicator Dispatcher.
- **Details**: Fully refactored the monolithic `add_indicator_v2` method into specialized, maintainable helpers for parsing, naming, and color resolution.
- **Summary**: Circular Dependency Fix.
- **Details**: Resolved a critical circular import between `chart.py` and `indicators.py` in the Python package using late-binding imports.

## [0.9.6] - 2026-04-12

### 🚀 Core Improvements / Features
- **Summary**: Modular Indicator Engine & 10 New Indicators.
- **Details**: Completed a massive overhaul of the technical indicator system. Refactored the monolithic logic into a structured `indicators/` backend and added 10 new high-performance indicators: **DEMA, TEMA, WMA, HMA, MFI, ROC, Keltner Channels, Donchian Channels, OBV, and ADL**.

### 🛠 Build & Workflow Optimizations
- **Summary**: Polars v0.48.1 Stabilization.
- **Details**: Successfully resolved multiple breaking API changes introduced in Polars v0.48.1, including `EWMOptions` field requirements, `LazyFrame::collect()` closure type ambiguity, and horizontal aggregation conflicts.
- **Summary**: Robust DrawingTool Visibility.
- **Details**: Decoupled core `DrawingTool` interaction methods from the `python-bridge` feature, ensuring internal engine components remain functional in all build configurations.

### ⚙ Internal Refactoring
- **Summary**: Backend Modularization.
- **Details**: Migrated `indicators.rs` to a dedicated `indicators/` directory with logic split across domain-specific modules (`oscillators.rs`, `volatility.rs`, `volume.rs`).
- **Summary**: Unified Indicator Registry.
- **Details**: Consolidated indicator metadata, default parameters, and mapping logic into a clean `registry.rs` architecture using embedded `.toml` configuration.

## [0.9.5] - 2026-04-12

### 🚀 Core Improvements / Features
- **Summary**: Group-Synced Indicator Visibility.
- **Details**: Redesigned the indicator visibility engine to ensure that hiding a multi-line indicator (like MACD or Bollinger Bands) correctly toggles all sub-series and legend items.
- **Summary**: Cross-Type Indicator Color Rotation.
- **Details**: Implemented a global color rotation system in the Rust backend. Overlay indicators (SMA, TEMA, DEMA) added sequentially now automatically receive distinct colors from the high-contrast palette.

### 🎯 UI & API Enhancements
- **Summary**: Premium Info Panel (v1.0 Refactor).
- **Details**: Rebranded and refactored the legacy Trend Panel into a consolidated "Info Panel". Added glassmorphic auto-expansion on data arrival and intelligent sentiment color-coding (Bullish/Bearish/Neutral).
- **Summary**: Unified Indicator Legend.
- **Details**: Enhanced the legend to always include the primary series in the sub-item list, providing a 1:1 visual mapping for all indicator components.

### ⚙ Internal Refactoring
- **Summary**: Python API Alignment.
- **Details**: Renamed `chart.py` methods to `set_info_panel_visibility` and `update_info_panel` to match the new UI branding. Fixed race conditions in the legend item creation sequence.

## [0.9.3] - 2026-04-12

### 🚀 Core Improvements / Features
- **Summary**: Integrated Indicator Search Engine.
- **Details**: Implemented a titlebar-integrated search box that enables real-time discovery of technical indicators. Supported both standard text filtering and Regular Expression (Regex) matching (e.g., `^SMA`, `RSI|MACD`).
- **Summary**: Rust-Native Position Coordination.
- **Details**: Migrated the 1:1 synchronization between visual position tools (SL/TP lines) and the paper trading logic entirely into the Rust core, eliminating execution lag and improving architectural consistency.

### 🛠 Build & Workflow Optimizations
- **Summary**: Structured AI Project Skills.
- **Details**: Overhauled project guidance in `.agent/skills/` to prioritize Rust implementations and ensure consistent environment management across all AI sessions.
- **Summary**: Automated Production Sync.
- **Details**: Optimized the frontend build pipeline to ensure that all UI enhancements are automatically propagated to both development and production templates.

### 🎯 UI & API Enhancements
- **Summary**: UI Shortcut De-cluttering.
- **Details**: Removed all legacy keyboard shortcuts (Legend, Execution, Positions, Trend Info) and their visual hints to streamline the interface and prevent accidental triggers.
- **Summary**: Dynamic Indicator Metadata Synchronization.
- **Details**: The backend now transmits the full schema of available indicators to the UI on startup, ensuring the search box is always synchronized with the engine.

### ⚙ Internal Refactoring
- **Summary**: Unified Action Routing.
- **Details**: Fixed initialization race conditions in `entry.js` and added missing PyO3 bindings for indicator schemas, ensuring robust communication between the UI and Rust backend.

## [0.9.2] - 2026-04-11

### 🚀 Core Improvements / Features
- **Summary**: Modular CSS Architecture.
- **Details**: Refactored the monolithic `terminal.css` into a structure-aligned multi-file system (`css/` directory) for significantly improved maintainability. Implemented an `@import` manifest pattern to maintain zero-impact integration for existing frontend assets.

### 🛠 Build & Workflow Optimizations
- **Summary**: Robust CSS Bundling & Distribution.
- **Details**: Added the `--bundle` flag to the `esbuild` configuration in `build-frontend.js`. This resolves a critical build regression where modular CSS dependencies were being excluded from the production `dist/` directory.
- **Summary**: OS-Aware Cross-Platform Builds.
- **Details**: Updated the GitHub Actions workflow and local build scripts to conditionally apply the `--compatibility` flag only on Linux platforms, ensuring stable wheel generation for macOS and Windows.
- **Summary**: Automated Build Cleanup.
- **Details**: Enhanced the repository hygiene job in CI/CD to automatically purge the `dist/` directory after a successful PyPI release.

### 🎯 UI & API Enhancements
- **Summary**: Professional Positions & Portfolio Dashboard.
- **Details**: Modernized the Positions Panel with a high-fidelity glassmorphic tabbed interface (Active vs. History). Implemented monospace financial data alignment and optimized typography for professional-grade legibility.
- **Summary**: Custom Legend Series Filtering.
- **Details**: Refined the Candlestick series legend to remove distracting close/settings buttons while increasing the default visibility of interactive controls for all other indicators to improve UX discoverability.

### ⚙ Internal Refactoring
- **Summary**: Unified Project Metadata Synchronization.
- **Details**: Synchronized a global project version bump to `0.9.2` across `pyproject.toml`, `Cargo.toml`, and `tauri.conf.json`.

## [0.9.1] - 2026-04-11

### 🚀 Core Improvements & Stability
- **Summary**: Scoped Pane Rebalancing.
- **Details**: Improved the visibility logic to independently scope pane resizing per chart. This ensures that hiding an indicator on one chart correctly reclaims vertical space without shifting the layout of other active charts.
- **Summary**: Robust Command Handlers.
- **Details**: Refactored `remove_indicator` and `open_indicator_settings` to correctly process both single-argument UI calls and structured backend command payloads, resolving unresponsive control buttons.

### 🎯 UI & API Enhancements
- **Summary**: Legend Stability & Crash Prevention.
- **Details**: Fixed a critical `ReferenceError` in the indicator legend initialization that caused crashes when adding multi-series indicators (e.g., Bollinger Bands).
- **Summary**: Settings Modal Restoration.
- **Details**: Resolved the "Empty Modal" bug by removing redundant metadata overwrites in the Python bridge and repairing the Rust schema export function.

### ⚙ Internal Refactoring
- **Summary**: Expanded Indicator Label Mapping.
- **Details**: Added logic-aware label matching for `bollingerbands` and `stochastic` in the Rust core for cleaner UI display.
- **Summary**: Rust-Native Automatic Color Rotation.
- **Details**: Implemented a stateful color rotation system in the Rust backend with an expanded 40-color palette. Multiple indicators of the same type added to a series now automatically receive distinct, high-contrast colors without manual user configuration.

### 🛠 Build & Workflow Optimizations
- **Summary**: Automated Repository Hygiene.
- **Details**: Integrated a `cleanup_repo` job into the GitHub Actions pipeline that automatically removes binary wheels from the repository history after successful publication to PyPI, preventing repository bloat.
- **Summary**: CI/CD Directory Synchronization.
- **Details**: Synchronized the CI/CD artifact directory naming to `wheels/` (from `dist/`) to align with local build helper script conventions.

## [0.9.0] - 2026-04-11

### 🚀 Core Improvements / Features
- **Summary**: Scoped Auto-Resizing Indicator Panes.
- **Details**: Implemented dynamic vertical space redistribution when indicators are hidden or removed. The logic is now independently scoped per chart, ensuring that hiding an indicator on one pane correctly reclaims space without interfering with other open charts.
- **Summary**: High-Performance MACD Integration.
- **Details**: Finalized the triple-series MACD implementation with optimized Rust-side calculations and real-time histogram coloring.

### 🎯 UI & API Enhancements
- **Summary**: Professional Legend Design & Legibility.
- **Details**: Flattened the indicator legend into high-density rows. Resolved name clipping (ellipses) by expanding CSS constraints and mapping technical codes to full titles (e.g., 'Bollinger Bands').
- **Summary**: Fully Functional Indicator Settings.
- **Details**: Restored the parameter modification modal by correcting property access on the Rust-to-JS metadata bridge. Users can now view and edit indicator periods and colors seamlessly.
- **Summary**: PEP 484 Type Hinting & API Documentation.
- **Details**: Applied full type annotations to `chart.py` and synchronized `docs/api.md` with all v0.9.0 programmatic methods (`history`, `add_indicator_v2`, `auto_resize`).

### ⚙ Internal Refactoring
- **Summary**: State-Synced Visibility Logic.
- **Details**: Resolved series ID shadowing in `commands.js` to ensure 1:1 state mapping between the Python backend and JS frontend during indicator toggles.
- **Summary**: Diagnostic Cleanup.
- **Details**: Purged development logs and minimized binary overhead for the production release.

## [0.8.5] - 2026-04-11

### 🚀 Core Improvements / Features
- **Summary**: Fully Synchronized Indicator Removal.
- **Details**: Migrated removal logic to the Rust backend, returning a precise list of removed IDs to ensure Python/Rust state synchronization.
- **Summary**: Robust Indicator Management.
- **Details**: Refactored frontend command handlers (`remove_indicator`, `update_indicator`) to be signature-agnostic, resolving a critical bug where legend controls (Close, Save Settings) were non-functional when triggered from the UI.
- **Summary**: Metadata-Robust Data Bridge.
- **Details**: Enhanced data commands to carry indicator metadata, ensuring legend functionality is preserved during high-frequency streaming.

### 🎯 UI & API Enhancements
- **Summary**: Visual Real-Estate Optimization.
- **Details**: Moved the Trend Info panel higher (`top: 40px`) to align with the legend and maximize available vertical space for chart rendering.
- **Summary**: Parameter-Aware Naming & Legibility.
- **Details**: Fixed indicator "naming gibberish" by ensuring labels consistently reflect human-readable, parameter-augmented names (e.g., `SMA(14)`).
- **Summary**: Expanded Indicator Test Suite.
- **Details**: Updated `indicator_test.py` with comprehensive examples for **RSI** and **Bollinger Bands**, verifying multi-pane and multi-series integration.
- **Summary**: Default UI Persistence.
- **Details**: Enabled Legend and Trend Info panels by default for a smoother first-run experience.

### ⚙ Internal Refactoring
- **Summary**: Secure Serialization Path.
- **Details**: Implemented explicit type casting (Int64/Float64) in the Polars-to-JSON bridge, preventing backend schema mismatches during indicator calculations.
- **Summary**: Clean Build Standards.
- **Details**: Resolved numerous Rust compilation warnings and standardized background process communication.

## [0.7.9] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Smart Indicator Naming.
- **Details**: Implemented parameter-aware labels in the legend (e.g., `SMA(14)`), resolving generic naming redundancies and providing better context at a glance.
- **Summary**: Backend-Driven Naming Sanitization.
- **Details**: Fixed "naming gibberish" by stripping JSON quotes from indicator types and parameters directly in the Rust core, ensuring a 100% human-readable legend.

### 🎯 UI & API Enhancements
- **Summary**: Streamlined Title Bar Navigation.
- **Details**: Overhauled the navigation system by replacing the horizontal layout banner and floating vertical toolbars with a clean, nested **Layouts** submenu integrated into the window's **View** menu.
- **Summary**: Positions Panel Modernization.
- **Details**: Refactored tab switching (Active vs History) from legacy onclick handlers to robust event listeners. Added a `max-height: 240px` scroll limit and integrated active-state notifications for a smoother dashboard experience.
- **Summary**: Production Build Synchronization.
- **Details**: Synchronized `index.dist.html` with `index.html` to ensure that custom Title Bar controls, glassmorphism menus, and all new UI refinements are present in the final bundled application.

### ⚙ Internal Refactoring
- **Summary**: Event-Driven UI Initialization.
- **Details**: Centralized UI component setup (Positions panel, Layout submenus) in `ui.js` and `entry.js` for improved reliability during fast application startup sequences.

## [0.7.8] - 2026-04-10

### ⚙ Internal Refactoring
- **Summary**: Development branch for Title Bar restoration and layout synchronization.

## [0.7.7] - 2026-04-10

### ⚙ Internal Refactoring
- **Summary**: Build script syntax fixes and tooling standardization (uv/.venv).

## [0.7.6] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: State-Synced Indicator Engine.
- **Details**: Resolved a critical issue where technical indicators would fail to appear after dynamic registration by shifting to a chart-level state synchronization model. 
- **Summary**: Logic-Driven Indicator Metadata.
- **Details**: Implemented `get_indicator_params_schema` in the Rust core to provide full parameter definitions (min/max/default) to the frontend during series creation.

### 🎯 UI & API Enhancements
- **Summary**: Premium Performance UI.
- **Details**: Implemented a bespoke glassmorphism title bar with functional window controls (Minimize, Maximize, Close) and a dynamic **View** menu for layout management.
- **Summary**: Smart Legend Redesign.
- **Details**: Overhauled the legend with a dynamic layout system. Single-component indicators (SMA/EMA) use elegant single rows, while multi-component indicators (MACD/Supertrend) use structured, toggleable folders.
- **Summary**: Legibility & Interaction Fixes.
- **Details**: Replaced internal Suffix IDs (gibberish strings) with human-readable labels (e.g., SMA, MACD). Fixed visibility and functional triggers for indicator Gear (settings) and Close buttons.
- **Summary**: Native Histogram Rendering.
- **Details**: Added `create_histogram_series` to the Rust core for professional MACD histogram visuals.

### ⚙ Internal Refactoring
- **Summary**: Build Log & Hotkey Management.
- **Details**: Cleaned up compiler warnings (redundant clones) and implemented global hotkeys (`T`, `L`, `E`, `P`, `I`) for panel management.

## [0.7.5] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Rust-Native Indicator Schemas & Orchestration.
- **Details**: Migrated indicator parameter definitions (ranges, defaults, types) centrally to the Rust core. Introduced `add_indicator_v2`, a unified orchestration call that handles sub-series creation, registration, and data calculation in a single Python-to-Rust context switch, drastically reducing API "chattiness" and improving performance.

### 🎯 UI & API Enhancements
- **Summary**: Premium Indicator Settings Modal.
- **Details**: Built a premium glassmorphism settings interface that dynamically generates inputs based on the Rust-provided metadata. Added full "Cancel" support to ensure system robustness when users abort parameter changes.
- **Summary**: Color-Coded Volume & Real-time Sync.
- **Details**: Overhauled the volume histogram system to support automated Up/Down (Green/Red) color-coding. Fixed a bug where volume bars were missing and implemented real-time, stateful volume updates to match price action ticks.

### ⚙ Internal Refactoring
- **Summary**: Safe Circular Dependency Management.
- **Details**: Refactored `indicators.py` with late-binding imports to safely manage the relationship between the `Series` mixin and the main `Chart` engine.


## [0.7.0] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Live Indicator Values in Legend.
- **Details**: Rewrote the crosshair-to-legend value pipeline in `ui.js`. The engine now performs a two-step lookup: first an exact match via `param.seriesData`, then a nearest-bar fallback via `series.dataByIndex(logicalIndex, -1)`. This ensures SMA, MACD, RSI, and all other indicator series always display live values while hovering — exactly like TradingView's legend behaviour.

### 🎯 UI & API Enhancements
- **Summary**: Independent Legend Panel Positioning.
- **Details**: Decoupled the `#legend` component from the `#info-panel` container (which holds Trend and Positions). The legend is now a standalone `position: fixed` panel anchored to the **top-left**, while Trend Info remains in the **top-right** via `#info-panel`. Both panels collapse/expand independently and share the same glassmorphism aesthetic.
- **Summary**: Legend Scroll & Size Constraint.
- **Details**: Added `max-height: 340px` and `overflow-y: auto` on `#legend-content` to prevent the legend from growing to full-screen when many series are registered.

### ⚙ Internal Refactoring
- **Summary**: Frontend Path Fix for Build Script.
- **Details**: Corrected the frontend build invocation to run `node build-frontend.js` from the `src/` subdirectory, matching the relative paths used by `esbuild` for `src-frontend/js/entry.js`.

## [0.6.5] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Stateful O(1) Real-time Scaling.
- **Details**: Refactored the technical indicator engine to persist internal states (EMA averages, MACD signal lines). This enables the engine to process incremental market ticks in constant time, $O(1)$, regardless of historical data depth, ensuring stable high-frequency rendering for SMA, EMA, RSI, and MACD.
- **Summary**: Zero-Copy Polars Bridge.
- **Details**: Optimized the Python-to-Rust data bridge to utilize zero-copy memory mapping for Polars DataFrames via Apache Arrow. This eliminates the serialization bottleneck and provides near-instantaneous ingestion of large datasets.

### 🎯 UI & API Enhancements
- **Summary**: Constrained Legend Layout.
- **Details**: Resolved a critical UI bug where the legend component would unexpectedly expand to fill the full screen. The legend has been moved into the `#info-panel` container, inheriting proper positioning constraints and premium glassmorphism styling.
- **Summary**: High-Precision Marker Snap.
- **Details**: Standardized the market data normalization layer to ensure high-precision floating point alignment for markers and drawing tools across all ingestion paths.

### 🛠 Build & Workflow Optimizations
- **Summary**: Robust Backend Schema Mapping.
- **Details**: Implemented a comprehensive type-normalization layer in `src-backend/chart.rs` and `time_utils.rs` that automatically casts heterogeneous numeric inputs (e.g., integer volumes or price ticks) to `Float64`, preventing backend `SchemaMismatch` panics.

## [0.6.4] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Rust-Native Technical Indicator Engine.
- **Details**: Migrated all technical indicator calculations (SMA, EMA, RSI, MACD, Bollinger Bands) from Python to the Rust core using Polars. This significantly reduces data transfer overhead and leverages Rust's high-performance SIMD-optimized math.
- **Summary**: Real-time Incremental Calculation.
- **Details**: Implemented stateful calculation logic in Rust that handles incremental price updates (ticks) with $O(1)$ complexity, avoiding redundant batch re-calculations for indicators like EMA and MACD during live market updates.
- **Summary**: Multi-Series Indicator Synchronization.
- **Details**: MACD and Bollinger Bands now correctly spawn and synchronize multiple sub-series (MACD/Signal/Histogram, Upper/Middle/Lower bands) directly from the backend.

### 🎯 UI & API Enhancements
- **Summary**: Modular Indicator Mixin API.
- **Details**: Refactored `chart.py` to move all technical indicator methods into a separate `IndicatorMixin` module (`indicators.py`). This provides a cleaner architecture while maintaining the intuitive `series.add_indicator()` API.
- **Summary**: Format-Agnostic Data Ingestion (Pandas/Polars/JSON).
- **Details**: Refactored `set_data` and `update` to automatically detect and convert Pandas DataFrames, Polars DataFrames, and raw JSON (lists of dicts) into the internal format, providing a seamless experience for data scientists using different libraries.
- **Summary**: Segmented Band Plugin Support.
- **Details**: Added support for dynamic segmented bands (clouds with variable colors) through the `add_segmented_band` method.

### 🛠 Build & Workflow Optimizations
- **Summary**: Polars 0.48.1 Migration.
- **Details**: Fully upgraded the backend to Polars 0.48.1, resolving upstream API breaking changes in `EWMOptions` and `RollingOptionsFixedWindow`.
- **Summary**: Robust `maturin` Build & `cffi` Handling.
- **Details**: Optimized the build process to automatically resolve `cffi` dependencies in `uv` environments and ensured the `python-bridge` feature is correctly toggled during development.

## [0.6.3] - 2026-04-10

### 🛠 Build & Workflow Optimizations
- **Summary**: Full Robustness & Portability Build.
- **Details**: Switched to a "Full Robustness" build strategy by removing both `--strip` and `--auditwheel skip`. The resulting wheels (~100MB) are fully self-contained with all system dependencies bundled (GTK/WebKit) and include full debug symbols for line-numbered tracebacks.

### ⚙ Internal Refactoring
- **Summary**: Global Version Synchronization.
- **Details**: Unified project version `0.6.3` across `pyproject.toml`, `src/src-tauri/Cargo.toml`, and all build helper scripts.

## [0.6.2] - 2026-04-10

### 🚀 Core Improvements / Features
- **Summary**: Enhanced Native Crash Diagnostics.
- **Details**: Integrated `faulthandler` into the Python bridge to capture and report C-level stack traces during native crashes. Added a custom Rust panic hook to provide precise line-number reporting for backend failures.

### 🛠 Build & Workflow Optimizations
- **Summary**: Robust Binary Distribution & Debugging.
- **Details**: Removed `--auditwheel skip` and `--strip` flags from development and production build scripts to ensure dependency safety and preserve debug symbols.
- **Summary**: Binary Integrity Protection.
- **Details**: Disabled UPX compression for shared libraries (`.so` files) to prevent potential binary corruption and preserve traceback reliability.

### 🎯 UI & API Enhancements
- **Summary**: Improved Backend Observability.
- **Details**: Implemented real-time `stderr` mirroring from the Tauri background process directly to the Python console, surfacing hidden GTK/WebKit initialization errors.

### ⚙ Internal Refactoring
- **Summary**: Global Version Synchronization.
- **Details**: Unified project version `0.6.2` across `pyproject.toml`, `src/src-tauri/Cargo.toml`, and all build helper scripts.

## [0.6.1] - 2026-04-09

### 🛠 Build & Workflow Optimizations
- **Summary**: Enhanced Git synchronization and push workflow.
- **Details**: Improved `helpers/upload_to_git.sh` with dynamic version extraction from `pyproject.toml`, automatic remote synchronization (pull/rebase) before pushing, and removed potentially destructive force-push flags for improved repository safety.

### ⚙ Internal Refactoring
- **Summary**: Resolved repository synchronization corruption.
- **Details**: Fixed the "incorrect old value provided" Git error by rebuilding local tracking references and ensuring clean synchronization with `origin/main`.

## [0.6.0] - 2026-04-09

### 🛠 Build & Workflow Optimizations
- **Lightweight Linux Distribution**: Successfully reduced the Linux `manylinux` wheel size by **95%** (from 80MB+ to **4.3MB**) by skipping redundant library bundling and enabling binary stripping.
- **CI/CD Publishing Robustness**: Restored the `environment: pypi` block and added manual `workflow_dispatch` support, ensuring reliable OIDC-based publishing and easier release management.
- **Repository Hygiene**: Updated the automated build pipeline to explicitly clear old wheels from Git tracking, preventing binary accumulation in the `dist/` folder on GitHub.

### 🎯 Documentation & UI
- **Feature Showcase**: Enriched the `README.md` with new high-quality screenshots for Multi-Chart Layouts and Drawing Tools.
- **Runtime Dependency Guide**: Provided a comprehensive step-by-step guide for Linux users to install required runtime libraries, ensuring the new lightweight binaries run seamlessly.

## [0.5.10] - 2026-04-09

### 🛠 Build & Workflow Optimizations
- **Binary Size Optimization**: Optimized the Linux `manylinux` wheel by excluding heavy GUI shared libraries (WebKitGTK, GTK3) from the bundle. This reduces the wheel size from ~80MB to <10MB.
- **GHA Permission Restoration**: Updated the GitHub Actions pipeline to explicitly restore executable permissions (`chmod +x`) after wheel repacking, preventing `PermissionError` on Linux.
- **Enhanced High Compression**: Expanded the UPX step in CI/CD to include the main `chart_engine` binary, aligning automated builds with local optimization standards.

### 🎯 Bug Fixes
- **Linux Execution Failure**: Resolved a critical `Permission Denied` error (Errno 13) that occurred when starting the chart engine background process.

### ⚙ Internal Refactoring
- **Version Synchronization**: Unified project version to `v0.5.10` across all metadata, build scripts, and deployment configurations.

## [0.5.9] - 2026-04-09

### 🚀 Core Improvements & Stabilization
- **Automated Column Aliasing**: The Rust backend now transparently handles `date` or `datetime` columns by mapping them to the expected `time` index, enabling true "plug-and-play" with standard historical data.
- **Buffered Rendering & Cold-Starts**: Implemented a command retry queue in `commands.js` that buffers data instructions until the frontend is fully initialized, resolving the "blank chart" bug during fast application startups.
- **PaperTrader Hardening**: Fixed a critical argument mismatch in the Rust bridge, enabling programmatic trade execution with full 64-bit timestamp support.

### 🛠 Build & Workflow Optimizations
- **Cross-Platform Build System**: Introduced `src/build-frontend.js`, a Node.js-based orchestrator that replaces POSIX shell commands. This ensures stable builds across Windows and Linux environments, specifically targeting GitHub Actions runner compatibility.
- **Version Synchronization**: Standardized the project version to v0.5.9 across all configuration files and scripts.

### 🎯 UI & API Enhancements
- **Modernized SyncManager**: Overhauled the crosshair synchronization logic to use native series resolution. Crosshairs are now perfectly mirrored across multiple charts in multi-pane layouts without silent failures.
- **Robust Screenshot Engine**: Refactored the snapshot system to use a Base64 bridge. This bypasses browser-level download restrictions in Tauri and allows screenshots (with custom filenames) to be saved directly to the project root.
- **UI Interaction Scoping**: Globally exposed all toolbar and legend handlers to ensure compatibility with standard HTML event listeners in the bundled production application.

### ⚙ Internal Refactoring
- Cleaned up unused imports (`uuid`, `datetime`, `zoneinfo`) in `chart.py`.
- Optimized the high-compression wheel repacking process to use cross-platform paths.

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
