# Stock Prices API

用 Go 寫嘅股票報價 API，資料來源係 Yahoo Finance quote endpoint，response 會用 TOON 格式輸出。

## 功能

- Go + Gin
- Yahoo Finance 股票報價
- CORS enabled
- TOON response format via `github.com/toon-format/toon-go`
- 支援 Heroku / Railway / Render / Fly.io 呢類 hosting

## 環境要求

- Go 1.25 或以上

## 開發

起 server：

```bash
go run ./src
```

預設會喺 `http://localhost:3000` 起 server。

## Build

```bash
go build -o stock-prices ./src
```

## Production

```bash
./stock-prices
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
│   ├── main.go           # server startup、graceful shutdown
│   ├── app/              # Gin routing、middleware
│   ├── controllers/      # Gin HTTP controllers
│   ├── models/           # response / domain structs
│   ├── services/         # Yahoo Finance client
│   └── views/            # TOON / JSON renderers
├── go.mod                # Go module
├── Procfile              # deployment entry
└── README.md
```

## Commands

- `go run ./src`：起 development server
- `go build -o stock-prices ./src`：build production binary
- `go test ./...`：跑 tests
- `gofmt -w .`：format Go code

## 部署

呢個 project 有 `Procfile`，可以部署去支援 Go buildpack 嘅 platform。部署前要確保：

```bash
go build -o stock-prices ./src
```

production command：

```bash
./stock-prices
```

## License

MIT
