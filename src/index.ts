import axios from "axios";
import cors from "cors";
import express, {Request, Response} from "express";

const app = express();
const PORT = process.env.PORT || 3000;

const FINNHUB_API_TOKEN = "d5tjcrhr01qt62njnql0d5tjcrhr01qt62njnqlg";

const QUOTE_CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes
const quoteCache = new Map<string, {data: Record<string, unknown>; expiresAt: number}>();

function getCacheKey(symbols: string): string {
    return symbols
        .split(",")
        .map(s => s.trim().toUpperCase())
        .filter(Boolean)
        .join(",");
}

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
    const cacheKey = getCacheKey(symbols);

    const cached = quoteCache.get(cacheKey);
    if (cached && cached.expiresAt > Date.now()) {
        return res.json(cached.data);
    }

    const quotes = await Promise.all(
        symbolsArray.map(symbol => {
            return axios.get(`https://finnhub.io/api/v1/quote?symbol=${symbol}&token=${FINNHUB_API_TOKEN}`);
        })
    );

    const responseData = {
        quotes: quotes.map((quote, index) => ({
            symbol: symbolsArray[index],
            currentPrice: quote.data.c,
            change: quote.data.d,
            percentChange: quote.data.dp,
            highPrice: quote.data.h,
            lowPrice: quote.data.l,
            openPrice: quote.data.o,
            previousClosePrice: quote.data.pc,
        })),
    };

    quoteCache.set(cacheKey, {
        data: responseData,
        expiresAt: Date.now() + QUOTE_CACHE_TTL_MS,
    });

    res.json(responseData);
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Server is running on http://localhost:${PORT}`);
    console.log(`📊 Stock Prices API is ready`);
});
