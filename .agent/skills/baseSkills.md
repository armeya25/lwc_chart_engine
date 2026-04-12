# Base Repository Skills

This document outlines core operational guidelines for AI agents working in this repository.

## Python Environment Management
- **Tooling**: Always use `uv` for all Python package installations and dependency management.
- **Environment**: Always use the `.venv` directory for the local Python virtual environment.

## Frontend Synchronization
- **HTML Integrity**: Whenever changes are made to `src/src-frontend/index.html`, ensure corresponding changes are also applied to `src/src-frontend/index.dist.html`. This ensures that both development and production templates remain synchronized.

## Architecture & Implementation
- **Rust First**: Always prioritize implementing core logic, data processing, and state coordination in the Rust backend (`src/src-backend/`). 
- **Minimize Python Logic**: Use the Python wrapper (`src/chart_engine/`) primarily as a thin orchestration layer and public API. Avoid complex coordination in Python to minimize redundant cross-language round-trips (e.g., Rust -> Python -> Rust).
