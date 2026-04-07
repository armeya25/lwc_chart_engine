import subprocess
import os
import json
import uuid
import polars as pl
from . import chart_engine_lib
import time
import threading
import logging
from .time_utils import (DateTimeEncoder, set_backend_timezone, process_polars_data)

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

class Series:
    def __init__(self, chart, series_id, name, chart_id="chart-0", rust_series=None):
        self.chart, self.series_id, self.name, self.chart_id, self._rust_series = chart, series_id, name, chart_id, rust_series
    def set_data(self, df):
        if self._rust_series:
            df = process_polars_data(df)
            data_json = json.dumps(df.to_dicts(), cls=DateTimeEncoder)
            cmd = json.loads(self._rust_series.set_data(data_json))
            cmd["chartId"] = self.chart_id
            self.chart._send_command(cmd)
    def update(self, item):
        if self._rust_series:
            # Handle both dict and DataFrame/Series
            if isinstance(item, dict):
                item = pl.DataFrame([item])
            
            item = process_polars_data(item)
            item_json = json.dumps(item.to_dicts()[0], cls=DateTimeEncoder)
            cmd = json.loads(self._rust_series.update(item_json))
            cmd["chartId"] = self.chart_id
            self.chart._send_command(cmd)
    def apply_options(self, options):
        if self._rust_series: self.chart._send_command(json.loads(self._rust_series.apply_options(json.dumps(options))))

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
        from .drawings import DrawingTool
        self.toolbox = DrawingTool(self, rust_toolbox=self._rust_chart.toolbox)
        rmain = self._rust_chart.series.get(main_series_id)
        self.series[main_series_id] = Series(self, main_series_id, "Main", rust_series=rmain)

        global _TAURI_PROCESS, _READY_EVENT
        if _TAURI_PROCESS is None:
            bin_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "chart_engine")
            _TAURI_PROCESS = subprocess.Popen([bin_path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
            
            # Synchronous wait for ready
            for line in _TAURI_PROCESS.stdout:
                if "__READY__" in line: break
            _READY_EVENT = True
            
            # Start background listener for UI events
            threading.Thread(target=self._listen_to_ui, daemon=True).start()

        self._initialized = True

    def _listen_to_ui(self):
        global _TAURI_PROCESS
        while _TAURI_PROCESS and _TAURI_PROCESS.poll() is None:
            line = _TAURI_PROCESS.stdout.readline()
            if not line: break
            try:
                msg = json.loads(line)
                if msg.get("action") == "log":
                    level_name = msg.get("level", "INFO").upper()
                    if level_name == "WARN": level_name = "WARNING"
                    py_level = getattr(logging, level_name, logging.INFO)
                    logger.log(py_level, f"[rust:{msg.get('target', 'engine')}] {msg.get('message', '')}")
                    continue

                if msg.get("action") == "trade" and self.on_trade:
                    self.on_trade(msg.get("data"))
            except: pass

    def set_on_trade(self, callback):
        self.on_trade = callback

    def update_positions(self, positions):
        """Update the active positions table in the UI"""
        self._send_command({"action": "update_positions", "data": positions})

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
        set_backend_timezone(tz)
        self._send_command({"action": "set_timezone", "data": {"timezone": tz}})
    
    def set_tooltip(self, v): 
        self._send_command({"action": "set_tooltip", "data": {"enabled": v}})
    
    def set_trend_info_visibility(self, v): 
        self._send_command({"action": "set_trend_info_visibility", "data": {"visible": v}})
    
    def set_layout_toolbar_visibility(self, v): 
        self._send_command({"action": "set_layout_toolbar_visibility", "data": {"visible": v}})

    def set_legend_visibility(self, v):
        self._send_command({"action": "set_legend_visibility", "data": {"visible": v}})

    def set_timeframe(self, tf):
        self._send_command({"action": "set_timeframe", "data": tf})

    def update_trend_info(self, **kwargs):
        self._send_command({"action": "update_trend", "data": kwargs})

    def set_crosshair_mode(self, mode=0):
        # 0 = Normal, 1 = Magnet
        self._send_command({"action": "set_crosshair_mode", "data": {"mode": mode}})

    def set_sync(self, enabled=True):
        self._send_command({"action": "set_sync", "data": {"enabled": enabled}})
    
    def take_screenshot(self, chart_id="chart-0"):
        self._send_command({"action": "take_screenshot", "chartId": chart_id})

    def show(self):
        """API compatibility - Tauri window is singleton and launches on __init__"""
        pass

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
