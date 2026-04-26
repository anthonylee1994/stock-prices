---
name: stock-prices
description: 用 Stock Prices API 查即時股票價格同市場數據。Response 係 TOON 格式，請用 @toon-format/toon decode。適合用喺查股票報價、分析市場數據，或者處理 AAPL、NVDA、GOOGL 等 ticker symbol。
---

# Stock Prices API Skill

呢個 skill 幫你用 Stock Prices API 拎即時市場數據同股票報價。

## API Endpoint

**Base URL**：`https://stock-prices.on99.app`

**主要 Endpoint**：`/quotes?symbols={SYMBOLS}`

## 快速開始

查一隻或多隻股票報價。Response 會係 TOON 格式：

```bash
curl "https://stock-prices.on99.app/quotes?symbols=NVDA"
curl "https://stock-prices.on99.app/quotes?symbols=AAPL,GOOGL,MSFT"
```

如果要 parse TOON，先裝 decoder：

```bash
pnpm add @toon-format/toon
```

## Response 格式

API 會回傳 **TOON**（Token-Oriented Object Notation）格式。TOON 係一種 compact、易讀嘅 encoding，通常比 JSON 慳大約 40% tokens，適合放入 LLM prompt 或 streaming 場景。

### TOON Response 例子

```text
quotes[1]{symbol,currentPrice,change,percentChange,highPrice,lowPrice,openPrice,previousClosePrice,preMarketPrice,preMarketChange,preMarketTime,preMarketChangePercent,...}:
  NVDA,188.54,-1.5,-0.789308,192.48,188.12,191.405,190.04,191.8799,3.3399048,2026-02-11T13:49:16.000Z,1.771457,...
```

### Decode TOON Response

用 `@toon-format/toon` 將 response parse 返做 JavaScript / JSON data：

```typescript
import {decode} from "@toon-format/toon";

const response = await fetch("https://stock-prices.on99.app/quotes?symbols=NVDA");
const toonText = await response.text();
const data = decode(toonText);

// data.quotes 係 quote object array
const quote = data.quotes[0];
console.log(`${quote.symbol}: $${quote.currentPrice}`);
```

Decode 完嘅 structure 同 JSON 一樣，都係 objects、arrays 同 primitives。

## 可用 Data Fields

| Field                    | Type              | 說明                     |
| ------------------------ | ----------------- | ------------------------ |
| `symbol`                 | string            | 股票 ticker symbol       |
| `currentPrice`           | number            | 現價                     |
| `change`                 | number            | 相對上一個收市價嘅升跌   |
| `percentChange`          | number            | 相對上一個收市價嘅升跌 % |
| `highPrice`              | number            | 即日最高價               |
| `lowPrice`               | number            | 即日最低價               |
| `openPrice`              | number            | 開市價                   |
| `previousClosePrice`     | number            | 上一個交易日收市價       |
| `preMarketPrice`         | number            | 盤前價格                 |
| `preMarketChange`        | number            | 盤前升跌                 |
| `preMarketTime`          | string (ISO 8601) | 盤前數據 timestamp       |
| `preMarketChangePercent` | number            | 盤前升跌 %               |

## 使用指引

### 多個 Symbols

用 comma 分隔 symbols，一次查多隻股票，最多 50 隻：

```bash
curl "https://stock-prices.on99.app/quotes?symbols=AAPL,GOOGL,MSFT,TSLA,AMZN"
```

### Error Handling

攞 data 前應該先確認 response 合法，並且先 decode TOON：

```typescript
import {decode} from "@toon-format/toon";

const response = await fetch("https://stock-prices.on99.app/quotes?symbols=NVDA");
const data = decode(await response.text());

if (data.quotes && data.quotes.length > 0) {
    const quote = data.quotes[0];
    console.log(`${quote.symbol}: $${quote.currentPrice}`);
}
```

### 價格分析

計常用指標：

```typescript
// 判斷股票升定跌
const isUp = quote.change > 0;
const direction = isUp ? "up" : "down";

// 計即日高低波幅 %
const rangePct = ((quote.highPrice - quote.lowPrice) / quote.lowPrice) * 100;

// 比較現價同開市價
const vsOpen = quote.currentPrice - quote.openPrice;
```

## 常見用途

### 1. 監察股價

```typescript
import {decode} from "@toon-format/toon";

async function checkPrice(symbol: string) {
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbol}`);
    const data = decode(await res.text());
    const quote = data.quotes[0];

    return {
        price: quote.currentPrice,
        change: quote.change,
        changePercent: quote.percentChange,
    };
}
```

### 2. Portfolio Tracking

```typescript
import {decode} from "@toon-format/toon";

async function getPortfolio(symbols: string[]) {
    const symbolString = symbols.join(",");
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbolString}`);
    const data = decode(await res.text());

    return data.quotes.map(q => ({
        symbol: q.symbol,
        value: q.currentPrice,
        dailyChange: q.percentChange,
    }));
}
```

### 3. 市場摘要

```typescript
import {decode} from "@toon-format/toon";

async function marketSummary(symbols: string[]) {
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbols.join(",")}`);
    const data = decode(await res.text());

    const gainers = data.quotes.filter(q => q.change > 0);
    const losers = data.quotes.filter(q => q.change < 0);

    return {
        totalStocks: data.quotes.length,
        gainers: gainers.length,
        losers: losers.length,
        avgChange: data.quotes.reduce((sum, q) => sum + q.percentChange, 0) / data.quotes.length,
    };
}
```

## 常用股票 Symbols

### Tech Giants (FAANG+)

- `AAPL` - Apple
- `GOOGL` - Alphabet (Google)
- `META` - Meta (Facebook)
- `AMZN` - Amazon
- `NFLX` - Netflix
- `MSFT` - Microsoft

### 高關注度股票

- `NVDA` - NVIDIA
- `TSLA` - Tesla
- `AMD` - Advanced Micro Devices
- `INTC` - Intel
- `ORCL` - Oracle

### 指數

- `^GSPC` - S&P 500
- `^DJI` - Dow Jones
- `^IXIC` - NASDAQ

## 注意事項

- **Response 格式**：API 回傳 TOON，唔係 JSON。請用 `@toon-format/toon` 嘅 `decode()` parse。
- 所有價格都係 USD
- 美股交易時段內數據會即時更新
- 支援盤前同盤後數據
- Timestamp 係 ISO 8601 格式，時區係 UTC
- 每次 request 最多 50 個 symbols
