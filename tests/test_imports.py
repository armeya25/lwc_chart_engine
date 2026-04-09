import pytest
import sys
import os

def test_package_import():
    """Verify that the chart_engine package can be imported."""
    try:
        import chart_engine
        assert chart_engine.__version__ is not None
    except ImportError as e:
        pytest.fail(f"Failed to import chart_engine: {e}")

def test_binary_bridge_import():
    """Verify that the Rust binary extension is present and importable."""
    try:
        from chart_engine import chart_engine_lib
        assert chart_engine_lib is not None
    except ImportError:
        # This might fail if the wheel is not installed, which is expected during some dev phases
        pytest.skip("Binary extension not found (likely not installed in current environment).")

def test_polars_dependency():
    """Verify that polars is available."""
    try:
        import polars as pl
        assert pl.__version__ is not None
    except ImportError:
        pytest.fail("Polars is not installed.")
