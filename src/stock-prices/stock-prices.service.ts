import YahooFinance from "yahoo-finance2";
import {Quote} from "./stock-prices.type";
import {BadGatewayException, Injectable} from "@nestjs/common";

@Injectable()
export class StockPricesService {
    private readonly yahooFinance = new YahooFinance({
        suppressNotices: ["yahooSurvey"],
    });

    public async getQuotes(symbols: string[]): Promise<Quote[]> {
        try {
            const quotes = await this.yahooFinance.quote(symbols, {lang: "zh-HK", region: "HK"});
            return quotes.map(quote => ({
                symbol: quote.symbol,
                name: quote.longName,
                market: quote.market,
                currentPrice: quote.regularMarketPrice,
                change: quote.regularMarketChange,
                percentChange: quote.regularMarketChangePercent,
                highPrice: quote.regularMarketDayHigh,
                lowPrice: quote.regularMarketDayLow,
                openPrice: quote.regularMarketOpen,
                regularMarketTime: quote.regularMarketTime,
                previousClosePrice: quote.regularMarketPreviousClose,
                preMarketPrice: quote.preMarketPrice,
                preMarketChange: quote.preMarketChange,
                preMarketTime: quote.preMarketTime,
                preMarketChangePercent: quote.preMarketChangePercent,
                postMarketPrice: quote.postMarketPrice,
                postMarketChange: quote.postMarketChange,
                postMarketChangePercent: quote.postMarketChangePercent,
                postMarketTime: quote.postMarketTime,
                forwardPE: quote.forwardPE,
                priceToBook: quote.priceToBook,
                dividendYield: quote.dividendYield,
            }));
        } catch (error) {
            throw new BadGatewayException("Failed to fetch quotes", {cause: error});
        }
    }
}
