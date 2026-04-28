package services

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/cookiejar"
	"strings"
	"sync"
	"time"

	"stock-prices/src/models"
)

const userAgent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"

type StockPricesService struct {
	client *http.Client
	mu     sync.Mutex
	crumb  string
}

func NewStockPricesService() *StockPricesService {
	jar, err := cookiejar.New(nil)
	if err != nil {
		panic(fmt.Sprintf("cookie jar config should be valid: %v", err))
	}

	return &StockPricesService{
		client: &http.Client{
			Jar:     jar,
			Timeout: 15 * time.Second,
		},
	}
}

func (service *StockPricesService) GetQuotes(ctx context.Context, symbols []string) ([]models.Quote, error) {
	response, err := service.sendQuotesRequest(ctx, symbols, false)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()

	if response.StatusCode == http.StatusUnauthorized || response.StatusCode == http.StatusForbidden {
		_, _ = io.Copy(io.Discard, response.Body)
		service.clearCrumb()

		response, err = service.sendQuotesRequest(ctx, symbols, true)
		if err != nil {
			return nil, err
		}
		defer response.Body.Close()
	}

	if response.StatusCode < 200 || response.StatusCode >= 300 {
		return nil, badGateway("Yahoo Finance returned status %s", response.Status)
	}

	var yahooResponse yahooQuoteResponse
	if err := json.NewDecoder(response.Body).Decode(&yahooResponse); err != nil {
		return nil, badGateway("failed to decode Yahoo Finance response: %w", err)
	}

	quotes := make([]models.Quote, 0, len(yahooResponse.QuoteResponse.Result))
	for _, yahooQuote := range yahooResponse.QuoteResponse.Result {
		quotes = append(quotes, newQuote(yahooQuote))
	}

	return quotes, nil
}

func (service *StockPricesService) sendQuotesRequest(ctx context.Context, symbols []string, forceCrumbRefresh bool) (*http.Response, error) {
	crumb, err := service.getCrumb(ctx, forceCrumbRefresh)
	if err != nil {
		return nil, err
	}

	request, err := http.NewRequestWithContext(ctx, http.MethodGet, "https://query1.finance.yahoo.com/v7/finance/quote", nil)
	if err != nil {
		return nil, badGateway("failed to create Yahoo Finance quote request: %w", err)
	}

	query := request.URL.Query()
	query.Set("symbols", strings.Join(symbols, ","))
	query.Set("lang", "zh-HK")
	query.Set("region", "HK")
	query.Set("crumb", crumb)
	request.URL.RawQuery = query.Encode()
	request.Header.Set("User-Agent", userAgent)

	response, err := service.client.Do(request)
	if err != nil {
		return nil, badGateway("failed to fetch quotes: %w", err)
	}

	return response, nil
}

func (service *StockPricesService) getCrumb(ctx context.Context, forceRefresh bool) (string, error) {
	if !forceRefresh {
		service.mu.Lock()
		crumb := service.crumb
		service.mu.Unlock()

		if crumb != "" {
			return crumb, nil
		}
	}

	if err := service.sendCrumbWarmupRequest(ctx); err != nil {
		return "", err
	}

	crumb, err := service.fetchCrumb(ctx)
	if err != nil {
		return "", err
	}

	crumb = strings.TrimSpace(crumb)
	if crumb == "" || strings.Contains(crumb, "Too Many Requests") {
		return "", badGateway("Yahoo Finance did not return a valid crumb")
	}

	service.mu.Lock()
	service.crumb = crumb
	service.mu.Unlock()

	return crumb, nil
}

func (service *StockPricesService) sendCrumbWarmupRequest(ctx context.Context) error {
	request, err := http.NewRequestWithContext(ctx, http.MethodGet, "https://fc.yahoo.com", nil)
	if err != nil {
		return badGateway("failed to create Yahoo Finance warmup request: %w", err)
	}
	request.Header.Set("User-Agent", userAgent)

	response, err := service.client.Do(request)
	if err != nil {
		return badGateway("failed to fetch Yahoo Finance cookies: %w", err)
	}
	defer response.Body.Close()

	_, _ = io.Copy(io.Discard, response.Body)
	return nil
}

