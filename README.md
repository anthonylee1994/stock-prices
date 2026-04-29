# Stock Prices API

用 NestJS 寫嘅股票報價 API，資料來源係 Yahoo Finance quote endpoint，response 會用 TOON 格式輸出。

## 功能

- NestJS + TypeScript
- Yahoo Finance 股票報價
- CORS enabled
- TOON response format
- 支援 Heroku / Railway / Render / Fly.io 呢類 hosting

## 環境要求

- Node.js 24 或以上
- pnpm 10 或以上

## 開發

安裝 dependencies：

```bash
pnpm install
```

起 server：

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
pnpm start
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
{"error": "symbols is required"}
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
│   ├── main.ts           # server startup
│   ├── app.module.ts     # NestJS module
│   ├── stock-prices/     # Yahoo Finance client + domain types
│   └── web/              # HTTP controllers / exception filter
├── test/                 # e2e tests
├── package.json          # Node package
├── Procfile              # deployment entry
└── README.md
```

## Commands

- `pnpm start:dev`：起 development server
- `pnpm build`：build production JS
- `pnpm start`：起 production server
- `pnpm test`：跑 tests
- `pnpm typecheck`：TypeScript type check
- `pnpm format`：Prettier format

## 部署

呢個 project 有 `Procfile`，可以部署去支援 Node.js buildpack 嘅 platform。部署前要確保：

```bash
pnpm install
pnpm build
```

production command：

```bash
pnpm start
```

## License

MIT
