import subprocess
import os
import json
import polars as pl
from . import chart_engine_lib
import time
import threading
import logging
import atexit
import base64
import faulthandler

# Enable faulthandler to get stack traces on native crashes (SIGSEGV, etc.)
faulthandler.enable()

# State for backend timezone (synced with Rust)
_BACKEND_TZ = "UTC"

def _set_backend_timezone(timezone_str: str):
    global _BACKEND_TZ
    _BACKEND_TZ = timezone_str
    chart_engine_lib.py_set_backend_timezone(timezone_str)

def _ensure_timestamp(val):
    if val is None: return None
    return chart_engine_lib.py_ensure_timestamp(val)

# Monkey patch to_arrow to fix the PyO3-Polars bridge error in Polars 1.x
_orig_df_to_arrow = pl.DataFrame.to_arrow
def _patched_df_to_arrow(self, *args, **kwargs):
    kwargs.pop("compat_level", None)
    return _orig_df_to_arrow(self, *args, **kwargs)
pl.DataFrame.to_arrow = _patched_df_to_arrow

_orig_s_to_arrow = pl.Series.to_arrow
def _patched_s_to_arrow(self, *args, **kwargs):
    kwargs.pop("compat_level", None)
    return _orig_s_to_arrow(self, *args, **kwargs)
pl.Series.to_arrow = _patched_s_to_arrow

def _process_polars_data(df: pl.DataFrame) -> pl.DataFrame:
    """
    Delegates all DataFrame pre-processing to the high-performance Rust backend.
    Handles column sanitization, timestamp conversion, and timezone alignment.
    """
    if df is None: return None
    return chart_engine_lib.py_process_polars_data(df)

class DateTimeEncoder(json.JSONEncoder):
    """Bridge for DateTimeEncoder."""
    def default(self, obj):
        ts = _ensure_timestamp(obj)
        if ts is not None: return ts
        # Fallback to default JSON encoder for other types
        try:
            return super().default(obj)
        except TypeError:
            return str(obj)

logger = logging.getLogger("chart_engine")

# Global process for the singleton window
_TAURI_PROCESS = None
_READY_EVENT = False 

class ChartAPI:
    def __init__(self, ready_event):
        self.ready_event = ready_event
    def mark_ready(self):
        self.ready_event.set()
        return {"status": "ok"}
    def log_message(self, msg):
        return {"status": "ok"}

class SubChart:
    def __init__(self, chart, chart_id): self.chart, self.chart_id = chart, chart_id
    def create_line_series(self, name="Line"): return self.chart.create_line_series(name, self.chart_id)
    def create_candlestick_series(self, name="Price"): return self.chart.create_candlestick_series(name, self.chart_id)

class PriceLine:
    def __init__(self, rust_line, chart):
        self._rust_line, self.chart, self.line_id = rust_line, chart, rust_line.line_id
    def update(self, price):
        cmd = self._rust_line.update(price)
        if cmd: self.chart._send_command(json.loads(cmd))

