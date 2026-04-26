import {Module} from "@nestjs/common";
import {StockPricesService} from "./stock-prices.service";

@Module({
    providers: [StockPricesService],
    exports: [StockPricesService],
})
export class StockPricesModule {}
