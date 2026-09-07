# ---- build stage ----
FROM rust:1-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked --bin stock-prices

# ---- runtime stage ----
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin app

COPY --from=builder /app/target/release/stock-prices /usr/local/bin/stock-prices

USER app

# Dokku 會 inject PORT；呢個係 fallback，同 EXPOSE 對齊。
ENV PORT=5000
EXPOSE 5000

CMD ["stock-prices"]
