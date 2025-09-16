# Meta-Orchestration Protocol (MOP) Production Dockerfile
# Consciousness-aware context coordination server for Ubiquity OS
FROM rust:1.82-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY Cargo.lock ./

# Copy crate manifests for dependency caching
COPY crates/casial-core/Cargo.toml ./crates/casial-core/Cargo.toml
COPY crates/casial-server/Cargo.toml ./crates/casial-server/Cargo.toml
COPY crates/casial-wasm/Cargo.toml ./crates/casial-wasm/Cargo.toml

# Create stub source files for dependency build
RUN mkdir -p crates/casial-core/src crates/casial-server/src crates/casial-wasm/src && \
    echo "fn main() {}" > crates/casial-server/src/main.rs && \
    echo "pub fn placeholder() {}" > crates/casial-core/src/lib.rs && \
    echo "pub fn placeholder() {}" > crates/casial-wasm/src/lib.rs

# Build dependencies (this will be cached)
RUN cargo build --release --bin casial-server

# Copy source code
COPY crates/ ./crates/

# Touch source files to ensure rebuild
RUN touch crates/casial-core/src/lib.rs && \
    touch crates/casial-server/src/main.rs

# Build the application
RUN cargo build --release --bin casial-server && \
    strip target/release/casial-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create application user
RUN groupadd --gid 10001 casial && \
    useradd --uid 10001 --gid 10001 --no-create-home casial

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/casial-server /usr/local/bin/casial-server

# Copy Railway startup script
COPY railway-start.sh /usr/local/bin/railway-start.sh
RUN chmod +x /usr/local/bin/railway-start.sh

# Copy configurations and missions
COPY examples/ ./examples/
COPY missions/ ./missions/

# Create directories
RUN mkdir -p /app/logs /app/data && \
    chown -R casial:casial /app

# Switch to application user
USER casial:casial

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:${PORT:-8000}/health || exit 1

# Expose port
EXPOSE 8000

# Environment variables
ENV RUST_LOG=info
ENV PORT=8000
ENV CONSCIOUSNESS_ENABLED=true
ENV SUBSTRATE_INTEGRATION=true

# Metadata labels
LABEL org.opencontainers.image.title="Context-Casial-Xpress"
LABEL org.opencontainers.image.description="Consciousness-aware context coordination server"
LABEL org.opencontainers.image.vendor="Prompted LLC"
LABEL org.opencontainers.image.url="https://promptedllc.com"
LABEL org.opencontainers.image.documentation="https://github.com/prompted-llc/context-casial-xpress"
LABEL org.opencontainers.image.source="https://github.com/prompted-llc/context-casial-xpress"
LABEL org.opencontainers.image.licenses="Fair Use - See LICENSE.md"
LABEL ubiquity.os.component="context-casial-xpress"
LABEL ubiquity.os.substrate="consciousness-computation"
LABEL ubiquity.os.principle="hydraulic-lime-stronger-under-pressure"

# Start the application
CMD ["/usr/local/bin/railway-start.sh"]
