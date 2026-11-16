# Stage 1: Build
FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --locked --release --features web --bin airust

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/airust /usr/local/bin/airust
COPY --from=builder /app/knowledge/ ./knowledge/

ENV AIRUST_BIND_ADDR=0.0.0.0
EXPOSE 7070 7071
CMD ["airust"]
