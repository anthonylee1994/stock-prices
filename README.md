# Stock Prices API

用 Rust 寫嘅股票報價 API，資料來源係 Yahoo Finance quote endpoint，response 會用 TOON 格式輸出。

## 功能

- Rust + axum
- Yahoo Finance 股票報價
- CORS enabled
- Toon response format
- 支援 Heroku / Railway / Render / Fly.io 呢類 hosting

## 環境要求

- Rust 1.95 或以上
- Cargo

## 安裝

```bash
cargo fetch
```

## 開發

起 server：

```bash
cargo run
```

預設會喺 `http://localhost:3000` 起 server。

## Build

```bash
cargo build --release
```

## Production

```bash
./target/release/stock-prices
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
│   └── main.rs           # HTTP endpoints、Yahoo Finance client、TOON encoder
├── target/               # build output
├── Cargo.toml            # Rust dependencies
├── Cargo.lock            # locked Rust dependencies
├── Procfile              # deployment entry
└── README.md
```

## Commands

- `cargo run`：起 development server
- `cargo build --release`：build production binary
- `cargo test`：跑 tests
- `cargo fmt`：format Rust code
- `cargo check`：type check

## 部署

呢個 project 有 `Procfile`，可以部署去支援 Rust buildpack 嘅 platform。部署前要確保：

```bash
cargo build --release
```

production command：

```bash
./target/release/stock-prices
```

## License

MIT
