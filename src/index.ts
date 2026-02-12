import cors from "cors";
import express, {Request, Response} from "express";
import YahooFinance from "yahoo-finance2";
import "dotenv/config";

const app = express();
const PORT = process.env.PORT || 3000;

const yahooFinance = new YahooFinance();

app.use(cors());
app.use(express.json());
app.use(express.urlencoded({extended: true}));

app.get("/", (_: Request, res: Response) => {
    res.json({
        message: "Stock Prices API",
        version: "1.0.0",
    });
});

app.get("/quotes", async (req: Request, res: Response) => {
    const symbols = req.query.symbols as string | undefined;
    if (!symbols) {
        return res.status(400).json({error: "symbols is required"});
    }
    const symbolsArray = symbols.split(",").map(s => s.trim());

    const quotes = await yahooFinance.quote(symbolsArray);

    res.json({
        quotes: quotes.map((quote, index) => ({
            symbol: symbolsArray[index],
            currentPrice: quote.regularMarketPrice,
            change: quote.regularMarketChange,
            percentChange: quote.regularMarketChangePercent,
            highPrice: quote.regularMarketDayHigh,
            lowPrice: quote.regularMarketDayLow,
            openPrice: quote.regularMarketOpen,
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
        })),
    });
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Server is running on http://localhost:${PORT}`);
    console.log(`📊 Stock Prices API is ready`);
});
