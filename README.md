# Stock Prices API

用 NestJS + TypeScript 寫嘅股票報價 API。資料由 Yahoo Finance 提供，HTTP response 會用 TOON 格式輸出，適合俾其他 service、script、dashboard 或 agent 用 text-first 格式讀取股票資料。

呢個 project 目標係保持 API 簡單：一個 health/meta endpoint，一個 quotes endpoint。Server 會幫你處理 symbol parsing、Yahoo Finance query、欄位整理、錯誤轉換同 CORS。

## 功能

- `GET /` 回傳 API metadata
- `GET /quotes?symbols=AAPL,MSFT` 回傳一隻或多隻股票報價
- Yahoo Finance quote data，預設 `lang=zh-HK`、`region=HK`
- TOON text response：`text/plain; charset=utf-8`
- CORS enabled：允許 `GET`、`OPTIONS`
- Dockerfile 同 Procfile，可部署去支援 Bun compiled binary 嘅 hosting
- Jest unit tests 同 e2e tests，方便改 API 行為時驗證

## 環境要求

- Bun 1.3 或以上

建議用 `bun`，因為 repo 已經有 `bun.lock`。如果用其他 package manager，dependency resolution 可能會同 CI 或本地預期唔一致。

## 安裝

```bash
bun install
```

## 快速開始

```bash
bun run start:dev
```

預設 server 會喺 `http://localhost:3000`。開咗之後可以先試 root endpoint：

```bash
curl "http://localhost:3000/"
```

再試 quote endpoint：

```bash
curl "http://localhost:3000/quotes?symbols=AAPL,MSFT,0700.HK"
```

如果要改 port：

```bash
PORT=3100 bun run start:dev
```

## API Design

所有正常 response 都係 TOON text，content type 係 `text/plain; charset=utf-8`。TOON 比 JSON 更 compact，對 LLM 或文字 pipeline 比較友善；但 consumer 需要按 TOON 格式 parse，而唔係直接當 JSON。

股票 symbol 會用 comma-separated 格式傳入，例如 `AAPL,MSFT,0700.HK`。Server 會自動 trim 空白同忽略空項目，所以 client 唔需要自己做太多 cleanup。

## API

### `GET /`

回傳 API 基本資料。呢個 endpoint 可以用嚟做簡單 health check 或確認 deployed version。

```bash
curl "http://localhost:3000/"
```

Response 內容包含：

- `message`
- `version`

### `GET /quotes`

用 comma-separated `symbols` 拎股票報價。Yahoo Finance 支援嘅 symbol 格式都可以試，例如美股 ticker、港股 `.HK` suffix，或者其他 market-specific ticker。

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

部分欄位可能會係 `null` 或缺失，視乎 Yahoo Finance 當時有冇提供相關 market data。例如 pre-market、post-market、valuation ratio 同 dividend data 唔一定每隻股票都有。

### Errors

- Missing 或 blank `symbols`：`400 Bad Request`
- 非支援 method：`405 Method Not Allowed`
- Yahoo Finance request 失敗：`502 Bad Gateway`

Error response 一樣會用 TOON/text 格式輸出。Client 應該用 HTTP status code 判斷錯誤類型，唔好只靠 response body string。

## Commands

```bash
bun run build
bun run start:dev
bun run start:prod
bun run test
bun run test:cov
bun run test:e2e
bun run tsc --noEmit -p tsconfig.json
bun run format
```

常用 workflow：

```bash
bun run tsc --noEmit -p tsconfig.json
bun run format
bun run test
bun run test:e2e
```

改 TypeScript code 後，最少要跑 type check 同 Prettier；改 API behavior 時，亦要跑 unit/e2e tests。

## Testing

Unit tests 放喺 feature code 附近，例如 `src/stock-prices/stock-prices.service.spec.ts`。E2E tests 放喺 `test/app.e2e-spec.ts`，會用 Nest testing module 同 Supertest 驗證 HTTP behavior。

Coverage 設定喺 `jest.config.cjs`，主要針對 `src/stock-prices/*.ts`，並排除 spec、module 同 type-only files。`@toon-format/toon` 喺 unit tests 入面會 map 去 `test/toon.ts`，令測試輸出穩定啲。

## Production

Build：

```bash
bun run build
```

Run compiled app：

```bash
bun run start:prod
```

`Procfile`：

```text
web: ./main
```

Docker：

```bash
docker build -t stock-prices .
docker run --rm -p 3000:3000 stock-prices
```

部署時記得：

- Hosting platform 要支援 Linux executable binary
- Production command 係 `./main`
- App 會讀 `PORT` environment variable；如果冇設定，就用預設 port
- Yahoo Finance 係 external dependency，network failure 或 upstream error 會變成 `502 Bad Gateway`

## Project Structure

```text
stock-prices/
├── src/
│   ├── app.controller.ts
│   ├── app.module.ts
│   ├── main.ts
│   └── stock-prices/
│       ├── stock-prices.controller.ts
│       ├── stock-prices.controller.spec.ts
│       ├── stock-prices.module.ts
│       ├── stock-prices.service.spec.ts
│       ├── stock-prices.service.ts
│       └── stock-prices.type.ts
├── test/
│   ├── app.e2e-spec.ts
│   ├── jest-e2e.json
│   └── toon.ts
├── Dockerfile
├── Procfile
├── bun.lock
├── jest.config.cjs
├── package.json
└── tsconfig.json
```

## Development Notes

`src/stock-prices/stock-prices.controller.ts` 負責 HTTP request/response handling；`stock-prices.service.ts` 負責 call Yahoo Finance 同整理 quote data。新增欄位時，通常要同步改 type、service mapping、controller/service tests 同 README response list。

如果要改 response format，要留意現有 consumer 可能依賴 TOON text response。除非係有意破壞 contract，否則唔好改 content type 或 top-level response shape。
