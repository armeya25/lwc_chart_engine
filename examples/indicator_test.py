import json
import time
import os
import polars as pl
from chart_engine.chart import Chart

def test_indicators():
    print("Testing Indicators with Rust Backend using 1d.parquet...")
    
    # 1. Load data
    parquet_path = "data/1d.parquet"
    if not os.path.exists(parquet_path):
        print(f"Error: {parquet_path} not found.")
        return
        
    df = pl.read_parquet(parquet_path)
    print(f"✅ Loaded {len(df)} rows.")
    
    # Use first 500 rows for initial batch
    data_df = df.head(500)
    
    chart = Chart(title="Indicator Test - Real Data")
    series = chart.create_candlestick_series("Main Series", "chart-0")
    
    print("Adding SMA(14)...")
    series.add_sma(period=14)
    
    print("Adding MACD(12, 26, 9)...")
    series.add_macd(fast=12, slow=26, signal=9)
    
    print("Setting data (Initial Batch)...")
    commands = series.set_data(data_df)
    
    print(f"Generated {len(commands)} commands.")
    
    # Check for indicator commands
    has_sma = any("sma" in str(cmd).lower() for cmd in commands)
    has_indicators = len(commands) > 2 # Main + Volume + Indicators
    
    print(f"SMA Command found: {has_sma}")
    print(f"Indicators generated commands: {has_indicators}")
    
    if has_indicators:
        print("SUCCESS: Indicators generated data commands.")
    else:
        print("FAILURE: No indicator data commands generated.")

    """print("\nTesting Real-time Update (Next point from data)...")
    if len(df) > 500:
        # Take the 501st point
        next_point = df.slice(500, 1).to_dicts()[0]
        update_cmds = series.update(next_point)
        print(f"Generated {len(update_cmds)} update commands for time {next_point['date']}.")"""
    
    chart.show()
    
if __name__ == "__main__":
    test_indicators()

