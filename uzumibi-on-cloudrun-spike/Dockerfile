# ビルドステージ
FROM rust:1.83 as builder

WORKDIR /app

RUN rm -rf src

# 実際のソースコードをコピーしてビルド
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# 実行ステージ
FROM debian:13-slim

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルドステージからバイナリをコピー
COPY --from=builder /app/target/release/uzumibi-on-cloudrun-spike /app/uzumibi-on-cloudrun-spike

# Cloud Runはデフォルトで8080ポートを使用
ENV PORT=8080

# バイナリを実行
CMD ["/app/uzumibi-on-cloudrun-spike"]
