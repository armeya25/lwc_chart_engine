
import os
import polars as pl
from chart_engine.chart import Chart

def verify_v0_9_8():
    """
    Verification script for LWC Chart Engine v0.9.8.
    Showcases:
    - Tightened margins (5% default)
    - Dynamic margin control (set_price_margins)
    - New indicators (Bollinger Bands, MFI, ROC)
    - Centered Trade Panel
    """
    print("🚀 Verifying LWC Chart Engine v0.9.8...")
    
    # 1. Load data
    parquet_path = "data/1d.parquet"
    if not os.path.exists(parquet_path):
        print(f"Error: {parquet_path} not found. Please ensure data/1d.parquet exists.")
        return
        
    df = pl.read_parquet(parquet_path).head(1000)
    print(f"✅ Loaded {len(df)} candles.")
    
    # 2. Initialize Chart
    chart = Chart(title="LWC Chart Engine v0.9.8 Verification")
    
    # 3. Create Series and Configure Margins
    series = chart.create_candlestick_series("Main Series")
    
    # --- SHOWCASE: Dynamic Margins ---
    # Setting tight margins (1% top/bottom) for a "pro" look
    print("📏 Applying tight price scale margins (1% top/bottom)...")
    chart.set_price_margins(0.01, 0.01)
    
    # 4. Add Modern Indicators
    print("📈 Adding v0.9.8 Indicator Suite...")
    series.add_bollinger_bands(period=20, deviation=2.0)
    series.add_sma(period=50)
    series.add_sma(period=200)
    
    # Oscillators in separate panes
    series.add_rsi(period=14)
    series.add_mfi(period=14)
    series.add_roc(period=14)
    
    # 5. Set Data
    series.set_data(df)
    
    # 6. Final UI Tweaks
    chart.set_legend_visibility(True)
    # Positioning the window and showing it
    print("✨ Window Opening... Verify centered Trade Panel and lean Legend UI.")
    chart.show()

if __name__ == "__main__":
    verify_v0_9_8()
