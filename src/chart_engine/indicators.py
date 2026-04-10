import json

def _get_schemas():
    from .chart import _get_indicator_schemas
    return _get_indicator_schemas()


class IndicatorMixin:
    """
    Mixin class for Series to provide technical indicator methods.
    All heavy computation is delegated to the stateful Rust backend.
    """

    # ── Trend / Overlay ──────────────────────────────────────────────────────

    def add_sma(self, period: int = 20, color: str = "#2962FF"):
        """Simple Moving Average overlay."""
        return self.add_indicator_v2("sma", {"period": period}, _get_schemas().get("sma", {}))

    def add_ema(self, period: int = 20, color: str = "#FF9800"):
        """Exponential Moving Average overlay."""
        return self.add_indicator_v2("ema", {"period": period}, _get_schemas().get("ema", {}))

    def add_dema(self, period: int = 20, color: str = "#26C6DA"):
        """Double EMA (2×EMA − EMA(EMA)) — reduced lag vs plain EMA."""
        return self.add_indicator_v2("dema", {"period": period}, _get_schemas().get("dema", {}))

    def add_tema(self, period: int = 20, color: str = "#AB47BC"):
        """Triple EMA (3×EMA − 3×EMA² + EMA³) — minimal lag trend follower."""
        return self.add_indicator_v2("tema", {"period": period}, _get_schemas().get("tema", {}))

    def add_bollinger_bands(self, period: int = 20, std_dev: float = 2.0, color: str = None):
        """Bollinger Bands (upper / mid / lower) overlay."""
        return self.add_indicator_v2("bollingerbands", {"period": period, "std_dev": std_dev}, _get_schemas().get("bollingerbands", {}))

    def add_vwap(self, color: str = "#E91E63"):
        """Volume-Weighted Average Price (cumulative, intra-session)."""
        s = self.chart.create_line_series(name="VWAP", chart_id=self.chart_id)
        s.apply_options({"color": color, "lineWidth": 2,
                          "priceLineVisible": False, "lastValueVisible": True})
        self.add_indicator("vwap", id=s.series_id, params={})
        return s

    # ── Oscillators (sub-pane) ───────────────────────────────────────────────

    def add_rsi(self, period: int = 14, color: str = None):
        """RSI oscillator (0–100) in a dedicated sub-pane."""
        return self.add_indicator_v2("rsi", {"period": period}, _get_schemas().get("rsi", {}))

    def add_macd(self, fast: int = 12, slow: int = 26, signal: int = 9):
        """MACD line + Signal + Histogram in a dedicated sub-pane."""
        return self.add_indicator_v2("macd", {"fast": fast, "slow": slow, "signal": signal}, _get_schemas().get("macd", {}))

    def add_atr(self, period: int = 14, color: str = None):
        """Average True Range — volatility measure in a dedicated sub-pane."""
        return self.add_indicator_v2("atr", {"period": period}, _get_schemas().get("atr", {}))

    def add_stochastic(self, period: int = 14, smooth_k: int = 3, smooth_d: int = 3, color: str = None):
        """Stochastic Oscillator (%K + %D) in a dedicated sub-pane."""
        return self.add_indicator_v2("stochastic", {"period": period, "smooth_k": smooth_k, "smooth_d": smooth_d}, _get_schemas().get("stochastic", {}))

    def add_cci(self, period: int = 20, color: str = None):
        """Commodity Channel Index oscillator in a dedicated sub-pane."""
        return self.add_indicator_v2("cci", {"period": period}, _get_schemas().get("cci", {}))

    def add_williams_r(self, period: int = 14, color: str = None):
        """Williams %R oscillator (-100 to 0) in a dedicated sub-pane."""
        return self.add_indicator_v2("williamsr", {"period": period}, _get_schemas().get("williamsr", {}))

    # ── Legacy Python-side Bollinger cloud (kept for backwards compat) ────────

    def add_bollinger_bands_cloud(self, period: int = 20, std_dev: float = 2.0,
                                   color: str = "rgba(31, 150, 243, 0.1)"):
        """Legacy Python Bollinger cloud. Prefer add_bollinger_bands() for Rust performance."""
        if self._last_df is None:
            return
        import polars as pl
        df = self._last_df.with_columns([
            pl.col("close").rolling_mean(window_size=period).alias("basis"),
            pl.col("close").rolling_std(window_size=period).alias("std"),
        ])
        df = df.with_columns([
            (pl.col("basis") + pl.col("std") * std_dev).alias("top"),
            (pl.col("basis") - pl.col("std") * std_dev).alias("bottom"),
        ]).drop_nulls()
        self.add_sma(period=period, color="rgba(255,255,255,0.2)")
        self.add_band(df.select(["time", "top", "bottom"]), color=color)
