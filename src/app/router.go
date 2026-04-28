package app

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"stock-prices/src/controllers"
	"stock-prices/src/views"
)

func NewRouter(stockPrices controllers.QuoteProvider, version string) *gin.Engine {
	rootController := controllers.RootController{
		Version: version,
	}
	quotesController := controllers.QuotesController{
		StockPrices: stockPrices,
	}

	router := gin.New()
	router.HandleMethodNotAllowed = true
	router.Use(gin.Recovery(), corsMiddleware())

	router.GET("/", rootController.Show)
	router.GET("/quotes", quotesController.Index)
	router.NoMethod(func(ctx *gin.Context) {
		views.RenderJSONError(ctx, http.StatusMethodNotAllowed, "method not allowed")
	})

	return router
}
