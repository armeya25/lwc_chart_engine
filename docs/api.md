# Chart Engine API Documentation (In-Depth)

This document provides a technical reference for the `chart_engine` Python API, detailing the high-performance Rust backend behaviors and data processing logic.

## Table of Contents
1. [Data Normalization & Scaling](#data-normalization--scaling)
2. [Chart Class](#chart-class)
3. [Series Class](#series-class)
4. [Advanced Drawing Logic](#advanced-drawing-logic)
5. [Trade Synchronization](#trade-synchronization)
6. [Paper Trading Methods](#paper-trading-methods)

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
The `Chart` class is the primary interface, wrapping the Rust implementation. It is a singleton; multiple instantiations will return the same instance.

#### `Chart(title: str = "Chart Window", width: int = 1200, height: int = 800, main_series_id: str = "main")`
Initializes the chart window and launches the Tauri backend.
- `title`: Window title.
- `width`/`height`: Initial window dimensions.
- `main_series_id`: ID of the default main series.

### Configuration Methods

#### `set_on_trade(callback: callable)`
Sets a custom callback function for trade events. The callback receives a `dict` containing trade details.

#### `positions` (Property)
Returns the current list of active positions as a list of `Position` objects.

#### `last_price` (Property)
Returns the last known market price updated via `trader_update_price`.

#### `history` (Property)
Returns the list of closed trade positions from the simulated engine as a list of dictionaries.

#### `set_timezone(tz: str)`
Configures the global timezone. Supports IANA timezone names (e.g., `"Asia/Tokyo"`, `"Europe/London"`).

#### `set_crosshair_mode(mode: int)`
- `0`: Normal (Free movement).
- `1`: Magnet (Snaps to OHLC values).

#### `set_sync(enabled: bool)`
Synchronizes crosshairs, scrolling, and zooming across all subcharts in a multi-pane layout.

#### `set_layout(layout: str = "single") -> List['SubChart']`
Configures the chart layout and returns a list of subchart interfaces.
- `layout`: `"single"`, `"double"`, `"1p2"` (1 top, 2 bottom), `"1p3"` (1 top, 3 bottom).

#### `set_tooltip(enabled: bool)`
Show or hide the floating tooltip that follows the crosshair. Use `enable_tooltip()` or `disable_tooltip()` for convenience.

#### `set_layout_toolbar_visibility(visible: bool)`
Show or hide the side toolbar containing layout settings. Use `enable_layout_toolbar()` or `disable_layout_toolbar()` for convenience.

#### `show_notification(message: str, type: str = "info")`
Displays a toast notification in the UI. Types: `"info"`, `"success"`, `"warning"`, `"error"`.

#### `set_trend_info_visibility(visible: bool)`
Show or hide the trend information overlay.

#### `set_watermark(text: str, chart_id: str = "chart-0")`
Sets a background watermark text for a specific chart.

#### `set_legend_visibility(visible: bool)`
Show or hide the series legend in the top-left corner.

#### `set_timeframe(tf: Union[str, Dict[str, Any]])`
Sets the timeframe indicator in the UI.

#### `update_trend_info(title: str = None, value: str = None, change: str = None, color: str = None)`
Updates the trend information overlay. Accepts arbitrary keys.

#### `take_screenshot(filename: str = None, chart_id: str = "chart-0")`
Triggers a screenshot of the specified chart. If `filename` is provided, the image is saved to the project root.

#### `set_price_margins(top: float = 0.05, bottom: float = 0.05, chart_id: str = "chart-0")`
Adjusts the vertical padding between the data (candles/lines) and the chart boundaries.
- `top`/`bottom`: Margin as a percentage of chart height (e.g. `0.05` for 5%).

#### `show(block: bool = True)`
Starts the UI event loop. If `block=True`, the Python script will wait until the window is closed.

#### `exit()`
Terminates the backend process and closes the chart window.

#### `remove_series(series_id: str, chart_id: str = "chart-0")`
Removes a specific series from a chart pane.

#### `remove_indicator(indicator_id: str)`
Programmatically removes an indicator group (e.g., "macd") and all its associated sub-series (signal lines, histograms, etc.) from the chart state.

#### `clear_all_series(chart_id: str = "chart-0")`
Removes all series from a specific chart pane.

### Series Management

#### `create_line_series(name: str = "Line", chart_id: str = "chart-0") -> 'Series'`
Creates a new line series on the specified chart pane.

#### `create_candlestick_series(name: str = "Price", chart_id: str = "chart-0") -> 'Series'`
Creates a new candlestick series on the specified chart pane.


---

## Paper Trading Methods
These methods are built-in to the `Chart` class and allow for real-time simulated trading.

#### `trader_execute(side: str, qty: float, price: float = None, tp: float = None, sl: float = None, series: Series = None, time: Any = None)`
Programmatically executes a trade.
- `side`: `"buy"` or `"sell"`.
- `series`: If provided, also places a visual marker on the chart at the trade price.
- `time`: Optional timestamp. If `None`, uses current local time or last data time.

#### `trader_update_price(price: float)`
Updates the last known market price. Automatically calculates P&L for all open positions and checks if any Take Profit (TP) or Stop Loss (SL) levels have been hit.

#### `update_positions(positions_data: list)`
Synchronizes the current internal list of positions with the UI's position table.

#### `trader_handle_callback(data: dict)`
Internal callback for trade events triggered from the UI.

#### `trader_close_position(side: str, qty: float, entry: float)`
Manually closes a position in the simulated engine.

---

## Utility Methods

#### `show_notification(message: str, type: str = "info")`
Displays a toast notification in the UI. Types: `"info"`, `"success"`, `"warning"`, `"error"`.

#### `show(block: bool = True)`
Starts the UI event loop. If `block=True`, the Python script will wait until the window is closed.

#### `exit()`
Terminates the backend process and closes the chart window.

## Position Object
Returned as a list via the `Chart.positions` property.

| Property | Type | Description |
|----------|------|-------------|
| `side`   | `str` | `"buy"` or `"sell"`. |
| `qty`    | `float` | Position quantity. |
| `entry`  | `float` | Average entry price. |
| `price`  | `float` | Last known market price. |
| `tp`     | `float` | Take Profit level (or `None`). |
| `sl`     | `float` | Stop Loss level (or `None`). |
| `pnl`    | `float` | Current unrealized Profit & Loss. |

---

## SubChart Class
Returned by `set_layout`. Provides a restricted interface for creating series on a specific pane.

#### `create_line_series(name="Line")`
#### `create_candlestick_series(name="Price")`

---

## Series Class

### `set_data(df: pl.DataFrame)`
Sets the entire dataset for the series. Overwrites existing data.

### `update(item: Union[pl.DataFrame, Any]) -> List[str]`
Appends new data to the series. Returns the sync commands generated by the backend.

### `apply_options(options: Dict[str, Any])`
Supports Lightweight Charts (LWC) series options.
- **Line Series**: `color`, `lineWidth`, `lineStyle`, `lineType`.
- **Candlestick**: `upColor`, `downColor`, `borderVisible`, `wickVisible`.

#### `set_auto_volume(enabled: bool)`
Enable or disable the automatic creation of a volume histogram pane for this series.

### Indicator Methods (v0.9.8)
All indicators are high-performance and calculated in the Rust backend using Polars. Indicators are logically grouped into categories.

#### Trend & Overlays
#### `add_sma(period: int = 14, color: str = "#2962FF") -> 'Series'`
Simple Moving Average.
#### `add_ema(period: int = 14, color: str = "#FF9800") -> 'Series'`
Exponential Moving Average.
#### `add_wma(period: int = 14, color: str = "#2196F3") -> 'Series'`
Weighted Moving Average.
#### `add_hma(period: int = 14, color: str = "#4CAF50") -> 'Series'`
Hull Moving Average.
#### `add_dema(period: int = 14, color: str = "#FF5252") -> 'Series'`
Double Exponential Moving Average.
#### `add_tema(period: int = 14, color: str = "#00BCD4") -> 'Series'`
Triple Exponential Moving Average.

#### Oscillators (Dedicated Sub-Panes)
#### `add_rsi(period: int = 14, color: str = "#9C27B0") -> 'Series'`
Relative Strength Index.
#### `add_macd(fast: int = 12, slow: int = 26, signal: int = 9) -> 'Series'`
Moving Average Convergence Divergence.
#### `add_stochastic(period: int = 14, smooth_k: int = 3, smooth_d: int = 3) -> 'Series'`
Stochastic Oscillator.
#### `add_cci(period: int = 14, color: str = None) -> 'Series'`
Commodity Channel Index.
#### `add_mfi(period: int = 14, color: str = "#FFC107") -> 'Series'`
Money Flow Index.
#### `add_roc(period: int = 14, color: str = "#FF1744") -> 'Series'`
Rate of Change.
#### `add_williams_r(period: int = 14, color: str = None) -> 'Series'`
Williams %R.

#### Volatility
#### `add_bollinger_bands(period: int = 14, std_dev: float = 2.0, color: str = None) -> 'Series'`
Bollinger Bands (Upper, Middle, Lower).
#### `add_atr(period: int = 14, color: str = "#FF5252") -> 'Series'`
Average True Range.
#### `add_keltner_channels(period: int = 20, multiplier: float = 2.0) -> 'Series'`
Keltner Channels.
#### `add_donchian_channels(period: int = 20) -> 'Series'`
Donchian Channels.

#### Volume
#### `add_vwap(color: str = None) -> 'Series'`
Volume-Weighted Average Price.
#### `add_obv(color: str = None) -> 'Series'`
On-Balance Volume.
#### `add_adl(color: str = None) -> 'Series'`
Accumulation/Distribution Line.

#### `add_segmented_line(df: pl.DataFrame, width: int = 2)`
Converts the series data into a Segmented Line, allowing for dynamic multi-color support within a single line. Requires a DataFrame with `time`, `value`, and `color` columns.

#### `add_band(df: pl.DataFrame, color: str = "rgba(31, 150, 243, 0.2)")`
Adds a technical Band (Cloud) between two series. Requires a DataFrame with `time`, `top`, and `bottom` columns.

#### `add_indicator_v2(ind_type: str, params: Dict[str, Any] = None, metadata: Dict[str, Any] = None) -> 'Series'`
Unified optimized call to add an indicator with minimal context switching. Returns the primary series.

### `add_marker(time: Any, position: str = "aboveBar", color: str = "#2196F3", shape: str = "arrowDown", text: str = "")`
Convenience method for adding a marker to this specific series.

---

## Advanced Drawing Logic

### Markers
#### `add_marker(series_id: str, time: Timestamp, position: str = "aboveBar", color: str = "#2196F3", shape: str = "arrowDown", text: str = "", chart_id: str = "chart-0")`
Markers are used for entry/exit labels or signals.
- **Shapes**: `"circle"`, `"arrowUp"`, `"arrowDown"`, `"square"`.
- **Positions**: `"aboveBar"`, `"belowBar"`, `"inBar"`.

### Box Management & Categories
#### `create_box(start_time, start_price, end_time, end_price, color: str = "rgba(33, 150, 243, 0.2)", border_color: str = "#2196F3", text: str = "", category: str = None, chart_id: str = "chart-0")`
The `category` parameter allows for automatic grouping:
- If a box with the **same category** is created, the system **automatically removes** previous boxes in that category on the same chart. This is ideal for showing "Live Zones" or "Current Targets" without manual cleanup.

#### `remove_box(box_id: str)`
Removes a specific box by ID.

### Horizontal Lines (`PriceLine`)
#### `create_horizontal_line(series_id: str, price: float, color: str = "#F44336", chart_id: str = "chart-0")`
Creates a `PriceLine` object.
- **`update(price: float)`**: Dynamically moves the line. If `price=0.0`, the line is hidden from the UI.

### Line Tools
#### `create_trendline(st, sp, et, ep, color: str = "#2196F3", width: int = 1, style: int = 0, visible: bool = True, text: str = "", chart_id: str = "chart-0")`
#### `create_ray(st, sp, et, ep, color: str = "#2196F3", width: int = 1, style: int = 0, visible: bool = True, text: str = "", chart_id: str = "chart-0")`
#### `create_fib_retracement(st, sp, et, ep, color: str = "#2196F3", width: int = 1, style: int = 0, visible: bool = True, text: str = "", chart_id: str = "chart-0")`
General purpose line drawing tools. `st`/`et` are timestamps, `sp`/`ep` are prices. `style` 0=Solid, 1=Dashed, 2=Dotted.

#### `remove_line_tool(tool_id: str)`
#### `clear_line_tools()`
Removes all line drawing tools.

### Position Tools
#### `create_long_position(st, ep, sl, tp, end_time=None, visible=True, quantity=1.0, chart_id="chart-0")`
#### `create_short_position(st, ep, sl, tp, end_time=None, visible=True, quantity=1.0, chart_id="chart-0")`
Visualizes a trade position with entry, SL, and TP levels.
- `st`: Start Time.
- `ep`: Entry Price.
- `sl`: Stop Loss Price.
- `tp`: Take Profit Price.

#### `remove_position(pos_id: str)`
#### `clear_positions(chart_id: str = None)`
Removes all position tools (optionally restricted to a specific chart).

---

## Trade Synchronization

### `sync_active_position(is_opened: bool, start_time: Timestamp = None, entry_price: float = None, sl_price: float = None, tp_price: float = None, pos_type: str = None, end_time: Timestamp = None, chart_id: str = "chart-0")`
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

