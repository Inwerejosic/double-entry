FROM rust:1.96-slim AS builder

WORKDIR /usr/src/double-entry
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 curl && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/double-entry/target/release/double-entry ./double-entry

EXPOSE 8080
CMD ["./double-entry"]
