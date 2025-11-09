# Development Workflow

This guide covers the process for building, testing, and working on the blog engine. It includes setup instructions, project structure, and a reference for common commands.

## Quick Start

### Prerequisites

-   Rust (latest stable)
-   WASM target: `rustup target add wasm32-unknown-unknown`
-   [SurrealDB 3.0.0-alpha.10](https://surrealdb.com/install)
-   Required cargo tools: `make install-pkgs`

### First-Time Setup

```bash
# 1. Clone the repository
git clone https://github.com/athola/blog.git
cd blog

# 2. Install required tools like cargo-leptos and cargo-make
make install-pkgs

# 3. Download and install the correct SurrealDB version
make install-surrealdb

# 4. Create a local .env file from the example
cp .env.example .env

# 5. Start the database and run initial schema migrations
make init-db
```

### Local Development

To start the development server, run the following command:

```bash
# This command starts the database, backend server, and frontend with live reload.
make watch
```

The application will then be available at `http://127.0.0.1:3007`.

## Project Structure

The workspace is organized to separate concerns.

-   `app/`: The core of the application, containing shared logic, Leptos components, routing, and API types used by both the frontend and backend.
-   `server/`: The Axum web server that handles API requests and server-side rendering (SSR).
-   `frontend/`: The entry point for the client-side WebAssembly (WASM) application.
-   `markdown/`: A utility crate for processing Markdown into HTML.
-   `migrations/`: Contains all `.surql` files for the database schema.
-   `tests/`: Integration and end-to-end tests.

## Command Reference

The project uses `make` to automate common tasks.

### Core Commands

-   `make watch`: The main command for development. Starts all services with live reload.
-   `make test`: Runs the full test suite using `nextest`.
-   `make validate`: A comprehensive check that runs formatting, linting, tests, and a security scan. This is recommended before committing changes.

### Testing

-   `make test-unit`: Runs only the unit tests.
-   `make test-ci`: Runs the test suite optimized for the CI environment.
-   `make test-coverage`: Calculates test coverage using `cargo-llvm-cov`.

### Database

-   `make init-db`: Initializes the database for the first time.
-   `make reset-db`: A destructive command that stops the database, deletes all data, and restarts it. Useful for achieving a clean state.
-   `make start-db` / `make stop-db`: Manually start or stop the SurrealDB container.

### Code Quality

-   `make format`: Formats the code using `cargo fmt`.
-   `make lint`: Lints the code with `clippy`.
-   `./run_secret_scan.sh`: Manually triggers the secret scanning script that runs in CI.

## CI/CD Pipeline

The project uses a security-first CI/CD pipeline in GitHub Actions.

1.  **Secret Scan:** The `secrets-scan.yml` workflow runs first to check for hardcoded secrets. A failure here will stop the entire run.
2.  **Build & Test:** If the secret scan passes, the `rust.yml` workflow builds the code, runs tests, and checks for formatting and linting errors.
3.  **Deploy:** On the `master` branch, if all previous steps pass, the `deploy.yml` workflow deploys the new version to DigitalOcean.

This process is designed to prevent build errors or exposed secrets from reaching the production environment.

## Troubleshooting

-   **Build failures:** Run `cargo clean && make build`. If the issue persists, run `make install-pkgs` to ensure all development tools are up to date.
-   **Database issues:** `make reset-db` can resolve many issues by providing a clean database. Also, verify that the installed SurrealDB version is `v3.0.0-alpha.10`.
-   **Lingering processes:** If services do not shut down correctly, they can be stopped manually with `pkill -f surreal` or `pkill -f server`.
