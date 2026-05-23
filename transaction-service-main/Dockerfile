# ---------- Builder ----------
FROM rust:latest AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy ALL source files (including .sqlx directory)
COPY . .

# Enable offline mode - SQLx will use the .sqlx metadata instead of connecting to DB
ENV SQLX_OFFLINE=true

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/transaction-service /usr/local/bin/transaction-service

EXPOSE 8080

CMD ["transaction-service"]