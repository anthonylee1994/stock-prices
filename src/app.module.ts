import {Module} from "@nestjs/common";
import {AppController} from "./app.controller";
import {StockPricesModule} from "./stock-prices/stock-prices.module";

@Module({
    controllers: [AppController],
    imports: [StockPricesModule],
})
export class AppModule {}
