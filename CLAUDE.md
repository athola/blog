# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `make watch` - Start the development server with live reload and database
- `make build` - Build the project
- `make build-release` - Build for production
- `make test` - Run all tests
- `make lint` - Run clippy linting
- `make format` - Format Rust code

### Package Management
- `make install-pkgs` - Install required Cargo tools (cargo-make, cargo-audit, cargo-bloat, cargo-leptos, etc.)
- `make upgrade` - Update all dependencies

### Quality Assurance
- `make security` - Run security audit
- `make outdated` - Check for outdated dependencies
- `make udeps` - Check for unused dependencies
- `make machete` - Check for unused crates
- `make spellcheck` - Check documentation spelling

## Architecture

This is a Rust-based blog engine with the following structure:

### Workspace Layout
- **app/**: Shared application logic and components (Leptos components, routing, API types)
- **frontend/**: WASM frontend entry point for hydration
- **server/**: Axum web server with SSR support
- **markdown/**: Markdown processing utilities
- **migrations/**: Database migration definitions

### Technology Stack
- **Leptos**: Full-stack Rust web framework for UI components and SSR
- **Axum**: Backend web server framework
- **SurrealDB**: Database layer
- **Tailwind CSS**: Styling (configured via tailwind.css in style/)
- **cargo-leptos**: Build tool for Leptos applications

### Key Files
- `Cargo.toml`: Workspace configuration with leptos metadata
- `Makefile`/`Makefile.toml`: Build system (use `make` commands, not `cargo make` directly)
- `leptosfmt.toml`: Leptos-specific formatting configuration
- `db.sh`: Database startup script (auto-runs with `make watch`)

### Development Workflow
1. Use `make watch` to start development (starts database and leptos watch automatically)
2. The site runs on 127.0.0.1:3007 with live reload on port 3001
3. Frontend code compiles to WASM, server runs with SSR
4. CSS is processed through Tailwind and served as `/pkg/blog.css`

### Leptos Configuration
- Site root: `target/site`
- Assets: `public/` directory
- Tailwind input: `style/tailwind.css`
- WASM release profile: `wasm-release` (optimized for size)
- Development environment uses debug assertions, production uses optimized builds