import time
import datetime
import polars as pl
import numpy as np
from chart_engine import Chart

def generate_sample_data(num_bars=100):
    start = datetime.datetime.now() - datetime.timedelta(days=num_bars)
    return pl.DataFrame({
        "time": [start + datetime.timedelta(days=i) for i in range(num_bars)],
        "open": np.cumsum(np.random.randn(num_bars)) + 100,
        "high": np.cumsum(np.random.randn(num_bars)) + 105,
        "low": np.cumsum(np.random.randn(num_bars)) + 95,
        "close": np.cumsum(np.random.randn(num_bars)) + 100,
    })

def run_ui_demo():
    # 1. Initialize with custom title
    chart = Chart(title="State-of-the-art UI Customization")
    
    df = generate_sample_data(200)
    chart.series["main"].set_data(df)
    
    # 2. Set Watermark (Branding)
    chart.set_watermark("testing watermark")
    
    # 3. Timezone Management
    chart.set_timezone("Asia/Kolkata")
    
    # 4. Toggle UI Components
    print("Customizing UI components...")
    chart.enable_tooltip()          # Show floating price info on crosshair
    chart.enable_layout_toolbar()   # Show the side layout selection menu
    chart.set_legend_visibility(True)
    
    # 5. Set Crosshair Mode (Magnet mode for snapping to bars)
    chart.set_crosshair_mode(1) 
    
    # 6. Change Legend / Data Context
    chart.set_timeframe({"label": "1D", "value": 1440})
    
    # 7. Take an automated screenshot (Saved to project root)
    # We move this to a thread so it doesn't block the main UI loop
    import threading
    def take_delayed_screenshot():
        time.sleep(3)  # Give time for the window to settle
        chart.show_notification("Auto-capturing UI snapshot...", "info")
        time.sleep(1)
        chart.take_screenshot()
        print("📸 UI snapshot saved to project root.")

    threading.Thread(target=take_delayed_screenshot, daemon=True).start()
    
    print("UI customized. Hover over the chart to see the magnet crosshair and tooltips.")
    chart.show()

if __name__ == "__main__":
    run_ui_demo()
