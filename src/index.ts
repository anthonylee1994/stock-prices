import axios from "axios";
import express, {Request, Response} from "express";

const app = express();
const PORT = process.env.PORT || 3000;

const FINNHUB_API_TOKEN = "d5tjcrhr01qt62njnql0d5tjcrhr01qt62njnqlg";

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
    const symbolsArray = symbols.split(",");

    const quotes = await Promise.all(
        symbolsArray.map(symbol => {
            return axios.get(`https://finnhub.io/api/v1/quote?symbol=${symbol}&token=${FINNHUB_API_TOKEN}`);
        })
    );

    res.json({
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
    });
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Server is running on http://localhost:${PORT}`);
    console.log(`📊 Stock Prices API is ready`);
});
