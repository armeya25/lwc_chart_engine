import polars as pl
import time
from chart_engine import Chart

def run_test():
    # 1. Load data
    print("📂 Loading data/1d.parquet...")
    df = pl.read_parquet("data/1d.parquet")
    print(f"✅ Loaded {len(df)} rows.")

    # 2. Initialize Chart
    print("🚀 Initializing Chart Engine...")
    chart = Chart(title="Live Stream Emulation")
    
    # 3. Create Series
    series = chart.create_candlestick_series(name="SOLUSDT")
    
    # 4. Set Initial Data (First 1000 bars)
    initial_bars = 1000
    print(f"📊 Setting initial {initial_bars} bars...")
    series.set_data(df.head(initial_bars))
    
    # 5. Emulate Live Stream
    print("⏱ Starting live stream emulation...")
    stream_df = df.slice(initial_bars)
    
    try:
        for i, row in enumerate(stream_df.iter_rows(named=True)):
            # Update the chart series with the new bar
            series.update(row)
            
            # Sync the paper trader's internal price with the close of the new bar
            chart.trader_update_price(row['close'])
            
            if i % 10 == 0:
                print(f"📡 Streamed {i} updates... Last Price: {row['close']:.2f}")
            
            # Artificial delay to mimic live updates
            time.sleep(0.05) 
            
            # Exit if window closed (hypothetical check)
            if i > 500: # Limit test to 500 bars for demonstration
                break
                
    except KeyboardInterrupt:
        print("🛑 Stream interrupted by user.")
    finally:
        chart.exit()
        print("✨ Test complete.")

if __name__ == "__main__":
    run_test()
