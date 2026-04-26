import type {Response} from "express";
import {encode} from "@toon-format/toon";
import {BadRequestException, Controller, Get, Query, Res} from "@nestjs/common";
import {StockPricesService} from "./stock-prices.service";

@Controller()
export class StockPricesController {
    constructor(private readonly stockPricesService: StockPricesService) {}

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
