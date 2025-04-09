# Stage 1: Build Environment with Rust nightly on Alpine
FROM rustlang/rust:nightly-alpine as builder

# Install required packages
RUN apk update && apk add --no-cache bash clang curl g++ libc-dev make perl

# Install cargo-leptos
RUN cargo install cargo-leptos

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Create working directory
WORKDIR /work
COPY . /work

# Build the application
RUN cargo leptos build --release -vv

# Stage 2: Runtime Environment
FROM rustlang/rust:nightly-alpine as runner

WORKDIR /app

# Copy the ssr binary and site content from the builder stage
COPY --from=builder /work/target/release/server /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8999"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 8999

# Run the ssr
CMD ["/app/blog"]
