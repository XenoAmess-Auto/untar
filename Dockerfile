# Build stage
FROM rust:bookworm AS builder

WORKDIR /usr/src/untar
COPY rust/ ./rust/
WORKDIR /usr/src/untar/rust
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends liblzma5 zlib1g \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/untar/rust/target/release/untar /usr/local/bin/untar

WORKDIR /workdir
ENTRYPOINT ["untar"]
