# syntax=docker/dockerfile:1

FROM oven/bun:1.3.13-alpine AS build
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .
RUN bun run build

FROM alpine:3.22 AS runner

ENV PORT="3000"
ENV NODE_ENV="production"

WORKDIR /app

RUN apk add --no-cache libgcc libstdc++

COPY --from=build /app/dist/main ./main

EXPOSE 3000

CMD ["./main"]