class Series:
    def __init__(self, chart, series_id, name, chart_id="chart-0", rust_series=None):
        self.chart, self.series_id, self.name, self.chart_id, self._rust_series = chart, series_id, name, chart_id, rust_series
    def set_data(self, df):
        if self._rust_series:
            df = _process_polars_data(df)
            data_json = json.dumps(df.to_dicts(), cls=DateTimeEncoder)
            cmd = json.loads(self._rust_series.set_data(data_json))
            cmd["chartId"] = self.chart_id
            self.chart._send_command(cmd)
    def update(self, item):
        if self._rust_series:
            # Handle both dict and DataFrame/Series
            if isinstance(item, dict):
                item = pl.DataFrame([item])
            
            item = _process_polars_data(item)
            item_json = json.dumps(item.to_dicts()[0], cls=DateTimeEncoder)
            cmd = json.loads(self._rust_series.update(item_json))
            cmd["chartId"] = self.chart_id
            self.chart._send_command(cmd)
    def apply_options(self, options):
        if self._rust_series: self.chart._send_command(json.loads(self._rust_series.apply_options(json.dumps(options))))

    def add_marker(self, **kwargs):
        """Convenience method for adding a marker to this specific series."""
        time_val = kwargs.pop('time', None)
        return self.chart.add_marker(self.series_id, time_val, **kwargs)

    def add_band(self, df, color="rgba(31, 150, 243, 0.2)"):
        """
        Adds a Band (Cloud) indicator to this series using the Band Plugin.
        Requires a DataFrame with 'time', 'top', and 'bottom' columns.
        """
        if df is None or df.is_empty(): return
        
        # Process timestamps (handles renames like date -> time)
        # print(f"DEBUG: Before processing: {df.columns}")
        df = _process_polars_data(df)
        # print(f"DEBUG: After processing: {df.columns}")
        
        # Now check if we have the required columns for the band plugin
        required = {"time", "top", "bottom"}
        if not required.issubset(set(df.columns)):
            # Fallback: help the user by renaming if the backend missed it for some reason
            if "date" in df.columns and "time" not in df.columns:
                df = df.rename({"date": "time"})
            
            if not required.issubset(set(df.columns)):
                raise ValueError(f"Processed DataFrame must contain columns: {required}. Found: {df.columns}")
        
        # Emit command to frontend
        data_json = json.dumps(df.to_dicts(), cls=DateTimeEncoder)
        self.chart._send_command({
            "action": "create_band_plugin",
            "seriesId": self.series_id,
            "chartId": self.chart_id,
            "color": color,
            "data": json.loads(data_json)
        })

