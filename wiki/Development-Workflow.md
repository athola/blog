# Development Workflow

A guide for building, testing, and working on the blog engine.

## Setup

### Prerequisites

-   Rust (latest stable)
-   WASM target: `rustup target add wasm32-unknown-unknown`
-   SurrealDB `v3.0.0-alpha.10`
-   Cargo tools: `make install-pkgs`

### Initial Setup

```bash
# 1. Clone the repository and navigate into it
git clone https://github.com/athola/blog.git && cd blog

# 2. Install build tools (e.g., cargo-leptos)
make install-pkgs

# 3. Install the correct SurrealDB version
make install-surrealdb

# 4. Create a local environment file
cp .env.example .env

# 5. Initialize the database
make init-db
```

### Running the Development Server

```bash
# Start the database, backend, and frontend with live reload
make watch
```

The application will be available at `http://127.0.0.1:3007`.

## Project Structure

-   `app/`: Shared application logic (Leptos components, routing, API types).
-   `server/`: Axum backend server (API, SSR).
-   `frontend/`: WASM frontend entry point.
-   `markdown/`: Markdown-to-HTML conversion utility.
-   `migrations/`: Database schema migrations (`.surql` files).
-   `tests/`: Integration tests.

## Makefile Commands

Common tasks are automated with `make`.

### Core

-   `make watch`: Start all services with live reload.
-   `make test`: Run the full test suite.
-   `make validate`: Run all checks (format, lint, test, security). Use before committing.

### Testing
-   `make test-unit`: Run only unit tests.
-   `make test-ci`: Run the CI-optimized test suite.
-   `make test-coverage`: Calculate test coverage.

### Database
-   `make init-db`: Initialize the database.
-   `make reset-db`: Delete all data and restart the database.
-   `make start-db` / `make stop-db`: Start or stop the database container.

### Code Quality
-   `make format`: Format the code.
-   `make lint`: Lint the code.
-   `./scripts/run_secret_scan.sh`: Run the secret scanning script.

## CI/CD Pipeline

The GitHub Actions pipeline is designed to prioritize security.

1.  **Secret Scan (`secrets-scan.yml`):** Checks for hardcoded secrets. A failure blocks the pipeline.
2.  **Build & Test (`rust.yml`):** Builds the project, runs tests, and checks formatting and linting.
3.  **Deploy (`deploy.yml`):** On a `master` branch push, deploys the application to DigitalOcean if all previous steps pass.

This sequence ensures that no code with build errors or exposed secrets is deployed.

## Troubleshooting

-   **Build Failures**: Try `cargo clean && make build`. If that doesn't work, run `make install-pkgs` to update the build tools.
-   **Database Issues**: `make reset-db` will reset the database to a clean state. Also, confirm you are running SurrealDB `v3.0.0-alpha.10`.
-   **Lingering Processes**: If a service doesn't shut down, stop it manually with `pkill -f surreal` or `pkill -f server`.
