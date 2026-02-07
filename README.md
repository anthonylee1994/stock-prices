# Stock Prices API

A starter Express.js API built with TypeScript and managed with pnpm.

## Features

- ✅ Express.js 5.x
- ✅ TypeScript
- ✅ Hot reload with Nodemon
- ✅ Type-safe development
- ✅ Sample stock price endpoints
- ✅ Error handling middleware
- ✅ Request logging

## Prerequisites

- Node.js (v18 or higher)
- pnpm

## Installation

```bash
pnpm install
```

## Development

Start the development server with hot reload:

```bash
pnpm dev
```

The server will start on `http://localhost:3000`

## Build

Compile TypeScript to JavaScript:

```bash
pnpm build
```

## Production

Run the compiled JavaScript:

```bash
pnpm start
```

## API Endpoints

### Root

- `GET /` - API information and available endpoints

### Stocks

- `GET /api/quotes?symbols=AAPL,GOOGL,MSFT` - Get multiple stock quotes (comma-separated, max 50)

### Examples

```bash
# Multiple quotes via query params
curl "http://localhost:3000/api/quotes?symbols=AAPL,GOOGL,MSFT"
```

## Project Structure

```
stock-prices/
├── src/
│   └── index.ts         # Main application file
├── dist/                # Compiled JavaScript (generated)
├── node_modules/        # Dependencies
├── package.json         # Project configuration
├── tsconfig.json        # TypeScript configuration
├── nodemon.json         # Nodemon configuration
└── README.md           # This file
```

## Scripts

- `pnpm dev` - Start development server with hot reload
- `pnpm build` - Build the project
- `pnpm start` - Run production build
- `pnpm clean` - Remove build directory
- `pnpm rebuild` - Clean and rebuild project
- `pnpm format` - Format code with Prettier
- `pnpm format:check` - Check code formatting

## Environment Variables

Create a `.env` file in the root directory:

```
PORT=3000
```

### Other Platforms

The project can also be deployed to:

- Heroku
- Railway
- Render
- Fly.io
- DigitalOcean App Platform

All use the same `Procfile` and build configuration.

## Next Steps

1. Add database integration (PostgreSQL, MongoDB, etc.)
2. Implement authentication & authorization
3. Add data validation with Zod or Joi
4. Set up testing with Jest or Vitest
5. Add API documentation with Swagger/OpenAPI
6. Implement real stock price data integration (Alpha Vantage, Finnhub, etc.)
7. Add rate limiting and security middleware (helmet, express-rate-limit)
8. Set up CI/CD pipeline
9. Add monitoring and error tracking

## License

MIT
