# Development Workflow

This guide covers common workflows for building, testing, and contributing to the blog engine.

## Local Development Setup

The initial setup process is documented in the `README.md` file. Before proceeding, follow the instructions in the **"Quick Start"** section of the main `README.md`.

Once initial setup is complete, use this primary command for development:

```bash
# Start the development server, database, and live reload
make watch
```

The application will be available at `http://127.0.0.1:3007`.

## Common Development Tasks

The `Makefile` automates most development tasks.

### Running Checks and Tests

-   **`make validate`**: Runs all quality checks: formatting, linting, tests, and security scans. Run before committing any changes.
-   **`make test`**: Runs the full test suite, including unit and integration tests.
-   **`make test-unit`**: Runs only the fast unit tests.
-   **`make test-coverage`**: Calculates and displays test coverage.

### Managing the Database

-   **`make init-db`**: Initializes a clean database with the latest schema.
-   **`make reset-db`**: **Deletes all data** and restarts the database container for a fresh start.
-   **`make start-db` / `make stop-db`**: Manually start or stop the SurrealDB Docker container.

### Code Quality and Security

-   **`make format`**: Formats all Rust code according to the project's style.
-   **`make lint`**: Lints the codebase to check for common issues.
-   **`./scripts/run_secret_scan.sh`**: Runs a comprehensive scan to detect any hardcoded secrets or credentials.

### Data Migration

-   **`./scripts/backfill_activity_ids.sh`**: A helper script to normalize activity record IDs in the database. This is typically only needed when working with older data. See the script for more details.

## CI/CD Pipeline

The GitHub Actions pipeline prioritizes security:

1.  **Secret Scan (`secrets-scan.yml`)**: The pipeline first scans for hardcoded secrets. A failure at this stage immediately blocks the workflow.
2.  **Build & Test (`rust.yml`)**: If the secret scan passes, the pipeline proceeds to build the project, run all tests, and perform formatting and linting checks.
3.  **Deploy (`deploy.yml`)**: If all previous steps succeed, a push to the `master` branch triggers an automatic deployment to DigitalOcean.

This sequence ensures that code with build errors, failing tests, or exposed secrets is never deployed.

## Troubleshooting

-   **Build Failures**: If the build fails, try `cargo clean && make build`. If the problem persists, `make install-pkgs` ensures build tools are updated.
-   **Database Issues**: Resolve database problems with `make reset-db` for a fresh start. Confirm you are running SurrealDB `3.0.0-alpha.10`.
-   **Lingering Processes**: If a service does not shut down correctly, you may need to terminate it manually. Use `pkill -f surreal` to stop the database or `pkill -f server` to stop the backend server.
