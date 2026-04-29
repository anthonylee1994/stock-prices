import request from "supertest";
import {Test} from "@nestjs/testing";
import type {App} from "supertest/types";
import {encode} from "@toon-format/toon";
import {AppModule} from "../src/app.module";
import type {INestApplication} from "@nestjs/common";
import type {Quote} from "../src/stock-prices/stock-prices.type";
import {StockPricesService} from "../src/stock-prices/stock-prices.service";

describe("App (e2e)", function () {
    const quotes: Quote[] = [
        {
            symbol: "AAPL",
            name: "Apple Inc.",
            market: "us_market",
            currentPrice: 230.12,
        },
    ];

    let app: INestApplication<App>;
    let stockPricesService: jest.Mocked<Pick<StockPricesService, "getQuotes">>;

    beforeAll(async function () {
        stockPricesService = {
            getQuotes: jest.fn().mockResolvedValue(quotes),
        };

        const moduleFixture = await Test.createTestingModule({
            imports: [AppModule],
        })
            .overrideProvider(StockPricesService)
            .useValue(stockPricesService)
            .compile();

        app = moduleFixture.createNestApplication();
        await app.init();
    });

    beforeEach(function () {
        stockPricesService.getQuotes.mockClear();
    });

    afterAll(async function () {
        await app.close();
    });

    it("returns API metadata", function () {
        return request(app.getHttpServer())
            .get("/")
            .expect(200)
            .expect("content-type", "text/plain; charset=utf-8")
            .expect(encode({message: "Stock Prices API", version: "1.0.0"}));
    });

    it("returns encoded quotes", async function () {
        await request(app.getHttpServer()).get("/quotes").query({symbols: " AAPL, MSFT ,, 0700.HK "}).expect(200).expect("content-type", "text/plain; charset=utf-8").expect(encode({quotes}));

        expect(stockPricesService.getQuotes).toHaveBeenCalledWith(["AAPL", "MSFT", "0700.HK"]);
    });

    it("rejects missing symbols", async function () {
        await request(app.getHttpServer()).get("/quotes").expect(400);

        expect(stockPricesService.getQuotes).not.toHaveBeenCalled();
    });

    it("rejects unsupported quote methods", async function () {
        await request(app.getHttpServer()).post("/quotes").expect(405);
    });
});
