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

    def add_sma(self, period: int = 14, color: str = None):
        """Simple Moving Average overlay."""
        return self.add_indicator_v2("sma", {"period": period, "color": color}, _get_schemas().get("sma", {}))

    def add_ema(self, period: int = 14, color: str = None):
        """Exponential Moving Average overlay."""
        return self.add_indicator_v2("ema", {"period": period, "color": color}, _get_schemas().get("ema", {}))

    def add_dema(self, period: int = 14, color: str = None):
        """Double EMA (2×EMA − EMA(EMA)) — reduced lag vs plain EMA."""
        return self.add_indicator_v2("dema", {"period": period, "color": color}, _get_schemas().get("dema", {}))

    def add_tema(self, period: int = 14, color: str = None):
        """Triple EMA (3×EMA − 3×EMA² + EMA³) — minimal lag trend follower."""
        return self.add_indicator_v2("tema", {"period": period, "color": color}, _get_schemas().get("tema", {}))

    def add_bollinger_bands(self, period: int = 14, std_dev: float = 2.0, color: str = None):
        """Bollinger Bands (upper / mid / lower) overlay."""
        params = {"period": period, "std_dev": std_dev}
        if color: params["color"] = color
        return self.add_indicator_v2("bollingerbands", params, _get_schemas().get("bollingerbands", {}))

    def add_vwap(self, color: str = None):
        """Volume-Weighted Average Price (cumulative, intra-session)."""
        return self.add_indicator_v2("vwap", {"color": color}, _get_schemas().get("vwap", {}))

    # ── Oscillators (sub-pane) ───────────────────────────────────────────────

    def add_rsi(self, period: int = 14, color: str = None):
        """RSI oscillator (0–100) in a dedicated sub-pane."""
        return self.add_indicator_v2("rsi", {"period": period, "color": color}, _get_schemas().get("rsi", {}))

    def add_macd(self, fast: int = 12, slow: int = 26, signal: int = 9):
        """MACD line + Signal + Histogram in a dedicated sub-pane."""
        return self.add_indicator_v2("macd", {"fast": fast, "slow": slow, "signal": signal}, _get_schemas().get("macd", {}))

    def add_atr(self, period: int = 14, color: str = None):
        """Average True Range — volatility measure in a dedicated sub-pane."""
        return self.add_indicator_v2("atr", {"period": period, "color": color}, _get_schemas().get("atr", {}))

    def add_stochastic(self, period: int = 14, smooth_k: int = 3, smooth_d: int = 3, color: str = None):
        """Stochastic Oscillator (%K + %D) in a dedicated sub-pane."""
        return self.add_indicator_v2("stochastic", {"period": period, "smooth_k": smooth_k, "smooth_d": smooth_d, "color": color}, _get_schemas().get("stochastic", {}))

    def add_cci(self, period: int = 14, color: str = None):
        """Commodity Channel Index oscillator in a dedicated sub-pane."""
        return self.add_indicator_v2("cci", {"period": period, "color": color}, _get_schemas().get("cci", {}))

    def add_williams_r(self, period: int = 14, color: str = None):
        """Williams %R oscillator (-100 to 0) in a dedicated sub-pane."""
        return self.add_indicator_v2("williamsr", {"period": period, "color": color}, _get_schemas().get("williamsr", {}))


