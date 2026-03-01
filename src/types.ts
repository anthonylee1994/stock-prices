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
    previousClosePrice?: number;
    regularMarketTime?: Date;
    preMarketPrice?: number;
    preMarketChange?: number;
    preMarketChangePercent?: number;
    preMarketTime?: Date;
    postMarketPrice?: number;
    postMarketChange?: number;
    postMarketChangePercent?: number;
    postMarketTime?: Date;
    forwardPE?: number;
    priceToBook?: number;
    dividendYield?: number;
}
