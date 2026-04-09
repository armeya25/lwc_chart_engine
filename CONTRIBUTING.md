# Contributing to LWC Chart Engine

Thank you for your interest in contributing to the LWC Chart Engine! We welcome all contributions, from bug reports and documentation updates to new features and performance optimizations.

## 🚀 Getting Started

1. **Fork the Repository**: Create a fork of the project on GitHub.
2. **Clone Locally**:
   ```bash
   git clone https://github.com/your-username/lwc_chart_engine.git
   cd lwc_chart_engine
   ```
3. **Setup Environment**:
   ```bash
   ./helpers/dev.sh
   ```

## 🛠 Development Workflow

- **Rust Backend**: Located in `src/src-tauri`. Use `cargo check` and `cargo fmt`.
- **Python API**: Located in `src/chart_engine`. Follow PEP 8 styles.
- **Frontend**: Located in `src/src-frontend`. We use `esbuild` for bundling.

## 🧪 Testing

Before submitting a pull request, please ensure all tests pass:
```bash
pytest
```

## 📝 Pull Request Process

1. Create a new branch for your feature or fix.
2. Document your changes in the `docs/changelog.md`.
3. Submit a Pull Request targeting the `main` branch.
4. Ensure the CI/CD pipeline passes.

---
*maintained by amit vaidya*
