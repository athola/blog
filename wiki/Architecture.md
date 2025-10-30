# Architecture Overview

This document describes the overall architecture of the Rust blog engine, including component interactions, data flow, and design decisions.

## System Architecture

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

**Responsibilities**:
- Client-side routing and navigation
- Interactive UI components with reactive state
- WASM compilation for near-native performance
- Hydration from server-rendered HTML

**Key Components**:
```rust
app/src/
├── lib.rs              // Main Leptos application entry
├── components/         // Reusable UI components
│   ├── layout.rs       // Main layout component
│   ├── post.rs         // Blog post component
│   ├── contact.rs      // Contact form component
│   └── error_template.rs // Error handling
├── pages/              // Route-specific components
│   ├── home.rs         // Homepage component
│   ├── post.rs         // Post detail page
│   └── contact.rs      // Contact page
├── api.rs              // Server function definitions
├── types.rs            // Shared data types
└── utils.rs            // Frontend utilities
```

### 2. Backend (Axum + Leptos SSR)

**Location**: `server/`

**Responsibilities**:
- HTTP request handling and routing
- Server-side rendering (SSR) with Leptos
- API endpoint implementation
- Database connection and query management
- Static asset serving
- Security middleware

**Key Components**:
```rust
server/src/
├── main.rs             // Axum server setup and routing
├── utils.rs            // Database connections and utilities
└── middleware/         // HTTP middleware (future)
```

### 3. Database Layer (SurrealDB 3.0.0)

**Location**: `migrations/`, database connection code in `server/src/utils.rs`

**Key Features**:
- Multi-model database (document + graph + relational)
- Advanced authentication with Root/Namespace/Database levels
- Real-time queries and live data
- Built-in permissions and security

**Schema**:
```surql
-- Authors table
DEFINE TABLE author SCHEMAFULL;
DEFINE FIELD name ON TABLE author TYPE string;
DEFINE FIELD email ON TABLE author TYPE string;
DEFINE FIELD bio ON TABLE author TYPE string;

-- Posts table
DEFINE TABLE post SCHEMAFULL;
DEFINE FIELD title ON TABLE post TYPE string;
DEFINE FIELD slug ON TABLE post TYPE string;
DEFINE FIELD content ON TABLE post TYPE string;
DEFINE FIELD excerpt ON TABLE post TYPE string;
DEFINE FIELD published_at ON TABLE post TYPE datetime;
DEFINE FIELD views ON TABLE post TYPE int DEFAULT 0;

-- Activity tracking (NEW)
DEFINE TABLE activity SCHEMAFULL;
DEFINE FIELD action ON TABLE activity TYPE string;
DEFINE FIELD resource_type ON TABLE activity TYPE string;
DEFINE FIELD resource_id ON TABLE activity TYPE string;
DEFINE FIELD timestamp ON TABLE activity TYPE datetime DEFAULT time::now();
```

## Data Flow

### Request Lifecycle

1. **Initial Request**
   ```
   Browser → HTTP Request → Axum Server → Leptos SSR → Database
                                            ↓
   HTML Response ← Server Rendering ← Component Rendering ← Query Results
   ```

2. **Client-side Interaction**
   ```
   User Action → Leptos Component → WASM Function → Server Function → Database
                                                    ↓
   UI Update ← WASM Hydration ← Response Data ← Query Results
   ```

3. **API Requests**
   ```
   Client → HTTP API → Axum Handler → Database Query → JSON Response
   ```

### Authentication Flow

```
Application Startup
├── SurrealDB Connection (Root credentials)
├── Namespace Creation (rustblog)
├── Database Creation (rustblog)
└── Namespace User Setup (if configured)

Runtime Operations
├── Database Operations use Namespace/Database credentials
├── Administrative operations use Root credentials
└── Connection pooling and retry logic handle failures
```

## Technology Stack Details

### Frontend Stack

#### Leptos Framework
- **Full-stack Rust**: Single codebase for frontend and backend
- **Fine-grained reactivity**: Signal-based state management
- **Server Functions**: Type-safe RPC between client and server
- **SSR + Hydration**: Fast initial loads with interactive client

#### TailwindCSS
- **Utility-first CSS**: Rapid UI development
- **Responsive design**: Mobile-first approach
- **Custom components**: Consistent design system
- **Bundle optimization**: CSS purification and minification

### Backend Stack

#### Axum Web Server
- **Async/await**: Tokio-based high performance
- **Type-safe routing**: Compile-time route checking
- **Middleware system**: Composable request processing
- **Error handling**: Structured error responses

#### SurrealDB 3.0.0
- **Multi-model**: Documents, graphs, and relations
- **Real-time**: Live queries and subscriptions
- **Permissions**: Row-level security
- **Scaling**: Built-in clustering support

