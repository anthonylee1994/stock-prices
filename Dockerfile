# syntax=docker/dockerfile:1

FROM rust:1.94-alpine AS build

WORKDIR /app

RUN apk add --no-cache build-base pkgconfig

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.23 AS runner

ENV PORT="3000"

WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=build /app/target/release/stock-prices ./stock-prices

EXPOSE 3000

CMD ["./stock-prices"]
