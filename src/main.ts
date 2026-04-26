import "dotenv/config";
import "reflect-metadata";
import {NestFactory} from "@nestjs/core";
import {AppModule} from "./app.module";

const PORT = process.env.PORT || 3000;

async function bootstrap(): Promise<void> {
    const app = await NestFactory.create(AppModule);

    app.enableCors();

    await app.listen(PORT);
    console.log(`🚀 Server is running on http://localhost:${PORT}`);
    console.log(`📊 Stock Prices API is ready`);
}

void bootstrap();
