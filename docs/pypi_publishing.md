# 🚀 Publishing to PyPI

This guide explains how to package and publish the `chart_engine` project to the Python Package Index (PyPI).

## 🛠 Prerequisites

1.  **PyPI Account**: Register at [pypi.org](https://pypi.org).
2.  **API Token**: Create an API token in your PyPI account settings.
3.  **Maturin**: Ensure you have `maturin` installed (`pip install maturin`).

---

## 🏗 Recommended: Automated Publishing (GitHub Actions)

The most reliable way to publish this project is via GitHub Actions. Since the project contains a Rust backend, it requires specific build environments for Windows, macOS, and Linux.

### Setup Steps:
1.  **Enable Trusted Publishing**:
    *   Go to your [PyPI Project Settings](https://pypi.org/manage/project/chart_engine/settings/publishing/).
    *   Select **"Add a Trusted Publisher"**.
    *   Choose **GitHub Actions**.
    *   **Owner**: `armeya` (or your GitHub username/organization).
    *   **Repository**: `lwc_chart_engine`.
    *   **Workflow Name**: `build_wheels.yml`.
    *   **Environment**: (You can leave this blank).
2.  **Push a Tag**: When you are ready to release, update the version in `pyproject.toml` and `src/src-tauri/Cargo.toml`, then push a git tag:
    ```bash
    git tag v0.3.6
    git push origin v0.3.6
    ```
3.  **Automatic Build & Publish**: The `build_wheels.yml` workflow will automatically build wheels for all platforms, commit them to the `wheels/` directory, and publish them to PyPI using modern OIDC authentication (no secrets required).

---

## 💻 Manual Publishing (Local)

If you prefer to publish manually from your local machine, follow these steps.

### 1. Build the Wheels
Maturin can build the wheel for your current platform:
```bash
maturin build --release
```

### 2. Upload to PyPI
You can use `maturin publish` which handles both building and uploading:
```bash
maturin publish
```
*You will be prompted for your PyPI API token.*

> [!WARNING]
> Manual publishing from a single machine will only upload the wheel for your current OS. Users on other operating systems will not be able to install the package unless they have the Rust toolchain installed and build from source. **Use the GitHub Actions workflow for production releases.**

---

## 🧪 Testing on TestPyPI

Before publishing to the real PyPI, it is highly recommended to test on TestPyPI:

1.  Register at [test.pypi.org](https://test.pypi.org).
2.  Publish using:
    ```bash
    maturin publish --repository testpypi
    ```

---

## 📝 Version Management

Ensure the version strings match in both configuration files before publishing:
1.  **`pyproject.toml`**: `version = "x.y.z"`
2.  **`src/src-tauri/Cargo.toml`**: `version = "x.y.z"`
