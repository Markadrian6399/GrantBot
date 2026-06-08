# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder
WORKDIR /app

# Install compile-time deps (pkg-config needed for some crates)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependency compilation separately from source changes
COPY Cargo.toml Cargo.lock ./
# Create stub lib + all bin stubs so Cargo resolves the full dependency graph
RUN mkdir -p src/bin && \
    echo "fn main(){}" > src/main.rs && \
    echo "pub fn placeholder(){}" > src/lib.rs && \
    for b in test_venice test_oneshot test_webhook; do \
      echo "fn main(){}" > src/bin/$b.rs; \
    done && \
    cargo build --release --bin backend && \
    rm -rf src

# Build the real source
COPY . .
# Touch main.rs so Cargo re-links even if only src files changed
RUN touch src/main.rs && cargo build --release --bin backend

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend ./backend

# migrations are embedded by sqlx::migrate!() at compile time — no folder needed

EXPOSE 3001

CMD ["./backend"]
