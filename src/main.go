package main

import (
	"log"
	"os"

	"github.com/gin-gonic/gin"

	"stock-prices/src/app"
	"stock-prices/src/services"
)

const version = "1.0.0"

func main() {
	gin.SetMode(gin.ReleaseMode)

	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}

	router := app.NewRouter(services.NewStockPricesService(), version)

	log.Printf("Server is running on http://localhost:%s", port)
	log.Fatal(router.Run(":" + port))
}
