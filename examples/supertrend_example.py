import polars as pl
import numpy as np
import os
from chart_engine import Chart

# --- 1. Supertrend Calculation Utility ---

def calculate_supertrend(df: pl.DataFrame, period: int = 10, multiplier: float = 3.0) -> pl.DataFrame:
    """
    Calculates the Supertrend indicator using Polars for preprocessing 
    and NumPy for the recursive trailing stop logic.
    """
    # 1.1 Calculate True Range and ATR
    df = df.with_columns([
        (pl.col("high") - pl.col("low")).alias("tr1"),
        (pl.col("high") - pl.col("close").shift(1)).abs().alias("tr2"),
        (pl.col("low") - pl.col("close").shift(1)).abs().alias("tr3")
    ])
    
    df = df.with_columns(
        pl.max_horizontal("tr1", "tr2", "tr3").alias("tr")
    )
    
    # Simple Moving Average for ATR (use ewm_mean for more "classic" ATR if desired)
    df = df.with_columns(
        pl.col("tr").rolling_mean(window_size=period).alias("atr")
    )

    # 1.2 Calculate Basic Upper and Lower Bands
    df = df.with_columns([
        ((pl.col("high") + pl.col("low")) / 2 + multiplier * pl.col("atr")).alias("basic_ub"),
        ((pl.col("high") + pl.col("low")) / 2 - multiplier * pl.col("atr")).alias("basic_lb")
    ])

    # 1.3 Iterative logic for Final Bands (Trailing Stops)
    # We use NumPy here because Polars' vectorized expressions don't 
    # natively support this specific row-on-prev-row recursive dependency easily.
    closes = df["close"].to_numpy()
    basic_ubs = df["basic_ub"].to_numpy()
    basic_lbs = df["basic_lb"].to_numpy()
    
    final_ubs = np.zeros(len(df))
    final_lbs = np.zeros(len(df))
    supertrend = np.zeros(len(df))
    
    # Initialize first valid row
    start_idx = period
    for i in range(start_idx, len(df)):
        # Final Upperband
        if basic_ubs[i] < final_ubs[i-1] or closes[i-1] > final_ubs[i-1]:
            final_ubs[i] = basic_ubs[i]
        else:
            final_ubs[i] = final_ubs[i-1]
            
        # Final Lowerband
        if basic_lbs[i] > final_lbs[i-1] or closes[i-1] < final_lbs[i-1]:
            final_lbs[i] = basic_lbs[i]
        else:
            final_lbs[i] = final_lbs[i-1]
            
        # Supertrend switch logic
        if supertrend[i-1] == final_ubs[i-1]:
            supertrend[i] = final_lbs[i] if closes[i] > final_ubs[i] else final_ubs[i]
        else:
            supertrend[i] = final_ubs[i] if closes[i] < final_lbs[i] else final_lbs[i]

    return df.with_columns([
        pl.Series("st", supertrend),
        pl.Series("st_ub", final_ubs),
        pl.Series("st_lb", final_lbs)
    ])


# --- 2. Main Script ---

def main():
    # 2.1 Load Data
    data_path = "data/1d.parquet"
    if not os.path.exists(data_path):
        print(f"❌ Error: {data_path} not found. Please ensure the data exists.")
        return

    print(f"📈 Loading data from {data_path}...")
    df = pl.read_parquet(data_path)
    
    # 2.2 Calculate Supertrend
    print("🪄 Calculating Supertrend (10, 3.0)...")
    df = calculate_supertrend(df, period=10, multiplier=3.0)
    
    # Calculate Bollinger Bands for the "Band Indicator" demonstration
    df = df.with_columns([
        pl.col("close").rolling_mean(window_size=20).alias("sma20"),
        pl.col("close").rolling_std(window_size=20).alias("std20")
    ]).with_columns([
        (pl.col("sma20") + 2 * pl.col("std20")).alias("bb_upper"),
        (pl.col("sma20") - 2 * pl.col("std20")).alias("bb_lower")
    ])

    # 2.3 Initialize Chart
    chart = Chart(title="Chart Engine v0.6.3 - Supertrend & Bands")
    
    # 2.4 Create Main Price Series
    price_series = chart.create_candlestick_series(name="Price")
    price_series.set_data(df)
    
    # 2.5 Add Bollinger Band Cloud using the new add_band API
    print("☁️ Adding Bollinger Band cloud...")
    bb_df = df.select([
        pl.col("date"),
        pl.col("bb_upper").alias("top"),
        pl.col("bb_lower").alias("bottom")
    ]).drop_nulls()
    
    price_series.add_band(bb_df, color="rgba(76, 175, 80, 0.15)")
    
    # 2.6 Add Supertrend Line
    print("📉 Adding Supertrend Line...")
    st_series = chart.create_line_series(name="Supertrend")
    
    # Apply styling: color the line based on position relative to price is a FE feature,
    # but here we just set a neutral color or split data. For simplicity, one line:
    st_series.set_data(df.select(["date", pl.col("st").alias("value")]))
    st_series.apply_options({
        "color": "#FF9800",
        "lineWidth": 2,
        "lineStyle": 0 # Solid
    })

    # 2.7 Show results
    print("✅ Done! Window should be open.")
    chart.show()

if __name__ == "__main__":
    main()
