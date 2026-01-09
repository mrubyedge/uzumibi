FROM rust:1.92 as builder

WORKDIR /app

RUN rm -rf src

COPY Cargo.toml ./
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:13-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder \
     /app/target/release/uzumibi-on-cloudrun-spike \
     /app/uzumibi-on-cloudrun-spike

ENV PORT=8080

CMD ["/app/uzumibi-on-cloudrun-spike"]
