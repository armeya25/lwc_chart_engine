import os
import time
import datetime
import polars as pl
import numpy as np
import threading
from chart_engine import Chart

def run_ui_demo():
    # 1. Load Data
    parquet_path = "data/1d.parquet"
    if not os.path.exists(parquet_path):
        print(f"Error: {parquet_path} not found.")
        return
        
    df = pl.read_parquet(parquet_path)
    df = df.tail(100)

    # 2. Initialize and Show Chart
    chart = Chart(title="Chart Engine - SubChart Test")
    #chart.show()  # Launch the Tauri window
    
    # 3. Configure Layout and Series
    subcharts = chart.set_layout("single")
    ch1 = subcharts[0].create_candlestick_series(name="BTC/USD")
    print(f"Series created: {ch1}")
    
    # 4. Set Data
    ch1.set_data(df)
    print("✅ Data series set successfully. Window should be open.")
    

    # 2. Set Watermark (Branding)
    chart.set_watermark("ANTIGRAVITY v0.5.5")
    # 3. Timezone Management
    chart.set_timezone("Asia/Kolkata")
    # 4. Toggle UI Components
    print("Customizing UI components...")
    chart.enable_tooltip()          # Show floating price info on crosshair
    chart.set_legend_visibility(True)
    
    # 5. Set Crosshair Mode (Magnet mode for snapping to bars)
    chart.set_crosshair_mode(1) 
    
    
    # 7. Take an automated screenshot (Saved to project root)
    # We move this to a thread so it doesn't block the main UI loop
    def take_delayed_screenshot():
        time.sleep(3)  # Give time for the window to settle
        chart.show_notification("Auto-capturing UI snapshot...", "info")
        time.sleep(1)
        chart.take_screenshot(filename="ui_customization_snapshot.png")
        print("📸 UI snapshot saved to project root.")

    threading.Thread(target=take_delayed_screenshot, daemon=True).start()
    
    print("UI customized. Hover over the chart to see the magnet crosshair and tooltips.")
    chart.show()

if __name__ == "__main__":
    run_ui_demo()