class Chart:
    _instance = None
    def __new__(cls, *args, **kwargs):
        if not cls._instance:
            cls._instance = super(Chart, cls).__new__(cls)
            cls._initialized = False
        return cls._instance

    def __init__(self, title="Chart Window", width=1200, height=800, main_series_id="main"):
        if getattr(self, '_initialized', False): return
        self.series, self._rust_chart = {}, chart_engine_lib.Chart()
        self.on_trade = None
        
        # Merge DrawingTool logic directly into Chart
        self.toolbox = self # For backward compatibility
        self._rust_toolbox = self._rust_chart.toolbox
        self._rust_trader = self._rust_chart.trader

        rmain = self._rust_chart.series.get(main_series_id)
        self.series[main_series_id] = Series(self, main_series_id, "Main", rust_series=rmain)

        global _TAURI_PROCESS, _READY_EVENT
        if _TAURI_PROCESS is None:
            bin_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "chart_engine")
            _TAURI_PROCESS = subprocess.Popen([bin_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1)
            
            # Start background listener for stderr
            threading.Thread(target=self.__consume_stderr, daemon=True).start()

            # Synchronous wait for ready
            for line in _TAURI_PROCESS.stdout:
                if "__READY__" in line: break
            _READY_EVENT = True
            
            # Start background listener for UI events
            threading.Thread(target=self.__listen_to_ui, daemon=True).start()

        # Auto-bind trading handler for unified API
        self.set_on_trade(self.trader_handle_callback)

        self._initialized = True
        atexit.register(self.exit)

    def __listen_to_ui(self):
        global _TAURI_PROCESS
        while _TAURI_PROCESS and _TAURI_PROCESS.poll() is None:
            line = _TAURI_PROCESS.stdout.readline()
            if not line: break
            try:
                if not line: break
                line = line.strip()
                # Diagnostic: Always print the line if we are troubleshooting
                # print(f"DEBUG: Tauri -> {line}")
                
                msg = json.loads(line)
                if msg.get("action") == "log":
                    level_name = msg.get("level", "INFO").upper()
                    if level_name == "WARN": level_name = "WARNING"
                    py_level = getattr(logging, level_name, logging.INFO)
                    logger.log(py_level, f"[rust:{msg.get('target', 'engine')}] {msg.get('message', '')}")
                    # Force print errors to console for visibility
                    if level_name in ["ERROR", "CRITICAL"]:
                        print(f"❌ [Chart Engine Error]: {msg.get('message', '')}")
                    continue

                if msg.get("action") == "trade" and self.on_trade:
                    self.on_trade(msg.get("data"))
                
                if msg.get("action") == "screenshot":
                    data = msg.get("data", {})
                    b64_data = data.get("base64", "")
                    filename = data.get("filename", "screenshot.png")
                    
                    if b64_data.startswith("data:image"):
                        b64_data = b64_data.split(",")[1]
                    
                    try:
                        with open(filename, "wb") as f:
                            f.write(base64.b64decode(b64_data))
                        print(f"🎬 Screenshot saved: {filename}")
                    except Exception as e:
                        logger.error(f"Failed to save screenshot: {e}")

            except json.JSONDecodeError as e:
                # Silently ignore noise, but log real issues if they look like JSON
                if line and (line.startswith('{') or line.startswith('[')):
                    logger.debug(f"Failed to parse line from Tauri: {line} | Error: {e}")
            except Exception as e:
                logger.error(f"Error in UI listener thread: {e}")

    def __consume_stderr(self):
        global _TAURI_PROCESS
        while _TAURI_PROCESS and _TAURI_PROCESS.poll() is None:
            line = _TAURI_PROCESS.stderr.readline()
            if not line: break
            line = line.strip()
            if line:
                print(f"⚠️ [Chart Engine Backend]: {line}")

    def set_on_trade(self, callback):
        self.on_trade = callback

    def update_positions(self, positions_data):
        """Update the active positions table in the UI"""
        self._send_command({"action": "update_positions", "data": positions_data})

    @property
    def positions(self):
        """Returns the current list of positions from the Rust trader"""
        return self._rust_trader.positions

    @property
    def last_price(self):
        """Returns the last market price from the Rust trader"""
        return self._rust_trader.last_price

    def set_layout(self, layout="single"):
        l_str = str(layout).lower()
        self._send_command({"action": "set_layout", "layout": l_str, "data": {"type": l_str}})
        num = 3 if "1p2" in l_str else 4 if "1p3" in l_str else 2 if "double" in l_str else 1
        return [SubChart(self, f"chart-{i}") for i in range(num)]

    def create_line_series(self, name="Line", chart_id="chart-0"):
        sid, cmd_json = self._rust_chart.create_line_series(name, chart_id)
        cmd = json.loads(cmd_json)
        cmd["chartId"], cmd["name"] = chart_id, name
        self._send_command(cmd)
        return Series(self, sid, name, chart_id=chart_id, rust_series=self._rust_chart.series.get(sid))

    def create_candlestick_series(self, name="Price", chart_id="chart-0"):
        sid, cmd_json = self._rust_chart.create_candlestick_series(name, chart_id)
        cmd = json.loads(cmd_json)
        cmd["chartId"], cmd["name"] = chart_id, name
        self._send_command(cmd)
        return Series(self, sid, name, chart_id=chart_id, rust_series=self._rust_chart.series.get(sid))

    def remove_series(self, series_id, chart_id="chart-0"):
        self._send_command({"action": "remove_series", "chartId": chart_id, "seriesId": series_id})

    def clear_all_series(self, chart_id="chart-0"):
        self._send_command({"action": "clear_all_series", "chartId": chart_id})

    def set_watermark(self, text, chart_id="chart-0"): 
        self._send_command({"action": "set_watermark", "chartId": chart_id, "data": {"text": text}})
    
    def set_timezone(self, tz): 
        _set_backend_timezone(tz)
        self._send_command({"action": "set_timezone", "data": {"timezone": tz}})
    
    def set_tooltip(self, v): 
        """Show or hide the floating tooltip on crosshair move (via Rust)"""
        cmd = self._rust_chart.set_tooltip(bool(v))
        self._send_command(json.loads(cmd))
    
    def enable_tooltip(self): self.set_tooltip(True)
    def disable_tooltip(self): self.set_tooltip(False)
    
    def set_layout_toolbar_visibility(self, v):
        """Show or hide the side toolbar containing layout settings"""
        cmd = self._rust_chart.set_layout_toolbar_visibility(bool(v))
        self._send_command(json.loads(cmd))

    def enable_layout_toolbar(self): self.set_layout_toolbar_visibility(True)
    def disable_layout_toolbar(self): self.set_layout_toolbar_visibility(False)
    
    def set_trend_info_visibility(self, v): 
        self._send_command({"action": "set_trend_info_visibility", "data": {"visible": v}})
    
    def set_layout_toolbar_visibility(self, v): 
        self._send_command({"action": "set_layout_toolbar_visibility", "data": {"visible": v}})

    def set_legend_visibility(self, v):
        self._send_command({"action": "set_legend_visibility", "data": {"visible": v}})

    def update_trend_info(self, **kwargs):
        self._send_command({"action": "update_trend", "data": kwargs})

    def set_crosshair_mode(self, mode=0):
        # 0 = Normal, 1 = Magnet
        self._send_command({"action": "set_crosshair_mode", "data": {"mode": mode}})

    def set_sync(self, enabled=False):
        self._send_command({"action": "set_sync", "data": {"enabled": enabled}})
    
    def take_screenshot(self, filename=None, chart_id="chart-0"):
        self._send_command({"action": "take_screenshot", "chartId": chart_id, "filename": filename})
    
    ################################################
    # --- Drawing & Markers (from DrawingTool) --- #
    ################################################
    def sync_active_position(self, is_opened, **kwargs):
        for c in self._rust_toolbox.sync_active_position(is_opened, **kwargs):
            self._send_command(json.loads(c))

    def add_marker(self, series_id, time, **kwargs):
        mid, cmd = self._rust_toolbox.add_marker(series_id, _ensure_timestamp(time), kwargs.get('position', "aboveBar"), kwargs.get('color', "#2196F3"), kwargs.get('shape', "arrowDown"), kwargs.get('text', ""), kwargs.get('chart_id', "chart-0"))
        self._send_command(json.loads(cmd))
        return mid

    def create_box(self, start_time, start_price, end_time, end_price, **kwargs):
        st = _ensure_timestamp(start_time)
        et = _ensure_timestamp(end_time)
        bid, cmds = self._rust_toolbox.create_box(st, start_price, et, end_price, kwargs.get('color', "rgba(33, 150, 243, 0.2)"), kwargs.get('border_color', "#2196F3"), kwargs.get('text', ""), kwargs.get('category'), kwargs.get('chart_id', "chart-0"))
        for c in cmds: self._send_command(json.loads(c))
        return bid

    def create_horizontal_line(self, series_id, price, **kwargs):
        lid, cmd = self._rust_toolbox.create_horizontal_line(series_id, price, kwargs.get('color', "#F44336"), kwargs.get('chart_id', "chart-0"))
        if cmd: self._send_command(json.loads(cmd))
        return PriceLine(self._rust_toolbox.lines.get(lid), self)

    def _create_line_tool(self, tool_type, start_time, start_price, end_time, end_price, **kwargs):
        st = _ensure_timestamp(start_time)
        et = _ensure_timestamp(end_time)
        tid, cmd = self._rust_toolbox.create_line_tool(tool_type, st, start_price, et, end_price, kwargs.get('color', "#2196F3"), kwargs.get('width', 1), kwargs.get('style', 0), kwargs.get('visible', True), kwargs.get('text', ""), kwargs.get('extended', False), kwargs.get('chart_id', "chart-0"))
        self._send_command(json.loads(cmd))
        return tid

    def create_trendline(self, st, sp, et, ep, **k): return self._create_line_tool("trendline", st, sp, et, ep, **k)
    def create_ray(self, st, sp, et, ep, **k): k['extended'] = True; return self._create_line_tool("ray", st, sp, et, ep, **k)
    def create_fib_retracement(self, st, sp, et, ep, **k): return self._create_line_tool("fib", st, sp, et, ep, **k)
    def remove_line_tool(self, tid): self._send_command(json.loads(self._rust_toolbox.remove_line_tool(tid)))
    def clear_line_tools(self): self._send_command(json.loads(self._rust_toolbox.clear_line_tools()))
    def remove_box(self, bid): self._send_command(json.loads(self._rust_toolbox.remove_box(bid)))
    def create_long_position(self, st, ep, sl, tp, **k):
        pid, cmds = self._rust_toolbox.create_position(_ensure_timestamp(st), ep, sl, tp, _ensure_timestamp(k.get('end_time')), k.get('visible', True), "long", k.get('quantity', 1.0), k.get('chart_id', "chart-0"))
        for c in cmds: self._send_command(json.loads(c))
        return pid

    def create_short_position(self, st, ep, sl, tp, **k):
        pid, cmds = self._rust_toolbox.create_position(_ensure_timestamp(st), ep, sl, tp, _ensure_timestamp(k.get('end_time')), k.get('visible', True), "short", k.get('quantity', 1.0), k.get('chart_id', "chart-0"))
        for c in cmds: self._send_command(json.loads(c))
        return pid

    def remove_position(self, pid): self._send_command(json.loads(self._rust_toolbox.remove_position(pid)))
    def clear_positions(self, cid=None):
        for c in self._rust_toolbox.clear_positions(cid): self._send_command(json.loads(c))
    
    ########################################
    # --- Paper Trading Logic (Merged) --- #
    ########################################
    def trader_handle_callback(self, data):
        """Internal callback for trade events from the UI"""
        cmds = self._rust_trader.handle_callback(json.dumps(data))
        for c in cmds: self._send_command(json.loads(c))

    def trader_update_price(self, price):
        """Update market price and check TP/SL for all positions in Rust"""
        cmds = self._rust_trader.update_price(price)
        for c in cmds: self._send_command(json.loads(c))

    def trader_execute(self, side, qty, price=None, tp=None, sl=None, series=None, time=None):
        """Programmatically execute a trade in the Rust backend"""
        st = _ensure_timestamp(time) if time else None
        cmds = self._rust_trader.execute(side, qty, price, tp, sl, st)
        for c in cmds: self._send_command(json.loads(c))
        
        if series:
            exec_price = price or self.last_price
            if exec_price:
                is_buy = side.lower() == 'buy'
                series.add_marker(
                    time=time,
                    position="belowBar" if is_buy else "aboveBar",
                    shape="arrowUp" if is_buy else "arrowDown",
                    color="#00e676" if is_buy else "#ff5252",
                    text=f"{side.upper()} @ {exec_price:.2f}"
                )

    def show_notification(self, message, type="info"):
        """Show a toast notification in the UI"""
        self._send_command({"action": "show_notification", "data": {"message": message, "type": type}})

    def show(self, block=True):
        """Keep the window open and block the Python script until it is closed."""
        global _TAURI_PROCESS
        if not _TAURI_PROCESS: return
        
        if block:
            try:
                # This will block until the child process (Tauri window) exits
                _TAURI_PROCESS.wait()
            except KeyboardInterrupt:
                self.exit()
    
    def _send_command(self, cmd):
        global _TAURI_PROCESS
        if _TAURI_PROCESS and _TAURI_PROCESS.poll() is None:
            cmd_json = json.dumps(cmd, cls=DateTimeEncoder)
            try:
                _TAURI_PROCESS.stdin.write(cmd_json + "\n")
                _TAURI_PROCESS.stdin.flush()
            except: pass

    def exit(self): 
        global _TAURI_PROCESS
        if _TAURI_PROCESS:
            _TAURI_PROCESS.terminate()
            _TAURI_PROCESS = None


