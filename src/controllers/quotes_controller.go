package controllers

import (
	"context"
	"errors"
	"log"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"

	"stock-prices/src/models"
	"stock-prices/src/services"
	"stock-prices/src/views"
)

type QuoteProvider interface {
	GetQuotes(ctx context.Context, symbols []string) ([]models.Quote, error)
}

type QuotesController struct {
	StockPrices QuoteProvider
}

func (controller QuotesController) Index(ctx *gin.Context) {
	symbols, ok := parseSymbols(ctx.Query("symbols"))
	if !ok {
		views.RenderJSONError(ctx, http.StatusBadRequest, "symbols is required")
		return
	}

	quotes, err := controller.StockPrices.GetQuotes(ctx.Request.Context(), symbols)
	if err != nil {
		log.Printf("Error fetching quotes: %v", err)
		views.RenderJSONError(ctx, statusFromError(err), "Failed to fetch quotes")
		return
	}

	views.RenderTOON(ctx, http.StatusOK, models.QuotesResponse{Quotes: quotes})
}

func parseSymbols(input string) ([]string, bool) {
	if input == "" {
		return nil, false
	}

	parts := strings.Split(input, ",")
	symbols := make([]string, 0, len(parts))
	for _, part := range parts {
		symbol := strings.TrimSpace(part)
		if symbol != "" {
			symbols = append(symbols, symbol)
		}
	}

	return symbols, len(symbols) > 0
}

func statusFromError(err error) int {
	var apiErr *services.APIError
	if errors.As(err, &apiErr) {
		return apiErr.Status
	}

	return http.StatusBadGateway
}
