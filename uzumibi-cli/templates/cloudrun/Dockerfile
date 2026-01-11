FROM rust:1.92 as builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y clang libclang-dev build-essential curl

RUN rm -rf src

COPY build.rs ./
COPY Cargo.toml ./
COPY lib ./lib
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:13-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder \
    /app/target/release/$$PROJECT_NAME$$ \
    /app/$$PROJECT_NAME$$

ENV PORT=8080

CMD ["/app/$$PROJECT_NAME$$"]
