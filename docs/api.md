# Chart Engine API Documentation (In-Depth)

This document provides a technical reference for the `chart_engine` Python API, detailing the high-performance Rust backend behaviors and data processing logic.

## Table of Contents
1. [Data Normalization & Scaling](#data-normalization--scaling)
2. [Chart Class](#chart-class)
3. [Series Class](#series-class)
4. [Advanced Drawing Logic](#advanced-drawing-logic)
5. [Trade Synchronization](#trade-synchronization)

---

## Data Normalization & Scaling

The Rust backend performs aggressive data normalization to ensure consistency across different data sources.

### Timestamp Processing
The engine uses heuristic-based auto-scaling for numeric timestamps. When a timestamp column (`time` or `date`) is provided:
- **Seconds**: Values < 1e10.
- **Milliseconds**: Values > 1e10 (e.g., JavaScript timestamps).
- **Microseconds**: Values > 1e12.
- **Nanoseconds**: Values > 1e15.
All values are normalized to **Unix seconds** internally.

### Column Sanitization
When passing a `polars.DataFrame`:
1. All column names are **lowercased**.
2. `date` is automatically aliased to `time` if `time` is missing.
3. Null values are filled with `0`.

---

## Chart Class
The `Chart` class is the primary interface, wrapping the Rust implementation.

### Configuration Methods

#### `set_timezone(tz: str)`
Configures the global timezone. Supports IANA timezone names (e.g., `"Asia/Tokyo"`, `"Europe/London"`). The backend uses `chrono-tz` for high-performance conversions.

#### `set_crosshair_mode(mode: int)`
- `0`: Normal (Free movement).
- `1`: Magnet (Snaps to OHLC values).

#### `set_sync(enabled: bool)`
Synchronizes crosshairs, scrolling, and zooming across all subcharts in a multi-pane layout.

---

## Series Class

### `update(item: dict | polars.DataFrame | polars.Series)`
Appends new data to the series.
- If a `dict` is passed, it is treated as a single bar.
- If a `DataFrame` is passed, only the **first row** is used.
- **Note**: The backend expects OHLC keys: `time`, `open`, `high`, `low`, `close` (and `value` for line series).

### `apply_options(options: dict)`
Supports Lightweight Charts (LWC) series options.
- **Line Series**: `color`, `lineWidth`, `lineStyle`, `lineType`.
- **Candlestick**: `upColor`, `downColor`, `borderVisible`, `wickVisible`.

---

## Advanced Drawing Logic

### Markers
#### `add_marker(series_id: str, time: Timestamp, **kwargs)`
Markers are used for entry/exit labels or signals.
- **Shapes**: `"circle"`, `"arrowUp"`, `"arrowDown"`, `"square"`.
- **Positions**: `"aboveBar"`, `"belowBar"`, `"inBar"`.

### Box Management & Categories
#### `create_box(start_time, start_price, end_time, end_price, category: str = None, **kwargs)`
The `category` parameter allows for automatic grouping:
- If a box with the **same category** is created, the system **automatically removes** previous boxes in that category on the same chart. This is ideal for showing "Live Zones" or "Current Targets" without manual cleanup.

### Horizontal Lines (`PriceLine`)
#### `create_horizontal_line(series_id: str, price: float, **kwargs)`
Creates a `PriceLine` object.
- **`update(price: float)`**: Dynamically moves the line. If `price=0.0`, the line is hidden from the UI.

---

## Trade Synchronization

### `sync_active_position(is_opened: bool, **kwargs)`
A specialized method for keeping the UI in sync with an external trade state (e.g., from a broker).

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `is_opened` | `bool` | - | Whether a position is currently open. |
| `start_time`| `Timestamp` | `None` | Entry time. |
| `entry_price`| `float` | `None` | Average entry price. |
| `sl_price` | `float` | `None` | Stop Loss level. |
| `tp_price` | `float` | `None` | Take Profit level. |
| `pos_type` | `str` | `None` | `"buy"` or `"sell"`. |

**Logic**:
- Informs the engine to render or clear the position tool.
- Automatically handles the conversion of `"buy"`/`"sell"` to `"long"`/`"short"`.
- Prevents redundant commands if the state hasn't changed.
