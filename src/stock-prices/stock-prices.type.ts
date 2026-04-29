export interface Quote {
    symbol: string;
    name?: string;
    market?: string;
    currentPrice?: number;
    change?: number;
    percentChange?: number;
    highPrice?: number;
    lowPrice?: number;
    openPrice?: number;
    regularMarketTime?: string;
    previousClosePrice?: number;
    preMarketPrice?: number;
    preMarketChange?: number;
    preMarketTime?: string;
    preMarketChangePercent?: number;
    postMarketPrice?: number;
    postMarketChange?: number;
    postMarketChangePercent?: number;
    postMarketTime?: string;
    forwardPE?: number;
    priceToBook?: number;
    dividendYield?: number;
}
