import time
import datetime
import polars as pl
import numpy as np
from chart_engine import Chart

def generate_live_data(n=200):
    start = datetime.datetime.now() - datetime.timedelta(hours=n)
    times = [start + datetime.timedelta(hours=i) for i in range(n)]
    
    # Generate a realistic random walk for the base price
    base_prices = 100 + np.cumsum(np.random.randn(n) * 0.5)
    
    data = []
    for p in base_prices:
        # Create valid OHLC for each point
        o = p + np.random.randn() * 0.1
        c = p + np.random.randn() * 0.1
        h = max(o, c) + abs(np.random.randn() * 0.2)
        l = min(o, c) - abs(np.random.randn() * 0.2)
        data.append({"open": o, "high": h, "low": l, "close": c})
        
    df = pl.DataFrame(data)
    df = df.insert_column(0, pl.Series("time", times))
    return df

def run_trading_demo():
    print("🚀 Starting Paper Trading Demo...")
    # Initialize chart with 'main' series tracked for auto-price sync
    chart = Chart(title="Programmatic Trading & TP/SL Visuals")
    
    df = generate_live_data(500)
    
    # Explicitly create the candlestick series to ensure it renders in the UI
    main = chart.create_candlestick_series("Main Series")
    
    # Sync the chart's main series tracker for auto-price updates
    chart.main_series_id = main.series_id
    
    # set_data will now automatically update the trader's last_price to the final close in df
    print("Feeding initial market data...")
    main.set_data(df)
    
    # 1. Create a Long Position with visual TP/SL tools
    # This now automatically synchronizes with the PaperTrader backend
    entry_price = float(df["close"][-1])
    sl_price = entry_price * 0.98   # 2% stop
    tp_price = entry_price * 1.01   # 1% target
    
    print(f"Opening visual Long Position at {entry_price:.2f}...")
    chart.create_long_position(
        df["time"][-1], 
        entry_price, 
        sl_price, 
        tp_price, 
        quantity=1.0,
        text="Demo Entry"
    )
    
    chart.show_notification("Sync Trade Executed", "success")
    
    print("Simulating real-time market ticks (Watch the P/L and TP/SL)...")
    current_price = entry_price
    try:
        # Run for a bit longer to allow user to see the updates
        for i in range(100):
            current_price += np.random.randn() * 0.2
            
            # This pushes the price to the Rust engine, checks stops, and updates UI
            chart.trader_update_price(current_price) 
            
            if i % 10 == 0:
                print(f"Tick {i}: Price @ {current_price:.2f}")
                
            time.sleep(0.5)
    except KeyboardInterrupt:
        pass
        
    print("\nDemo complete. Inspect the 'Active Positions' and 'History' tabs in the UI.")
    chart.show()

if __name__ == "__main__":
    run_trading_demo()
