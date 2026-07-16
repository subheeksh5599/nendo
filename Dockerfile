# Nendo — Agent RPC Firewall for Avalanche
# Multi-stage Docker build: compile in Rust image, run in slim Debian.

# ── Build stage ──────────────────────────────────────────────────────────
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build release binary (cached layers for dependencies)
RUN cargo build --release && \
    strip target/release/nendo

# ── Runtime stage ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the compiled binary
COPY --from=builder /app/target/release/nendo /app/nendo

# Default config
COPY config.example.toml /app/config.example.toml

EXPOSE 8545

# Run with default config; override with -v config.toml:/app/config.toml
ENTRYPOINT ["/app/nendo"]
