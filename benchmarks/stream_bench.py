import time
import polars as pl
import random
from chart_engine import Chart

def run_benchmark(num_rows=19_000):
    print(f"🚀 Starting Performance Benchmark: {num_rows:,} rows")
    
    # 1. Generate dataset with Polars and standard random
    start_gen = time.time()
    dates = pl.date_range(
        start=pl.datetime(2020, 1, 1),
        end=pl.datetime(2020, 1, 1) + pl.duration(minutes=num_rows - 1),
        interval="1m",
        eager=True
    )
    
    df = pl.DataFrame({
        "time": dates,
        "open": [random.uniform(100, 200) for _ in range(num_rows)],
        "high": [random.uniform(200, 300) for _ in range(num_rows)],
        "low": [random.uniform(0, 100) for _ in range(num_rows)],
        "close": [random.uniform(100, 200) for _ in range(num_rows)],
    })
    gen_time = time.time() - start_gen
    print(f"✅ Data Generation (Polars): {gen_time:.4f}s")

    # 2. Initialize Chart Engine
    # Note: We use a headless mode simulation if possible, but here we just test data push latency
    chart = Chart(title="Performance Bench")
    
    # 3. Measure Data Push Latency
    start_push = time.time()
    chart.set_data(df)
    push_time = time.time() - start_push
    
    print(f"✅ Data Push Latency: {push_time:.4f}s")
    print(f"📈 Throughput: {num_rows / push_time:,.0f} rows/sec")
    
    # Keep window open for a bit to ensure async tasks finalize
    time.sleep(2)
    chart.show()

if __name__ == "__main__":
    try:
        run_benchmark()
    except Exception as e:
        print(f"❌ Benchmark failed: {e}")
        print("Note: This benchmark requires the chart_engine package to be installed.")
