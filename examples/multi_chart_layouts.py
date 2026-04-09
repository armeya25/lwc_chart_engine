import time
import datetime
import polars as pl
import numpy as np
from chart_engine import Chart

def generate_sample_data(num_bars=100, trend=1.0):
    start = datetime.datetime(2023, 1, 1)
    return pl.DataFrame({
        "time": [start + datetime.timedelta(days=i) for i in range(num_bars)],
        "open": np.linspace(100, 100 * trend, num_bars) + np.random.randn(num_bars) * 5,
        "high": np.linspace(110, 110 * trend, num_bars) + np.random.randn(num_bars) * 5,
        "low": np.linspace(90, 90 * trend, num_bars) + np.random.randn(num_bars) * 5,
        "close": np.linspace(100, 100 * trend, num_bars) + np.random.randn(num_bars) * 5,
    })

def run_multi_chart_demo():
    # Initialize main chart
    chart = Chart(title="Multi-Chart Layout Demo")

    ## disable sync
    chart.set_sync(True)

    # 1. Change Layout to "1P2" (1 Primary, 2 Secondary)
    # This returns a list of SubChart objects
    subcharts = chart.set_layout("1p2")
    
    print(f"Created {len(subcharts)} charts.")
    
    # Generate different data for each chart
    data_main = generate_sample_data(100, 1.5)
    data_sub1 = generate_sample_data(100, 0.8)
    data_sub2 = generate_sample_data(100, 1.2)
    
    # 2. Add data to the main chart (already in subcharts[0] but also chart.series["main"] works)
    main_series = subcharts[0].create_candlestick_series("Main Assets")
    main_series.set_data(data_main)
    
    # 3. Add data to the secondary charts
    chart1_series = subcharts[1].create_candlestick_series("Overlay Index")
    chart1_series.set_data(data_sub1)
    
    chart2_series = subcharts[2].create_candlestick_series("Relative Strength")
    chart2_series.set_data(data_sub2)
    
    # 4. Enable crosshair synchronization
    chart.set_sync(False)
    
    # 5. Add notifications to show interaction
    chart.show_notification("Layout synchronized!", "success")
    
    chart.show()

if __name__ == "__main__":
    run_multi_chart_demo()
