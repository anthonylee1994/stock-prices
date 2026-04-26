# Stock Prices API

用 NestJS + TypeScript 寫嘅股票報價 API，資料來源係 `yahoo-finance2`，response 會用 Toon 格式輸出。

## 功能

- NestJS application structure
- TypeScript
- Yahoo Finance 股票報價
- CORS enabled
- Toon response format
- 支援 Heroku / Railway / Render / Fly.io 呢類 Node.js hosting

## 環境要求

- Node.js 18 或以上
- pnpm 8 或以上

## 安裝

```bash
pnpm install
```

## 開發

開 watch mode：

```bash
pnpm start:dev
```

預設會喺 `http://localhost:3000` 起 server。

## Build

```bash
pnpm build
```

## Production

```bash
pnpm start:prod
```

## API

### `GET /`

回傳 API 基本資料。

例子：

```bash
curl "http://localhost:3000/"
```

### `GET /quotes`

用 comma-separated symbols 拎多隻股票報價。

例子：

```bash
curl "http://localhost:3000/quotes?symbols=AAPL,GOOGL,MSFT"
```

如果冇傳 `symbols`，會回傳 `400`：

```json
{"error":"symbols is required"}
```

## 環境變數

可以喺 project root 建立 `.env`：

```env
PORT=3000
```

## Project Structure

```text
stock-prices/
├── src/
│   ├── app.controller.ts # HTTP endpoints
│   ├── app.module.ts     # Nest module
│   ├── app.service.ts    # 股票報價 service
│   ├── main.ts           # Nest bootstrap
│   └── types.ts          # 共用 types
├── dist/                 # build output
├── nest-cli.json         # Nest CLI config
├── package.json          # scripts 同 dependencies
├── tsconfig.json         # TypeScript config
├── Procfile              # deployment entry
└── README.md
```

## Scripts

- `pnpm start`：用 Nest CLI 起 app
- `pnpm start:dev`：watch mode 開發
- `pnpm start:debug`：debug + watch mode
- `pnpm start:prod`：跑 build 後嘅 `dist/main`
- `pnpm build`：build NestJS project
- `pnpm format`：用 Prettier format TypeScript files
- `pnpm lint`：用 ESLint fix TypeScript files
- `pnpm test`：跑 Jest tests
- `pnpm test:watch`：Jest watch mode
- `pnpm test:cov`：Jest coverage
- `pnpm test:e2e`：跑 e2e tests

## 部署

呢個 project 有 `Procfile`，可以部署去 Heroku 類似嘅 Node.js platform。部署前要確保：

```bash
pnpm build
```

production command：

```bash
pnpm start:prod
```

Nest CLI build output 入口係 `dist/main`。如果部署平台直接讀 `Procfile`，記得同 `start:prod` 保持一致。

## License

ISC
