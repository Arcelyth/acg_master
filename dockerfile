FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binaryen \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /app
COPY . .

RUN cd web && trunk build --release
RUN cargo build -p acg_master_server --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/acg_master_server /app/server
COPY --from=builder /app/web/dist /app/site

ENV LEPTOS_SITE_ROOT=/app/site
ENV LEPTOS_SITE_ADDR=0.0.0.0:80

EXPOSE 8060

CMD ["./server"]
