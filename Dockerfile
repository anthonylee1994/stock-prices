# syntax=docker/dockerfile:1

FROM node:24.15-alpine AS base

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

WORKDIR /app

RUN corepack enable

FROM base AS deps

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

RUN pnpm install --frozen-lockfile

FROM deps AS build

COPY nest-cli.json tsconfig.json ./
COPY src ./src

RUN pnpm build
RUN pnpm prune --prod

FROM node:24.15-alpine AS runner

ENV NODE_ENV="production"
ENV PORT="3000"

WORKDIR /app

COPY --from=build /app/package.json ./package.json
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist

EXPOSE 3000

CMD ["node", "dist/main.js"]
