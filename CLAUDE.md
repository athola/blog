# Quick Command Reference

This file provides a summary of the most common development commands. For complete details on the project architecture, setup, and workflows, refer to `README.md`.

## Common Commands

-   **Run development server**: `make watch`
    -   Starts the Axum backend, SurrealDB, and the Leptos frontend with live reload.
    -   Application is available at `http://127.0.0.1:3007`.

-   **Run all checks**: `make validate`
    -   Executes formatting, linting, tests, and security scans. This should be run before committing.

-   **Run tests**: `make test`
    -   Runs the full suite of integration and unit tests.

-   **Run security scan**: `./scripts/run_secret_scan.sh`
    -   Scans the codebase for hardcoded secrets and credentials.

See the `Makefile` for a full list of available commands.
