import json
from . import chart_engine_lib
from .time_utils import ensure_timestamp

class PriceLine:
    def __init__(self, rust_line, tool):
        self._rust_line, self.tool, self.line_id = rust_line, tool, rust_line.line_id
    def update(self, price):
        cmd = self._rust_line.update(price)
        if cmd: self.tool.chart._send_command(json.loads(cmd))

class DrawingTool:
    def __init__(self, chart, rust_toolbox=None):
        self.chart = chart
        self._rust_toolbox = rust_toolbox or chart_engine_lib.DrawingTool()

    def sync_active_position(self, is_opened, **kwargs):
        for c in self._rust_toolbox.sync_active_position(is_opened, **kwargs):
            self.chart._send_command(json.loads(c))

    def add_marker(self, series_id, time, **kwargs):
        mid, cmd = self._rust_toolbox.add_marker(series_id, ensure_timestamp(time), kwargs.get('position', "aboveBar"), kwargs.get('color', "#2196F3"), kwargs.get('shape', "arrowDown"), kwargs.get('text', ""), kwargs.get('chart_id', "chart-0"))
        self.chart._send_command(json.loads(cmd))
        return mid

    def create_box(self, start_time, start_price, end_time, end_price, **kwargs):
        # Normalize timestamps for Rust
        st = ensure_timestamp(start_time)
        et = ensure_timestamp(end_time)
        bid, cmds = self._rust_toolbox.create_box(st, start_price, et, end_price, kwargs.get('color', "rgba(33, 150, 243, 0.2)"), kwargs.get('border_color', "#2196F3"), kwargs.get('text', ""), kwargs.get('category'), kwargs.get('chart_id', "chart-0"))
        for c in cmds: self.chart._send_command(json.loads(c))
        return bid

    def create_horizontal_line(self, series_id, price, **kwargs):
        lid, cmd = self._rust_toolbox.create_horizontal_line(series_id, price, kwargs.get('color', "#F44336"), kwargs.get('chart_id', "chart-0"))
        if cmd: self.chart._send_command(json.loads(cmd))
        return PriceLine(self._rust_toolbox.lines.get(lid), self)

    def _create_line_tool(self, tool_type, start_time, start_price, end_time, end_price, **kwargs):
        st = ensure_timestamp(start_time)
        et = ensure_timestamp(end_time)
        tid, cmd = self._rust_toolbox.create_line_tool(tool_type, st, start_price, et, end_price, kwargs.get('color', "#2196F3"), kwargs.get('width', 1), kwargs.get('style', 0), kwargs.get('visible', True), kwargs.get('text', ""), kwargs.get('extended', False), kwargs.get('chart_id', "chart-0"))
        self.chart._send_command(json.loads(cmd))
        return tid

    def create_trendline(self, st, sp, et, ep, **k): return self._create_line_tool("trendline", st, sp, et, ep, **k)
    def create_ray(self, st, sp, et, ep, **k): k['extended'] = True; return self._create_line_tool("ray", st, sp, et, ep, **k)
    def create_fib_retracement(self, st, sp, et, ep, **k): return self._create_line_tool("fib", st, sp, et, ep, **k)
    def remove_line_tool(self, tid): self.chart._send_command(json.loads(self._rust_toolbox.remove_line_tool(tid)))
    def clear_line_tools(self): self.chart._send_command(json.loads(self._rust_toolbox.clear_line_tools()))
    def remove_box(self, bid): self.chart._send_command(json.loads(self._rust_toolbox.remove_box(bid)))
    def create_long_position(self, st, ep, sl, tp, **k):
        pid, cmds = self._rust_toolbox.create_position(ensure_timestamp(st), ep, sl, tp, ensure_timestamp(k.get('end_time')), k.get('visible', True), "long", k.get('quantity', 1.0), k.get('chart_id', "chart-0"))
        for c in cmds: self.chart._send_command(json.loads(c))
        return pid

    def create_short_position(self, st, ep, sl, tp, **k):
        pid, cmds = self._rust_toolbox.create_position(ensure_timestamp(st), ep, sl, tp, ensure_timestamp(k.get('end_time')), k.get('visible', True), "short", k.get('quantity', 1.0), k.get('chart_id', "chart-0"))
        for c in cmds: self.chart._send_command(json.loads(c))
        return pid

    def remove_position(self, pid): self.chart._send_command(json.loads(self._rust_toolbox.remove_position(pid)))
    def clear_positions(self, cid=None):
        for c in self._rust_toolbox.clear_positions(cid): self.chart._send_command(json.loads(c))