func (service *StockPricesService) fetchCrumb(ctx context.Context) (string, error) {
	request, err := http.NewRequestWithContext(ctx, http.MethodGet, "https://query1.finance.yahoo.com/v1/test/getcrumb", nil)
	if err != nil {
		return "", badGateway("failed to create Yahoo Finance crumb request: %w", err)
	}
	request.Header.Set("User-Agent", userAgent)

	response, err := service.client.Do(request)
	if err != nil {
		return "", badGateway("failed to fetch Yahoo Finance crumb: %w", err)
	}
	defer response.Body.Close()

	if response.StatusCode < 200 || response.StatusCode >= 300 {
		_, _ = io.Copy(io.Discard, response.Body)
		return "", badGateway("Yahoo Finance crumb endpoint returned status %s", response.Status)
	}

	body, err := io.ReadAll(response.Body)
	if err != nil {
		return "", badGateway("failed to read Yahoo Finance crumb: %w", err)
	}

	return string(body), nil
}

func (service *StockPricesService) clearCrumb() {
	service.mu.Lock()
	service.crumb = ""
	service.mu.Unlock()
}

type yahooQuoteResponse struct {
	QuoteResponse yahooQuoteResult `json:"quoteResponse"`
}

type yahooQuoteResult struct {
	Result []yahooQuote `json:"result"`
}

type yahooQuote struct {
	Symbol                     string   `json:"symbol"`
	LongName                   *string  `json:"longName"`
	Market                     *string  `json:"market"`
	RegularMarketPrice         *float64 `json:"regularMarketPrice"`
	RegularMarketChange        *float64 `json:"regularMarketChange"`
	RegularMarketChangePercent *float64 `json:"regularMarketChangePercent"`
	RegularMarketDayHigh       *float64 `json:"regularMarketDayHigh"`
	RegularMarketDayLow        *float64 `json:"regularMarketDayLow"`
	RegularMarketOpen          *float64 `json:"regularMarketOpen"`
	RegularMarketTime          *int64   `json:"regularMarketTime"`
	RegularMarketPreviousClose *float64 `json:"regularMarketPreviousClose"`
	PreMarketPrice             *float64 `json:"preMarketPrice"`
	PreMarketChange            *float64 `json:"preMarketChange"`
	PreMarketTime              *int64   `json:"preMarketTime"`
	PreMarketChangePercent     *float64 `json:"preMarketChangePercent"`
	PostMarketPrice            *float64 `json:"postMarketPrice"`
	PostMarketChange           *float64 `json:"postMarketChange"`
	PostMarketChangePercent    *float64 `json:"postMarketChangePercent"`
	PostMarketTime             *int64   `json:"postMarketTime"`
	ForwardPE                  *float64 `json:"forwardPE"`
	PriceToBook                *float64 `json:"priceToBook"`
	DividendYield              *float64 `json:"dividendYield"`
}

func newQuote(quote yahooQuote) models.Quote {
	return models.Quote{
		Symbol:                  quote.Symbol,
		Name:                    quote.LongName,
		Market:                  quote.Market,
		CurrentPrice:            quote.RegularMarketPrice,
		Change:                  quote.RegularMarketChange,
		PercentChange:           quote.RegularMarketChangePercent,
		HighPrice:               quote.RegularMarketDayHigh,
		LowPrice:                quote.RegularMarketDayLow,
		OpenPrice:               quote.RegularMarketOpen,
		RegularMarketTime:       epochToISO(quote.RegularMarketTime),
		PreviousClosePrice:      quote.RegularMarketPreviousClose,
		PreMarketPrice:          quote.PreMarketPrice,
		PreMarketChange:         quote.PreMarketChange,
		PreMarketTime:           epochToISO(quote.PreMarketTime),
		PreMarketChangePercent:  quote.PreMarketChangePercent,
		PostMarketPrice:         quote.PostMarketPrice,
		PostMarketChange:        quote.PostMarketChange,
		PostMarketChangePercent: quote.PostMarketChangePercent,
		PostMarketTime:          epochToISO(quote.PostMarketTime),
		ForwardPE:               quote.ForwardPE,
		PriceToBook:             quote.PriceToBook,
		DividendYield:           quote.DividendYield,
	}
}

func epochToISO(epochSeconds *int64) *string {
	if epochSeconds == nil {
		return nil
	}

	value := time.Unix(*epochSeconds, 0).UTC().Format("2006-01-02T15:04:05.000Z")
	return &value
}
