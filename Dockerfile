# syntax=docker/dockerfile:1

FROM golang:1.26-alpine AS build

WORKDIR /app

COPY go.mod ./
COPY src ./src

RUN CGO_ENABLED=0 go build -o stock-prices ./src

FROM alpine:3.23 AS runner

ENV PORT="3000"

WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=build /app/stock-prices ./stock-prices

EXPOSE 3000

CMD ["./stock-prices"]
