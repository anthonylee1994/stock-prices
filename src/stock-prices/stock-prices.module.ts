import {Module} from "@nestjs/common";
import {StockPricesService} from "./stock-prices.service";
import {StockPricesController} from "./stock-prices.controller";

@Module({
    controllers: [StockPricesController],
    providers: [StockPricesService],
})
export class StockPricesModule {}
