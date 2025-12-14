# Stage 1: Build Environment with Rust nightly
FROM rustlang/rust:nightly-slim as builder

# Install required packages for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-leptos and wasm-bindgen-cli
RUN cargo install cargo-leptos wasm-bindgen-cli

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Configure WASM-specific environment for ring crate
ENV RING_CORE_PREFIX=ring_core_prefix_0_17_14

# Create app user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create working directory
WORKDIR /work

# Set environment variables for ring crate build
ENV CC=clang
ENV RING_CORE_PREFIX=ring_core_prefix

# Create analyzed metadata for DigitalOcean buildpack compatibility
RUN mkdir -p /layers && \
    echo '[analyzed]' > /layers/analyzed.toml && \
    echo 'version = "1.0.0"' >> /layers/analyzed.toml && \
    echo '[[analyzed.layers]]' >> /layers/analyzed.toml && \
    echo 'id = "build"' >> /layers/analyzed.toml && \
    echo 'version = "1.0.0"' >> /layers/analyzed.toml && \
    echo 'name = "build"' >> /layers/analyzed.toml

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY app/Cargo.toml ./app/
COPY frontend/Cargo.toml ./frontend/
COPY markdown/Cargo.toml ./markdown/
COPY server/Cargo.toml ./server/
COPY shared_utils/Cargo.toml ./shared_utils/

# Copy only source code needed for build
COPY app/ ./app/
COPY frontend/ ./frontend/
COPY markdown/ ./markdown/
COPY server/ ./server/
COPY shared_utils/ ./shared_utils/
COPY public/ ./public/
COPY style/ ./style/
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Build the application with optimizations
RUN cargo leptos build --release

# Stage 2: Runtime Environment - using Ubuntu for newer GLIBC support
FROM ubuntu:23.04 as runner

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user (matching builder stage)
RUN groupadd -r appuser && useradd -r -g appuser appuser

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

# Expose port (DigitalOcean App Platform uses 8080)
EXPOSE 8080

# Run the application
CMD ["/app/blog"]
