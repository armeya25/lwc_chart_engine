# Streamer Rust Bridge

Highly optimized Rust-based streaming engine with Python bindings (via PyO3).

## 🚀 Speed Optimized Build (Native CPU Support)
- [x] build time ~5-10 minutes depending on system resources

To build a version of the `streamer` module that's specifically tuned for your machine's processor architecture, follow the steps below.

### 1. Prerequisites (Fresh System Install)

If you're on a brand new system, you'll need the following installed:

**Install Rust & Cargo:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Setup Python Virtual Environment:**
It is highly recommended to build inside a clean virtual environment to avoid conflicts.
```bash
# Install venv if on Ubuntu/Debian
sudo apt update && sudo apt install -y python3-venv

# Create and activate venv
python3 -m venv .venv
source .venv/bin/activate
```

**Install Maturin (The Build Tool):**
```bash
pip install --upgrade pip
pip install maturin
```

### 2. Build for Native Performance

This command tells the Rust compiler to use all available CPU instructions (SIMD like AVX/AVX2/AVX-512, etc.) specific to your hardware. This can significantly improve performance for data-heavy tasks.

```bash
RUSTFLAGS="-C target-cpu=native" maturin build --release
```

The resulting `.whl` (Python Wheel) will be located in:
`target/wheels/`

### 3. Installation

After building, you can install the wheel into your current Python environment:

```bash
# If using a virtual environment
pip install target/wheels/streamer-*.whl --force-reinstall
```

---

### 🛠️ Key Performance Features (in Cargo.toml)

The current configuration is tuned for maximum throughput:
- **`opt-level = 3`**: Maximum compiler optimizations.
- **`lto = true`**: Link-time optimization (further cross-crate code slimming).
- **`codegen-units = 1`**: Allows the compiler to look at everything together for deeper optimization.
- **`panic = "abort"`**: Streamlines binary logic for faster execution.
