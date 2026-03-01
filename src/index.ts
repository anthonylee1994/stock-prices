import cors from "cors";
import express, {Request, Response} from "express";
import {encode} from "@toon-format/toon";
import "dotenv/config";
import {getQuotes} from "./service";

const app = express();
const PORT = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());
app.use(express.urlencoded({extended: true}));

app.get("/", (_: Request, res: Response) => {
    res.send(
        encode({
            message: "Stock Prices API",
            version: "1.0.0",
        })
    );
});

app.get("/quotes", async (req: Request, res: Response) => {
    try {
        const symbols = req.query.symbols as string | undefined;
        if (!symbols) {
            return res.status(400).json({error: "symbols is required"});
        }
        const symbolsArray = symbols.split(",").map(s => s.trim());
        const quotes = await getQuotes(symbolsArray);

        res.send(
            encode({
                quotes,
            })
        );
    } catch (error) {
        res.status(500).json({error: "Failed to fetch quotes"});
    }
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Server is running on http://localhost:${PORT}`);
    console.log(`📊 Stock Prices API is ready`);
});
