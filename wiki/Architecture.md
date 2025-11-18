# Architecture

This document explains the architecture of the Rust-based blog engine.

## System Diagram

```
┌─────────────────┐    HTTP     ┌─────────────────┐
│   Browser/WASM  │ ◄─────────► │   Axum Server   │
│   (Leptos)      │             │   + Leptos SSR   │
└─────────────────┘             └─────────────────┘
                                       │
                                       │ SurrealDB
                                       ▼
                              ┌─────────────────┐
                              │  SurrealDB 3.0  │
                              │   (Database)    │
                              └─────────────────┘
```

## Core Components

### 1. Frontend: Leptos (WASM)

-   **Location**: `app/`, `frontend/`
-   **Role**: Renders the UI in the browser using WebAssembly. It handles client-side routing and interactivity, and hydrates the HTML rendered by the server.
-   **Key Crates**: `leptos`, `leptos_router`.

### 2. Backend: Axum

-   **Location**: `server/`
-   **Role**: An Axum web server that handles all incoming HTTP requests. It serves static files (CSS, JS), runs the API, and server-renders the initial HTML for fast page loads.
-   **Key Crates**: `axum`, `tokio`, `surrealdb`.

### 3. Database: SurrealDB

-   **Location**: `migrations/`
-   **Role**: A SurrealDB instance that stores all data (posts, comments, etc.). The schema is managed via `.surql` migration files.
-   **Key Features**: Chosen for its flexible schema, built-in authentication, and real-time query features.

## Key Design Decisions

### Why Full-Stack Rust?

A monolithic, full-stack Rust application was chosen over a separate frontend and backend for a few key reasons:
-   **End-to-End Type Safety**: The `Post` struct is defined once and used by the database, backend, and frontend. This makes it impossible for the API contract to drift, which was a recurring problem with the previous Node.js/React architecture.
-   **Simpler Toolchain**: The entire project uses Cargo. There is no `package.json`, no Node.js, and no need to coordinate two different package managers.
-   **Performance**: Server-side rendering with Leptos results in a fast initial page load (~200ms FCP) and a small client-side footprint (~150KB gzipped WASM).

### Why SurrealDB?

SurrealDB was chosen over PostgreSQL for its modern feature set, despite being pre-release software.
-   **Real-Time Queries**: Live queries are a built-in feature, which means no separate WebSocket or messaging system is needed for features like real-time notifications.
-   **Row-Level Security**: The database has its own permissions system, which simplifies authorization logic in the application code.
-   **The Trade-off**: Using an alpha version (`3.0.0-alpha.10`) is a calculated risk. The benefit of the modern feature set was deemed worth the potential for instability.

### Activity Feed & Newsletter Sync

-   **Deterministic IDs**: The `post_activity` SurrealDB event creates activity records with IDs derived from the post slug (e.g., `activity:post:<slug>`). The event uses `type::record("activity", ...)` so that re-running migrations produces the same identifiers.
-   **Reasoning**: A downstream newsletter sync job deduplicates updates by record ID. Stable IDs prevent staging refreshes from resending old posts and allow integration tests to assert on predictable IDs.
-   **Extensibility**: While the automated process is deterministic, manual activity seeds can override the ID by supplying a `RecordId` via the Rust client. This is used for backfills or special campaigns. To match the server-generated format in Rust code, call `Activity::deterministic_post_id("my-slug")`.

### Three-Tier Testing Architecture

The test suite is split into three tiers to balance speed with coverage. Each tier runs on a different cadence:

-   **Unit Tests (~2s)**: Run on every local save. They test individual functions in isolation with no network or database access.
-   **CI Tests (~8s)**: Run on every pull request. This is a subset of integration tests using an in-memory database to quickly catch breaking changes.
-   **Integration Tests (~44s)**: Run on merges to the `main` branch. These are full workflow tests against a real SurrealDB instance.

This structure keeps the pull request feedback loop fast (~8 seconds) while ensuring full test coverage on the `main` branch.

## Performance Optimizations

### Frontend

-   **Code Splitting**: Route-specific components are loaded on demand. For example, the contact form logic is only loaded when a user navigates to the `/contact` page.
-   **CSS Purging**: TailwindCSS removes all unused styles from the final CSS bundle, reducing its size.
-   **Asset Optimization**: Images are converted to modern formats like WebP and served with appropriate cache headers.

### Backend

-   **Connection Pooling**: The server maintains a pool of ready database connections. Reusing connections avoids the latency of establishing a new one for each incoming request.
-   **Response Caching**: Blog posts are cached in memory for 5 minutes after being fetched. Subsequent requests for the same post are served from the cache, resulting in a near-instant response and fewer database queries.
-   **Asynchronous I/O**: The entire backend is built on `tokio`, an asynchronous Rust runtime. This ensures that no threads are blocked waiting for database queries or network requests, allowing the server to handle many concurrent users with minimal resources.

### Database

Strategic indexes have been added to improve query performance based on common access patterns.

```surql
-- For fetching posts by slug (most common query)
DEFINE INDEX slug_idx ON TABLE post COLUMNS slug UNIQUE;

-- For fetching recent posts on the homepage
DEFINE INDEX published_idx ON TABLE post COLUMNS published_at DESC;
```

Adding the `published_idx` index, for example, reduced the homepage load time by avoiding a full table scan.
