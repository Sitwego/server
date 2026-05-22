# ---------- BUILD STAGE ----------
ARG debian_ver=bookworm 
FROM debian:${debian_ver} AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \ 
    clang \
    gcc \
    ninja-build \
    curl \
    git \
    libssl-dev \
    pkg-config \
    ca-certificates \
    wget \
    libpq-dev \
    librdkafka-dev \
    && update-ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
COPY rust-toolchain.toml ./
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain $(grep 'channel' rust-toolchain.toml | sed 's/.*"\(.*\)".*/\1/')
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-chef
RUN cargo install cargo-chef

WORKDIR /usr/src/api

# ---------- PLANNER STAGE ----------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------- COOKER STAGE ----------
FROM chef AS builder
ARG rust_flags
ENV RUSTFLAGS=$rust_flags
WORKDIR /usr/src/api

COPY --from=planner /usr/src/api/recipe.json recipe.json
# CHANGED: Added --release for consistency
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
 # ADDED: Strip binary to reduce size
RUN cargo build --bin api --release && \
    strip /usr/src/api/target/release/api

# ---------- RUNTIME STAGE ----------
FROM debian:${debian_ver}-slim AS runtime

LABEL maintainer="bizz.john@yahoo.com"
LABEL org.opencontainers.image.source="https://github.com/Sitwego/backend"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    librdkafka1 \
    libssl3 \
    curl \
    jq \
    && update-ca-certificates && \
    mkdir -p /app/data && \
    chown -R nobody:nogroup /app/data && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/api/target/release/api /usr/local/bin/api
# COPY --from=builder /usr/src/api/.env /app/.env

# Copy migrations directory to expected path
COPY packages/api/migrations /usr/src/api/packages/api/migrations
RUN chown -R nobody:nogroup /usr/src/api/packages/api
# Run as non-root user
USER nobody:nogroup

# Expose port and add health check
EXPOSE 8090
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:8090 || exit 1

ENTRYPOINT ["/usr/local/bin/api"]
