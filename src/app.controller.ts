import type {Response} from "express";
import {encode} from "@toon-format/toon";
import {StockPricesService} from "./stock-prices/stock-prices.service";
import {BadRequestException, Controller, Get, Query, Res} from "@nestjs/common";

@Controller()
export class AppController {
    constructor(private readonly stockPricesService: StockPricesService) {}

    @Get()
    getRoot(@Res() res: Response): void {
        res.send(encode({message: "Stock Prices API", version: "1.0.0"}));
    }

    @Get("quotes")
    async getQuotes(@Query("symbols") symbols: string | undefined, @Res() res: Response): Promise<void> {
        if (!symbols) {
            throw new BadRequestException({error: "symbols is required"});
        }

        try {
            const symbolsArray = symbols.split(",").map(symbol => symbol.trim());
            const quotes = await this.stockPricesService.getQuotes(symbolsArray);

            res.send(encode({quotes}));
        } catch (error) {
            console.error("Error fetching quotes:", error);
        }
    }
}
