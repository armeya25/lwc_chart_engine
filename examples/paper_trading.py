import time
import datetime
import polars as pl
import numpy as np
from chart_engine import Chart

def generate_live_data(n=200):
    start = datetime.datetime.now() - datetime.timedelta(hours=n)
    return pl.DataFrame({
        "time": [start + datetime.timedelta(hours=i) for i in range(n)],
        "open": np.cumsum(np.random.randn(n)) + 100,
        "high": np.cumsum(np.random.randn(n)) + 105,
        "low": np.cumsum(np.random.randn(n)) + 95,
        "close": np.cumsum(np.random.randn(n)) + 100,
    })

def run_trading_demo():
    print("🚀 Starting Paper Trading Demo...")
    chart = Chart(title="Programmatic Trading & TP/SL Visuals")
    
    df = generate_live_data(500)
    main = chart.series["main"]
    main.set_data(df)
    
    # 1. Manually create a Long Position with visual TP/SL tools
    entry_price = float(df["close"][-1])
    sl_price = entry_price * 0.95
    tp_price = entry_price * 1.10
    
    print(f"Opening visual Long Position at {entry_price:.2f}...")
    chart.create_long_position(
        df["time"][-1], 
        entry_price, 
        sl_price, 
        tp_price, 
        quantity=0.5,
        text="Targeting 10% gain"
    )
    
    # 2. Programmatically execute a trade in the Rust backend
    # This will appear in the UI's position table
    print("Executing programmatic Buy order...")
    chart.trader_execute("buy", 1.0, price=entry_price, tp=tp_price, sl=sl_price, series=main)
    
    chart.show_notification("Trade Executed Programmatically", "success")
    
    # 3. Simulate price movement to trigger TP/SL in the backend
    # In a real app, you would call this on every new tick
    print("Simulating market ticks...")
    current_price = entry_price
    for i in range(50):
        current_price += np.random.randn() * 0.5
        chart.trader_update_price(current_price) # This updates the Rust backend and the UI PnL
        time.sleep(0.1)
        
    print("Demo complete. You can inspect the 'Active Positions' table in the UI.")
    chart.show()

if __name__ == "__main__":
    run_trading_demo()
