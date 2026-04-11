
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
    series.set_auto_volume(False)
    
    print("Adding SMA(14)...")
    series.add_sma(period=14)
    series.add_sma(period=50)
    
    print("Setting data...")
    series.set_data(data_df)
    
    chart.show()
    
if __name__ == "__main__":
    test_indicators()

