# Stock Prices API

用 Rust + Axum 寫嘅股票報價 API。資料由 Yahoo Finance 提供，HTTP response 係 JSON，適合俾其他 service、script、dashboard 或 agent 讀取股票資料。

呢個 project 目標係保持 API 簡單：一個 health/meta endpoint，一個 quotes endpoint。Server 會幫你處理 symbol parsing、Yahoo Finance query（連 cookie/crumb session）、欄位整理、錯誤轉換同 CORS。

## 功能

- `GET /` 回傳 API metadata
- `GET /quotes?symbols=AAPL,MSFT` 回傳一隻或多隻股票報價
- Yahoo Finance `v7/finance/quote` data，預設 `lang=zh-HK`、`region=HK`
- JSON response：`application/json`
- CORS enabled：允許 `GET`、`OPTIONS`
- Cargo unit tests 同 integration tests，方便改 API 行為時驗證

## 環境要求

- Rust 1.90 或以上（edition 2024）

冇其他 runtime dependency：`cargo build --release` 出嚟係一個 static-ish binary，直接跑就得。

## 安裝

```bash
cargo build
```

## 快速開始

```bash
cargo run
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
PORT=3100 cargo run
```

## API Design

所有正常 response 都係 JSON，content type 係 `application/json`。缺失嘅欄位會直接由 response 中省略，而唔會出現 `null`。

股票 symbol 會用 comma-separated 格式傳入，例如 `AAPL,MSFT,0700.HK`。Server 會自動 trim 空白同忽略空項目，所以 client 唔需要自己做太多 cleanup。非港股 symbol 嘅第一個 `.` 會轉成 `-`（例如 `BRK.B` → `BRK-B`），因為 Yahoo 就係用呢個格式表示 share class；`.HK` suffix 會原樣保留。

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

部分欄位可能會缺失，視乎 Yahoo Finance 當時有冇提供相關 market data。例如 pre-market、post-market、valuation ratio 同 dividend data 唔一定每隻股票都有。時間欄位係 ISO 8601 UTC string，例如 `2026-09-04T20:00:01.000Z`。已退市嘅 symbol（Yahoo 回 `quoteType: "NONE"`）會被過濾走。

### Errors

- Missing 或 blank `symbols`：`400 Bad Request`
- 非支援 method：`405 Method Not Allowed`
- Yahoo Finance request 失敗：`502 Bad Gateway`

Error response 一樣係 JSON，shape 係 `{message, error?, statusCode}`。Client 應該用 HTTP status code 判斷錯誤類型，唔好只靠 response body string。上游錯誤嘅完整 cause chain 會 log 去 stderr，唔會出現喺 response 度。

## Commands

```bash
cargo build
cargo build --release
cargo run
cargo test
cargo fmt
cargo clippy --all-targets
```

常用 workflow：

```bash
cargo fmt
cargo test
```

改完 Rust code 後，最少要跑 `cargo fmt`；改 API behavior 時，亦要跑 `cargo test`。

## Testing

Unit tests 放喺被測 code 同一個檔案嘅 `#[cfg(test)] mod tests` 入面，例如 `src/stock_prices/controller.rs` 嘅 symbol parsing 同 `src/stock_prices/service.rs` 嘅 quote mapping。Integration tests 放喺 `tests/app_e2e.rs`，會用 `tower::ServiceExt::oneshot` 直接打真嘅 Axum `Router`，驗證 status code、content type、JSON body 同 CORS header。

Integration tests 用 `QuoteProvider` trait 嘅 stub 代替真 Yahoo Finance，所以測試唔會出網。

## Production

Build：

```bash
cargo build --release
```

Run compiled app：

```bash
./target/release/stock-prices
```

部署時記得：

- Production command 係 `./target/release/stock-prices`
- App 會讀 `PORT` environment variable；如果冇設定，就用 `3000`
- Server bind `0.0.0.0`，所以 container 或 PaaS 都用得
- Yahoo Finance 係 external dependency，network failure 或 upstream error 會變成 `502 Bad Gateway`

## Project Structure

```text
stock-prices/
├── src/
│   ├── app.rs                  # Router wiring + CORS
│   ├── app_controller.rs       # GET /
│   ├── error.rs                # ApiError → HTTP status + JSON body
│   ├── lib.rs
│   ├── main.rs                 # bootstrap，讀 PORT，graceful shutdown
│   └── stock_prices/
│       ├── controller.rs       # GET /quotes + symbol parsing
│       ├── mod.rs
│       ├── service.rs          # QuoteProvider trait + Yahoo → Quote mapping
│       ├── types.rs            # Quote response shape
│       └── yahoo.rs            # Yahoo Finance client（cookie/crumb session）
├── tests/
│   └── app_e2e.rs
├── Cargo.toml
├── Cargo.lock
└── rustfmt.toml
```

## Development Notes

`src/stock_prices/controller.rs` 負責 HTTP request/response handling；`service.rs` 負責整理 quote data；`yahoo.rs` 負責同 Yahoo Finance 溝通。新增欄位時，通常要同步改 `yahoo.rs` 嘅 `YahooQuote`、`types.rs` 嘅 `Quote`、`service.rs` 嘅 `map_quote`、相關 tests 同 README response list。

Yahoo 嘅 `v7/finance/quote` 要 session cookie 加一個配對嘅 crumb token，`yahoo.rs` 會 lazy 咁拎一次然後 cache，等 Yahoo 拒收（`401`/`403` 或者 description 提到 crumb）時先自動換新嘅再 retry 一次。另外 Yahoo edge 會用 protocol error 中斷我哋嘅 HTTP/2 stream，所以 client 特意 pin 咗 HTTP/1.1，唔好隨手拆。

如果要改 response format，要留意現有 consumer 可能依賴而家嘅 JSON shape。除非係有意破壞 contract，否則唔好改 content type 或 top-level response shape。
