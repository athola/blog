# Stage 1: Build Environment with Rust nightly
FROM rustlang/rust:nightly-slim as builder

# Install required packages for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-leptos and wasm-bindgen-cli
RUN cargo install cargo-leptos wasm-bindgen-cli

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Create app user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create working directory
WORKDIR /work

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY app/Cargo.toml ./app/
COPY frontend/Cargo.toml ./frontend/
COPY markdown/Cargo.toml ./markdown/
COPY server/Cargo.toml ./server/

# Copy only source code needed for build
COPY app/ ./app/
COPY frontend/ ./frontend/
COPY markdown/ ./markdown/
COPY server/ ./server/
COPY public/ ./public/
COPY style/ ./style/
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Build the application with optimizations
ENV RUSTFLAGS="-C target-cpu=native"
RUN cargo leptos build --release

# Stage 2: Runtime Environment - using distroless for security
FROM gcr.io/distroless/cc-debian12 as runner

# Copy app user from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Create app directory
WORKDIR /app

# Copy the binary and site content from the builder stage
COPY --from=builder --chown=appuser:appuser /work/target/release/server /app/blog
COPY --from=builder --chown=appuser:appuser /work/target/site /app/site

# Switch to non-root user
USER appuser

# Set environment variables for production
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_HASH_FILES="true"
ENV LEPTOS_RELOAD_PORT="3001"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/blog", "--health-check"] || exit 1

# Expose port (DigitalOcean App Platform uses 8080)
EXPOSE 8080

# Run the application
CMD ["/app/blog"]
