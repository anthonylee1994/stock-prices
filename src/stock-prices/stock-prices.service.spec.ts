import {Test} from "@nestjs/testing";
import {BadGatewayException} from "@nestjs/common";
import {StockPricesService} from "./stock-prices.service";

describe("StockPricesService", () => {
    async function createService() {
        const moduleRef = await Test.createTestingModule({
            providers: [StockPricesService],
        }).compile();
        const service = moduleRef.get(StockPricesService);
        const yahooFinance = {
            quote: jest.fn(),
        };

        Reflect.set(service, "yahooFinance", yahooFinance);

        return {service, yahooFinance};
    }

    it("maps Yahoo Finance quotes to API quotes", async () => {
        const {service, yahooFinance} = await createService();
        const regularMarketTime = new Date("2026-04-29T09:30:00.000Z");
        const preMarketTime = new Date("2026-04-29T08:00:00.000Z");
        const postMarketTime = new Date("2026-04-29T20:00:00.000Z");

        yahooFinance.quote.mockResolvedValue([
            {
                symbol: "AAPL",
                longName: "Apple Inc.",
                market: "us_market",
                regularMarketPrice: 230.12,
                regularMarketChange: 1.23,
                regularMarketChangePercent: 0.54,
                regularMarketDayHigh: 231,
                regularMarketDayLow: 228.4,
                regularMarketOpen: 229,
                regularMarketTime,
                regularMarketPreviousClose: 228.89,
                preMarketPrice: 229.5,
                preMarketChange: 0.61,
                preMarketTime,
                preMarketChangePercent: 0.27,
                postMarketPrice: 230.4,
                postMarketChange: 0.28,
                postMarketChangePercent: 0.12,
                postMarketTime,
                forwardPE: 28.5,
                priceToBook: 45.3,
                dividendYield: 0.005,
            },
        ]);

        await expect(service.getQuotes(["AAPL"])).resolves.toEqual([
            {
                symbol: "AAPL",
                name: "Apple Inc.",
                market: "us_market",
                currentPrice: 230.12,
                change: 1.23,
                percentChange: 0.54,
                highPrice: 231,
                lowPrice: 228.4,
                openPrice: 229,
                regularMarketTime,
                previousClosePrice: 228.89,
                preMarketPrice: 229.5,
                preMarketChange: 0.61,
                preMarketTime,
                preMarketChangePercent: 0.27,
                postMarketPrice: 230.4,
                postMarketChange: 0.28,
                postMarketChangePercent: 0.12,
                postMarketTime,
                forwardPE: 28.5,
                priceToBook: 45.3,
                dividendYield: 0.005,
            },
        ]);
        expect(yahooFinance.quote).toHaveBeenCalledWith(["AAPL"], {lang: "zh-HK", region: "HK"});
    });

    it("wraps Yahoo Finance failures", async () => {
        const {service, yahooFinance} = await createService();
        const error = new Error("network failed");

        yahooFinance.quote.mockRejectedValue(error);

        await expect(service.getQuotes(["AAPL"])).rejects.toMatchObject({
            message: "Failed to fetch quotes",
            cause: error,
        });
        await expect(service.getQuotes(["AAPL"])).rejects.toBeInstanceOf(BadGatewayException);
    });
});