### Development Tooling

#### Testing Architecture
- **Three-tier strategy**: Unit → CI → Integration
- **Test frameworks**: nextest + cargo-llvm-cov
- **Mocking**: Test doubles for external services
- **CI-aware**: Optimized for pipeline execution

#### Security Tooling
- **Multi-tool scanning**: Gitleaks + Semgrep + Trufflehog
- **Automated gates**: Security findings block deployment
- **Dependency audit**: Cargo audit for known vulnerabilities
- **Secrets management**: Environment-based configuration

## Design Decisions

### 1. Full-stack Rust
**Rationale**: Type safety across the entire stack, shared code between frontend and backend, excellent performance.

**Benefits**:
- No JavaScript/TypeScript context switching
- Compile-time guarantees for API contracts
- Single dependency management system
- Shared data models and validation logic

### 2. SurrealDB over Traditional Databases
**Rationale**: Modern database with real-time capabilities, flexible schema, and built-in permissions.

**Benefits**:
- No separate ORM layer needed
- Real-time updates without additional infrastructure
- Flexible schema for rapid development
- Built-in authentication and permissions

### 3. Leptos over Other Frameworks
**Rationale**: True full-stack Rust with excellent SSR performance and fine-grained reactivity.

**Benefits**:
- Isomorphic rendering with minimal code duplication
- Fine-grained reactivity without virtual DOM overhead
- Type-safe server functions
- Excellent performance characteristics

### 4. Three-tier Testing Architecture
**Rationale**: Balance between comprehensive testing and CI performance constraints.

**Benefits**:
- Fast feedback for unit tests (~0s)
- Reasonable CI execution time (~5s)
- Full integration validation (~44s)
- Resource-conscious execution (50% reduction)

## Security Architecture

### Defense in Depth

1. **Network Layer**
   - HTTPS enforcement
   - Security headers (CSP, HSTS, etc.)
   - Rate limiting and DDoS protection

2. **Application Layer**
   - Input validation and sanitization
   - SQL injection prevention via parameterized queries
   - Authentication and authorization

3. **Data Layer**
   - Database-level permissions
   - Connection encryption
   - Audit logging

4. **Infrastructure Layer**
   - Container security (non-root execution)
   - Secrets management
   - Automated vulnerability scanning

### Authentication & Authorization

```rust
// Multi-level SurrealDB authentication
pub struct DatabaseConfig {
    root_credentials: Credentials,      // Administrative operations
    namespace_credentials: Credentials, // Namespace management
    database_credentials: Credentials,  // Application operations
}

// Connection with retry logic and default
pub async fn connect_with_retry() -> Result<Surreal<Client>, Error> {
    retry_with_exponential_backoff(|| {
        connect_to_database().await
    }).await
}
```

## Performance Optimization

### Frontend Optimizations
- **WASM Bundle**: Optimized for size (~150KB gzipped)
- **Code Splitting**: Lazy-loaded components
- **Asset Optimization**: CSS/JS minification and compression
- **Caching**: HTTP caching headers and service worker

### Backend Optimizations
- **Connection Pooling**: Efficient database connections
- **HTTP Compression**: gzip, brotli, deflate, zstd
- **Caching Strategy**: Multi-level caching for frequently accessed data
- **Async Processing**: Non-blocking I/O throughout

### Database Optimizations
- **Query Optimization**: Efficient SurrealDB queries
- **Indexing Strategy**: Strategic indexes for common queries
- **Connection Management**: Persistent connections with retry logic
- **Data Modeling**: Optimized schema for access patterns

## Scalability Considerations

### Horizontal Scaling
- **Stateless Design**: Server processes can be horizontally scaled
- **Database Scaling**: SurrealDB clustering support
- **CDN Integration**: Static asset distribution
- **Load Balancing**: Multiple server instances

### Vertical Scaling
- **Resource Optimization**: Efficient memory and CPU usage
- **Performance Monitoring**: Built-in metrics and health checks
- **Caching Layers**: Multiple caching strategies
- **Connection Pooling**: Database connection efficiency

## Future Architecture Evolution

### Planned Enhancements

1. **Microservices Architecture**
   - Separate content management service
   - Analytics and monitoring service
   - Notification service

2. **Event-Driven Architecture**
   - Message queue for async processing
   - Event sourcing for audit trails
   - Real-time notifications

3. **Advanced Caching**
   - Redis integration for session storage
   - CDN for global asset distribution
   - Application-level caching

4. **Advanced Security**
   - Zero-trust architecture
   - Advanced threat detection
   - Compliance automation

---

**Related Documents**:
- [Database Guide](Database-Guide.md)
- [Testing Guide](Testing-Guide.md)
- [Security Guide](Security-Guide.md)
- [Performance Tuning](Performance-Tuning.md)