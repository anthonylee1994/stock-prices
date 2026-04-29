# Stock Prices API

用 NestJS + TypeScript 寫嘅股票報價 API。資料由 Yahoo Finance 提供，HTTP response 會用 TOON 格式輸出。

## 功能

- `GET /` 回傳 API metadata
- `GET /quotes?symbols=AAPL,MSFT` 回傳一隻或多隻股票報價
- Yahoo Finance quote data，預設 `lang=zh-HK`、`region=HK`
- TOON text response：`text/plain; charset=utf-8`
- CORS enabled：允許 `GET`、`OPTIONS`
- Dockerfile 同 Procfile，可部署去常見 Node.js hosting

## 環境要求

- Node.js 24 或以上
- pnpm 10 或以上

## 安裝

```bash
pnpm install
```

## 開發

```bash
pnpm start:dev
```

預設 server 會喺 `http://localhost:3000`。

如果要改 port：

```bash
PORT=3100 pnpm start:dev
```

## API

### `GET /`

回傳 API 基本資料。

```bash
curl "http://localhost:3000/"
```

Response 內容包含：

- `message`
- `version`

### `GET /quotes`

用 comma-separated `symbols` 拎股票報價。

```bash
curl "http://localhost:3000/quotes?symbols=AAPL,MSFT,0700.HK"
```

`symbols` 會自動 trim 空白同忽略空項目，所以以下 request 會被解析成 `AAPL`、`MSFT`、`0700.HK`：

```bash
curl "http://localhost:3000/quotes?symbols=%20AAPL,%20MSFT%20,,0700.HK"
```

Response 內容：

- `quotes[].symbol`
- `quotes[].name`
- `quotes[].market`
- `quotes[].currentPrice`
- `quotes[].change`
- `quotes[].percentChange`
- `quotes[].highPrice`
- `quotes[].lowPrice`
- `quotes[].openPrice`
- `quotes[].regularMarketTime`
- `quotes[].previousClosePrice`
- `quotes[].preMarketPrice`
- `quotes[].preMarketChange`
- `quotes[].preMarketChangePercent`
- `quotes[].preMarketTime`
- `quotes[].postMarketPrice`
- `quotes[].postMarketChange`
- `quotes[].postMarketChangePercent`
- `quotes[].postMarketTime`
- `quotes[].forwardPE`
- `quotes[].priceToBook`
- `quotes[].dividendYield`

### Errors

- Missing 或 blank `symbols`：`400 Bad Request`
- 非支援 method：`405 Method Not Allowed`
- Yahoo Finance request 失敗：`502 Bad Gateway`

## Commands

```bash
pnpm build
pnpm start
pnpm start:dev
pnpm test
pnpm test:cov
pnpm test:e2e
pnpm exec tsc --noEmit -p tsconfig.json
pnpm exec prettier --write "src/**/*.ts" "test/**/*.ts" "tsconfig*.json" "jest.config.cjs" "test/jest-e2e.json"
```

## Production

Build：

```bash
pnpm build
```

Run compiled app：

```bash
pnpm start:prod
```

`Procfile`：

```text
web: node dist/main.js
```

Docker：

```bash
docker build -t stock-prices .
docker run --rm -p 3000:3000 stock-prices
```

## Project Structure

```text
stock-prices/
├── src/
│   ├── app.controller.ts
│   ├── app.module.ts
│   ├── main.ts
│   └── stock-prices/
│       ├── stock-prices.controller.ts
│       ├── stock-prices.module.ts
│       ├── stock-prices.service.ts
│       └── stock-prices.type.ts
├── test/
│   ├── app.e2e-spec.ts
│   ├── jest-e2e.json
│   └── toon.ts
├── Dockerfile
├── Procfile
├── jest.config.cjs
├── package.json
└── tsconfig.json
```
