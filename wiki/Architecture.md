# Architecture Overview

This document outlines the architecture of the Rust blog engine, including component interactions, data flow, and design rationale.

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

## Component Architecture

### 1. Frontend (Leptos + WASM)

**Location**: `app/`, `frontend/`

-   **Responsibilities**: Handles client-side routing, interactive UI components, and client-side hydration of server-rendered HTML. It is compiled to WebAssembly for performance.
-   **Key Crates**: `leptos`, `leptos_router`.

### 2. Backend (Axum + Leptos SSR)

**Location**: `server/`

-   **Responsibilities**: Handles HTTP requests, performs server-side rendering (SSR) with Leptos, implements API endpoints, manages database connections, and serves static assets.
-   **Key Crates**: `axum`, `tokio`, `surrealdb`.

### 3. Database (SurrealDB)

**Location**: `migrations/`

-   **Responsibilities**: Provides data persistence. The schema is defined in `.surql` migration files.
-   **Key Features**: Supports flexible data models, tiered authentication, and real-time queries.

## Design Rationale

### Full-Stack Rust with Leptos

The project is built as a single, full-stack Rust application rather than separate frontend and backend codebases. This approach was chosen for several key reasons:

-   **Type Safety**: Sharing types (e.g., the `Post` struct) between the frontend and backend eliminates an entire class of API contract errors at compile time, which would otherwise manifest as runtime errors.
-   **Reduced Complexity**: A single toolchain and language simplify the development and build process.
-   **Performance**: Leptos enables server-side rendering with client-side hydration, leading to fast initial page loads (~200ms First Contentful Paint) and small asset sizes (~150KB gzipped WASM).

### SurrealDB as the Database

SurrealDB was chosen over traditional relational databases like PostgreSQL for its unique feature set:

-   **Real-Time Capabilities**: Built-in support for live queries simplifies the implementation of features like real-time notifications.
-   **Embedded Permissions**: The database has a built-in, row-level security and permissions system, which reduces the need for custom authentication and authorization middleware in the application layer.
-   **Trade-off**: The version used (`3.0.0-alpha.10`) is pre-release software, which introduces a risk of instability and bugs compared to a mature database.

### Three-Tier Testing Architecture

The testing strategy is designed to provide a balance between fast feedback and thorough validation.

-   **Unit Tests (~2s)**: Run on every local save. They test individual functions in isolation with no network or database access.
-   **CI Tests (~8s)**: Run on every pull request. This is a subset of integration tests using an in-memory database to quickly catch breaking changes.
-   **Integration Tests (~44s)**: Run on merges to the `main` branch. These are full workflow tests against a real SurrealDB instance.

This tiered approach significantly reduces CI resource consumption and provides a faster feedback loop for developers.

## Performance Optimizations

### Frontend

-   **Code Splitting**: Route-specific components are loaded on demand. For example, the contact form logic is only loaded when a user navigates to the `/contact` page.
-   **CSS Purging**: TailwindCSS removes all unused styles from the final CSS bundle, reducing its size.
-   **Asset Optimization**: Images are converted to modern formats like WebP and served with appropriate cache headers.

### Backend

-   **Connection Pooling**: Database connections are managed in a pool and reused across requests to reduce connection overhead.
-   **Response Caching**: High-traffic content, such as blog posts, is cached in memory for a short duration (5 minutes) to reduce database load.
-   **Asynchronous I/O**: All operations are non-blocking, allowing the server to handle a high number of concurrent requests efficiently.

### Database

Strategic indexes have been added to improve query performance based on common access patterns.

```surql
-- For fetching posts by slug (most common query)
DEFINE INDEX slug_idx ON TABLE post COLUMNS slug UNIQUE;

-- For fetching recent posts on the homepage
DEFINE INDEX published_idx ON TABLE post COLUMNS published_at DESC;
```

Adding the `published_idx` index, for example, reduced the homepage load time by avoiding a full table scan.