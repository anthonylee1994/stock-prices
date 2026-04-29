# Repository Guidelines

## Project Structure & Module Organization

呢個 repo 係用 NestJS + TypeScript 寫嘅股票報價 API。Runtime source code 放喺 `src/`，入口係 `src/main.ts`，root module wiring 喺 `src/app.module.ts`。股票報價功能集中喺 `src/stock-prices/`，包括 controller、service、module、type 同 unit spec。End-to-end tests 同測試 helper 放喺 `test/`。Build output 會產生喺 `dist/`，唔好直接改。

## Build, Test, and Development Commands

- `bun install`: 按 `bun.lock` 安裝 dependencies。
- `bun run start:dev`: 用 Bun watch mode 跑 NestJS server，預設係 `http://localhost:3000`。
- `PORT=3100 bun run start:dev`: 用自訂 port 本地開發。
- `bun run build`: 用 `bun build --compile` compile executable 到 `dist/main`。
- `bun run start:prod`: 跑 compiled executable `./dist/main`。
- `bun run test`: 跑 Jest unit tests。
- `bun run test:e2e`: 用 `test/jest-e2e.json` 跑 API e2e tests。
- `bun run test:cov`: 跑 Jest coverage。
- `bun run tsc --noEmit -p tsconfig.json`: 只做 type check，唔輸出檔案。
- `bun run format`: 用 Prettier format `src/**/*.ts` 同 `test/**/*.ts`。

## Coding Style & Naming Conventions

跟返 repo 現有 TypeScript 同 NestJS 寫法。Feature code 按 module 分組，例如 `stock-prices.controller.ts`、`stock-prices.service.ts`、`stock-prices.module.ts`。用 4 spaces indentation、semicolons、double quotes；trailing comma 由 Prettier 決定。API boundary 優先用明確 exported class、interface 同 type。Controller 保持薄身；Yahoo Finance access 同 quote normalization 放喺 service。

## Testing Guidelines

Unit tests 用 Jest + `ts-jest`；test file 命名做 `*.spec.ts`，放喺被測 code 旁邊。E2E tests 放喺 `test/`，用 Supertest。Coverage 而家收集 `src/stock-prices/*.ts`，但排除 specs、modules 同 type-only files。改 request parsing、response shape、error handling 或 Yahoo Finance mapping 時，要同步加或更新 tests。

## Commit & Pull Request Guidelines

近期 commit 用短 conventional-style prefix，例如 `chore:`、`docs:`、`refactor:`。Commit subject 要具體同帶動作，例如 `fix: handle blank quote symbols`。Pull request 要有簡短描述、已跑嘅 test commands、相關 issue link；如果 API behavior 有變，要附 sample request 或 response。

## Security & Configuration Tips

唔好 commit secrets 或本地 environment files。呢個 API 依賴 Yahoo Finance response，所以 upstream failure 要 defensive 咁處理；除非係有意改 API contract，否則要保留現有 `400`、`405`、`502` 行為。
