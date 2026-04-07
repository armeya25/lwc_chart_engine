import json
import polars as pl
from . import chart_engine_lib
import datetime
import zoneinfo

# State for backend timezone (synced with Rust)
_BACKEND_TZ = "UTC"

def set_backend_timezone(timezone_str: str):
    global _BACKEND_TZ
    _BACKEND_TZ = timezone_str
    chart_engine_lib.py_set_backend_timezone(timezone_str)

def ensure_timestamp(val):
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

def process_polars_data(df: pl.DataFrame) -> pl.DataFrame:
    """
    Delegates all DataFrame pre-processing to the high-performance Rust backend.
    Handles column sanitization, timestamp conversion, and timezone alignment.
    """
    if df is None: return None
    return chart_engine_lib.py_process_polars_data(df)

class DateTimeEncoder(json.JSONEncoder):
    """Bridge for DateTimeEncoder."""
    def default(self, obj):
        ts = ensure_timestamp(obj)
        if ts is not None: return ts
        return super().default(obj)
