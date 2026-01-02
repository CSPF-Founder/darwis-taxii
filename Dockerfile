# syntax=docker/dockerfile:1

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy everything
COPY . .

# Build with offline SQLx (no database needed at build time)
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bin taxii-server --bin taxii-cli && \
    cp target/release/taxii-server /taxii-server && \
    cp target/release/taxii-cli /taxii-cli

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash taxii

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /taxii-server /app/taxii-server
COPY --from=builder /taxii-cli /app/taxii-cli

# Copy migrations for CLI to run
COPY migrations /app/migrations

# Set ownership and switch to non-root user
RUN chown -R taxii:taxii /app
USER taxii

EXPOSE 9000

CMD ["/app/taxii-server"]
