import type {Response} from "express";
import {encode} from "@toon-format/toon";
import {StockPricesService} from "./stock-prices.service";
import {All, BadRequestException, Controller, Get, Inject, MethodNotAllowedException, Query, Res} from "@nestjs/common";

@Controller("quotes")
export class StockPricesController {
    private readonly stockPricesService: StockPricesService;

    public constructor(@Inject(StockPricesService) stockPricesService: StockPricesService) {
        this.stockPricesService = stockPricesService;
    }

    @Get()
    public async index(@Query("symbols") symbolsInput: string | undefined, @Res() response: Response): Promise<void> {
        const symbols = this.parseSymbols(symbolsInput);
        if (!symbols) {
            throw new BadRequestException("symbols is required");
        }

        const quotes = await this.stockPricesService.getQuotes(symbols);

        response.status(200).type("text/plain; charset=utf-8").send(encode({quotes}));
    }

    @All()
    public methodNotAllowed(): never {
        throw new MethodNotAllowedException();
    }

    private parseSymbols(input: string | undefined): string[] | undefined {
        if (!input) {
            return undefined;
        }

        const symbols = input
            .split(",")
            .map(symbol => symbol.trim())
            .filter(symbol => symbol !== "");

        return symbols.length > 0 ? symbols : undefined;
    }
}
