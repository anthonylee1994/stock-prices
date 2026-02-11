---
name: stock-prices
description: Query real-time stock prices and market data using the Stock Prices API. Use when fetching stock quotes, analyzing market data, or working with symbols like AAPL, NVDA, GOOGL, or any ticker symbols.
---

# Stock Prices API Skill

This skill helps you work with the Stock Prices API to fetch real-time market data and stock quotes.

## API Endpoint

**Base URL**: `https://stock-prices.on99.app`

**Primary Endpoint**: `/quotes?symbols={SYMBOLS}`

## Quick Start

Fetch stock quotes for one or more symbols:

```bash
curl "https://stock-prices.on99.app/quotes?symbols=NVDA"
curl "https://stock-prices.on99.app/quotes?symbols=AAPL,GOOGL,MSFT"
```

## Response Format

The API returns JSON with the following structure:

```json
{
    "quotes": [
        {
            "symbol": "NVDA",
            "currentPrice": 188.54,
            "change": -1.5,
            "percentChange": -0.789308,
            "highPrice": 192.48,
            "lowPrice": 188.12,
            "openPrice": 191.405,
            "previousClosePrice": 190.04,
            "preMarketPrice": 191.8799,
            "preMarketChange": 3.3399048,
            "preMarketTime": "2026-02-11T13:49:16.000Z",
            "preMarketChangePercent": 1.771457
        }
    ]
}
```

## Available Data Fields

| Field                    | Type              | Description                           |
| ------------------------ | ----------------- | ------------------------------------- |
| `symbol`                 | string            | Stock ticker symbol                   |
| `currentPrice`           | number            | Current trading price                 |
| `change`                 | number            | Price change from previous close      |
| `percentChange`          | number            | Percentage change from previous close |
| `highPrice`              | number            | Day's high price                      |
| `lowPrice`               | number            | Day's low price                       |
| `openPrice`              | number            | Opening price                         |
| `previousClosePrice`     | number            | Previous day's closing price          |
| `preMarketPrice`         | number            | Pre-market trading price              |
| `preMarketChange`        | number            | Pre-market price change               |
| `preMarketTime`          | string (ISO 8601) | Pre-market data timestamp             |
| `preMarketChangePercent` | number            | Pre-market percentage change          |

## Usage Guidelines

### Multiple Symbols

Query multiple stocks by separating symbols with commas (max 50):

```bash
curl "https://stock-prices.on99.app/quotes?symbols=AAPL,GOOGL,MSFT,TSLA,AMZN"
```

### Error Handling

Always check for valid responses:

```typescript
const response = await fetch("https://stock-prices.on99.app/quotes?symbols=NVDA");
const data = await response.json();

if (data.quotes && data.quotes.length > 0) {
    const quote = data.quotes[0];
    console.log(`${quote.symbol}: $${quote.currentPrice}`);
}
```

### Price Analysis

Calculate common metrics:

```typescript
// Determine if stock is up or down
const isUp = quote.change > 0;
const direction = isUp ? "📈" : "📉";

// Calculate day's range percentage
const rangePct = ((quote.highPrice - quote.lowPrice) / quote.lowPrice) * 100;

// Compare current to open
const vsOpen = quote.currentPrice - quote.openPrice;
```

## Common Use Cases

### 1. Price Monitoring

```typescript
async function checkPrice(symbol: string) {
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbol}`);
    const data = await res.json();
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
async function getPortfolio(symbols: string[]) {
    const symbolString = symbols.join(",");
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbolString}`);
    const data = await res.json();

    return data.quotes.map(q => ({
        symbol: q.symbol,
        value: q.currentPrice,
        dailyChange: q.percentChange,
    }));
}
```

### 3. Market Summary

```typescript
async function marketSummary(symbols: string[]) {
    const res = await fetch(`https://stock-prices.on99.app/quotes?symbols=${symbols.join(",")}`);
    const data = await res.json();

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

## Popular Stock Symbols

### Tech Giants (FAANG+)

- `AAPL` - Apple
- `GOOGL` - Alphabet (Google)
- `META` - Meta (Facebook)
- `AMZN` - Amazon
- `NFLX` - Netflix
- `MSFT` - Microsoft

### High-Profile Stocks

- `NVDA` - NVIDIA
- `TSLA` - Tesla
- `AMD` - Advanced Micro Devices
- `INTC` - Intel
- `ORCL` - Oracle

### Indices

- `^GSPC` - S&P 500
- `^DJI` - Dow Jones
- `^IXIC` - NASDAQ

## Notes

- All prices are in USD
- Data updates in real-time during market hours
- Pre-market and after-hours data is available
- Timestamps are in ISO 8601 format (UTC)
- Maximum 50 symbols per request
