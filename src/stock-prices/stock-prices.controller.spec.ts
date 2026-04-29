import {BadRequestException, MethodNotAllowedException} from "@nestjs/common";
import {Test} from "@nestjs/testing";
import {encode} from "@toon-format/toon";
import {StockPricesController} from "./stock-prices.controller";
import {StockPricesService} from "./stock-prices.service";
import {Quote} from "./stock-prices.type";

describe("StockPricesController", () => {
    const quotes: Quote[] = [
        {
            symbol: "AAPL",
            name: "Apple Inc.",
            market: "us_market",
            currentPrice: 230.12,
        },
    ];

    function createResponse() {
        const response = {
            send: jest.fn(),
            status: jest.fn(),
            type: jest.fn(),
        };

        response.status.mockReturnValue(response);
        response.type.mockReturnValue(response);

        return response;
    }

    async function createController() {
        const stockPricesService: jest.Mocked<Pick<StockPricesService, "getQuotes">> = {
            getQuotes: jest.fn().mockResolvedValue(quotes),
        };
        const moduleRef = await Test.createTestingModule({
            controllers: [StockPricesController],
            providers: [
                {
                    provide: StockPricesService,
                    useValue: stockPricesService,
                },
            ],
        }).compile();

        return {
            controller: moduleRef.get(StockPricesController),
            stockPricesService,
        };
    }

    it("returns encoded quotes for comma-separated symbols", async () => {
        const {controller, stockPricesService} = await createController();
        const response = createResponse();

        await controller.index(" AAPL, MSFT ,, 0700.HK ", response as never);

        expect(stockPricesService.getQuotes).toHaveBeenCalledWith(["AAPL", "MSFT", "0700.HK"]);
        expect(response.status).toHaveBeenCalledWith(200);
        expect(response.type).toHaveBeenCalledWith("text/plain; charset=utf-8");
        expect(response.send).toHaveBeenCalledWith(encode({quotes}));
    });

    it("rejects missing symbols", async () => {
        const {controller, stockPricesService} = await createController();
        const response = createResponse();

        await expect(controller.index(undefined, response as never)).rejects.toBeInstanceOf(BadRequestException);

        expect(stockPricesService.getQuotes).not.toHaveBeenCalled();
    });

    it("rejects blank symbol lists", async () => {
        const {controller, stockPricesService} = await createController();
        const response = createResponse();

        await expect(controller.index(" , , ", response as never)).rejects.toBeInstanceOf(BadRequestException);

        expect(stockPricesService.getQuotes).not.toHaveBeenCalled();
    });

    it("rejects unsupported methods", async () => {
        const {controller} = await createController();

        expect(() => controller.methodNotAllowed()).toThrow(MethodNotAllowedException);
    });
});
