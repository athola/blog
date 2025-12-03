# Architecture

This document describes the architecture of the Rust-based blog engine.

## System Diagram

```
┌──────────────────┐     HTTP      ┌──────────────────┐
│   Browser (WASM) │ ◄───────────► │   Axum Server    │
│     (Leptos)     │               │ (with Leptos SSR)│
└──────────────────┘               └──────────────────┘
                                           │
                                           │ SurrealDB
                                           ▼
                                 ┌──────────────────┐
                                 │  SurrealDB 3.0   │
                                 │    (Database)    │
                                 └──────────────────┘
```

## Core Components

### 1. Frontend (Leptos + WASM)

-   **Location**: `app/` and `frontend/`
-   **Role**: Renders the UI in the browser via WebAssembly. It handles client-side routing, interactivity, and hydrates initial server-rendered HTML.
-   **Key Crates**: `leptos`, `leptos_router`.

### 2. Backend (Axum)

-   **Location**: `server/`
-   **Role**: An Axum web server that handles all incoming HTTP requests. It serves static assets, provides an API, and performs server-side rendering of the initial page load.
-   **Key Crates**: `axum`, `tokio`, `surrealdb`.

### 3. Database (SurrealDB)

-   **Location**: `migrations/` contains schema definitions.
-   **Role**: A SurrealDB instance for data persistence (posts, comments, etc.). The schema is managed through `.surql` migration files, leveraging its flexible data model, built-in access controls, and live query features.

## Architectural Decisions

### Monolithic Architecture with Full-Stack Rust

A monolithic, full-stack Rust application was chosen over a separate frontend and backend for three main reasons:
-   **End-to-End Type Safety**: Shared data structures (e.g., the `Post` struct) across the database, backend, and frontend eliminate API contract drift, a problem encountered with the previous Node.js/React architecture.
-   **Simplified Toolchain**: The entire project is managed with Cargo, avoiding the complexity of coordinating multiple package managers like `npm` and `cargo`.
-   **Performance**: Server-side rendering with Leptos provides a fast initial page load (~200ms First Contentful Paint) and a small client-side footprint (~150KB gzipped WASM).

### SurrealDB for Data Persistence

SurrealDB was selected over traditional databases like PostgreSQL due to its modern feature set, despite its pre-release status.
-   **Integrated Real-Time Queries**: Live queries are a built-in feature, eliminating the need for a separate WebSocket or messaging system for features like real-time notifications.
-   **Embedded Row-Level Security**: The database's native permission system simplifies authorization logic in the application layer.
-   **Calculated Risk**: Using an alpha version (`3.0.0-alpha.10`) is a deliberate trade-off. The benefits of its feature set were judged to outweigh the risk of potential instability.

### Deterministic IDs for Activity Feeds

-   **ID Generation**: The system generates deterministic IDs for activity records from the post slug (e.g., `activity:post:<slug>`). The generation logic includes normalization to handle variations, such as removing redundant `-post-` tokens.
-   **Rationale**: This approach supports a downstream newsletter sync job that deduplicates updates by record ID. Stable, predictable IDs prevent duplicate announcements and allow integration tests to make reliable assertions.
-   **Flexibility**: While new activities are normalized automatically via `Activity::deterministic_post_id`, manual backfills can still use custom `RecordId`s. The script at `scripts/backfill_activity_ids.sh` can be used to align older records with the new format.

### Layered Testing Strategy

The test suite employs a layered strategy with three distinct tiers, each with a different execution cadence, to balance rapid feedback and comprehensive coverage.

-   **Tier 1: Unit Tests (~2s)**
    -   **Scope**: Individual functions in isolation, with no network or database access.
    -   **Cadence**: Run automatically on every local file save.
-   **Tier 2: Fast Integration Tests (~8s)**
    -   **Scope**: A subset of integration tests using an in-memory database.
    -   **Cadence**: Run on every pull request to quickly catch breaking changes.
-   **Tier 3: Full Integration Tests (~44s)**
    -   **Scope**: Full end-to-end workflow tests against a real SurrealDB instance.
    -   **Cadence**: Run on merges to the `main` branch.

This layered approach provides a fast feedback loop for pull requests (~8 seconds) while ensuring complete test coverage is maintained on the `main` branch.

## Performance

Performance is a key consideration. The following optimizations have been implemented.

### Frontend

-   **Code Splitting**: Route-specific components are loaded on demand. For example, the logic for the contact form is only fetched when a user navigates to the `/contact` page.
-   **CSS Purging**: Unused CSS rules are removed from the final bundle by TailwindCSS, reducing its size.
-   **Asset Optimization**: Images are converted to modern formats like WebP and served with efficient cache policies.

### Backend

-   **Connection Pooling**: The server maintains a pool of database connections to avoid the latency of establishing a new connection for each request.
-   **In-Memory Caching**: Fetched blog posts are cached in memory for five minutes. Subsequent requests for a cached post are served instantly, reducing database load.
-   **Asynchronous I/O**: The entire backend is built on the `tokio` asynchronous runtime, allowing the server to handle many concurrent users efficiently.

### Database

Strategic indexes were added to improve query performance for common access patterns. For example, the `published_idx` was added to avoid a full table scan when fetching recent posts for the homepage.

```surql
-- For fetching a post by its slug (most common query)
DEFINE INDEX slug_idx ON TABLE post COLUMNS slug UNIQUE;

-- For fetching recent posts on the homepage
DEFINE INDEX published_idx ON TABLE post COLUMNS published_at DESC;
```
