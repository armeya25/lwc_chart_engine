import logging
import sys
from chart_engine import Chart
import os
import polars as pl
import time

def test_chart():
    
    # 1. Load Data
    parquet_path = "data/1d.parquet"
    if not os.path.exists(parquet_path):
        print(f"Error: {parquet_path} not found.")
        return
        
    df = pl.read_parquet(parquet_path)
    df = df.tail(100)
    #df = df.rename({"date":"time"})
    # 2. Initialize and Show Chart
    chart = Chart(title="Chart Engine v0.6.3 - SubChart Test")
    #chart.show()  # Launch the Tauri window
    
    # 3. Configure Layout and Series
    subcharts = chart.set_layout("single")
    ch1 = subcharts[0].create_candlestick_series(name="BTC/USD")
    print(f"Series created: {ch1}")
    
    # 4. Set Data
    ch1.set_data(df)
    print("✅ Data series set successfully. Window should be open.")
    chart.show()

if __name__ == "__main__":
    test_chart()
