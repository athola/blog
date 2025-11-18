# Claude Quick Start

This is a token-optimized guide for working with this repository. For a full explanation, see the [Development Workflow Guide](wiki/Development-Workflow.md).

## Core Commands

-   **`make watch`**: Starts the dev server, DB, and live reload. App is at `http://127.0.0.1:3007`.
-   **`make validate`**: Runs all quality checks (format, lint, test, security). Use this before committing.
-   **`make test`**: Runs the full test suite.
-   **`./scripts/run_secret_scan.sh`**: Runs the security scan for secrets.

## Project Structure

-   `app/`: Shared Leptos components and types.
-   `server/`: Axum backend.
-   `frontend/`: WASM frontend.
-   `tests/`: Integration tests.

For more details on architecture, commands, and troubleshooting, please read the [full development guide](wiki/Development-Workflow.md).
