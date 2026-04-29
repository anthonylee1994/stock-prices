import {Module} from "@nestjs/common";
import {RootController} from "./app.controller";
import {StockPricesModule} from "./stock-prices/stock-prices.module";

@Module({
    imports: [StockPricesModule],
    controllers: [RootController],
})
export class AppModule {}
