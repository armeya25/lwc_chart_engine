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
    
    # Critical: Use consistent naming. 
    # The engine handles 'date' -> 'time' mapping internally, 
    # but renaming here keeps the Python script synchronized with the engine's X-axis.
    if 'date' in df.columns:
        df = df.rename({'date': 'time'})

    # 2. Initialize Chart
    chart = Chart(title="Chart Engine - Drawing Alignment Fix")
    
    # 3. Configure Layout and Series
    subcharts = chart.set_layout("single")
    ch1 = subcharts[0].create_candlestick_series(name="BTC/USD")
    
    # 4. Set Data
    ch1.set_data(df)
    
    # 5. Add Trendline
    # We use the 'time' column which now matches exactly between 
    # Python scalars and the processed candle data in Rust.
    t_start, p_start = df['time'][0], df['close'][0]
    t_end, p_end = df['time'][-1], df['close'][-1]
    
    print(f"Drawing trendline from {t_start} to {t_end}")
    chart.create_trendline(t_start, p_start, t_end, p_end, color="#2ebd85", width=3)

    # 6. Add Fibonacci Retracement
    # Automatically find the local low and high in the 100-bar slice
    local_low_idx = df['low'].arg_min()
    local_high_idx = df['high'].arg_max()
    
    t_low, p_low = df['time'][local_low_idx], df['low'][local_low_idx]
    t_high, p_high = df['time'][local_high_idx], df['high'][local_high_idx]
    
    print(f"Drawing Fib from low at {p_low} to high at {p_high}")
    chart.create_fib_retracement(t_low, p_low, t_high, p_high, color="rgba(100, 150, 250, 0.4)")
    
    # 7. Add Box (Supply Zone)
    # Highlight the area around the recent high for the last 20 bars
    box_start_idx = max(0, len(df) - 20)
    t_box_start = df['time'][box_start_idx]
    
    chart.create_box(
        t_box_start, p_high * 1.01, t_end, p_high * 0.99, 
        color="rgba(246, 70, 93, 0.2)", 
        border_color="#f6465d",
        text="Supply Zone"
    )

    # 8. Add Long Position (Risk/Reward Tool)
    # Set a trade from the last close, with SL at the local low and TP at 2x RR
    entry_price = p_end
    stop_loss = p_low
    take_profit = entry_price + (entry_price - stop_loss) * 1
    
    chart.create_long_position(
        t_end, entry_price, stop_loss, take_profit,
        quantity=1.0,
        visible=True
    )
    
    # 9. Add Short Position (Projected from local high)
    chart.create_short_position(
        t_high, p_high, p_high * 1.05, p_high * 0.90,
        quantity=2.0,
        visible=True
    )

    # 10. Add Ray (Extended trendline)
    chart.create_ray(t_low, p_low, t_start, p_start, color="#ff9800", width=2)

    # 11. Add Horizontal Line (Global Resistance)
    chart.create_horizontal_line("main", max_price := df['high'].max(), color="#f44336")

    # 12. Add Static Markers (Annotations)
    ch1.add_marker(time=t_low, position="belowBar", color="#2196F3", shape="arrowUp", text="SIGNAL BUY")
    ch1.add_marker(time=t_high, position="aboveBar", color="#FF5252", shape="arrowDown", text="SIGNAL SELL")

    # 13. UI Customization & Notifications
    chart.set_watermark({"text": "PRO DEMO", "color": "rgba(255, 255, 255, 0.05)"})
    chart.show_notification("All Drawing Tools Loaded!", "success")
    
    print("✅ All available tools (Trend, Fib, Box, Pos, Ray, Horz, Markers) initialized.")
    chart.show()

if __name__ == "__main__":
    test_chart()
