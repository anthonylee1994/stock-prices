package app

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"slices"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"

	"stock-prices/src/models"
)

func init() {
	gin.SetMode(gin.TestMode)
}

type fakeQuoteProvider struct {
	quotes  []models.Quote
	symbols []string
	err     error
}

func (provider *fakeQuoteProvider) GetQuotes(_ context.Context, symbols []string) ([]models.Quote, error) {
	provider.symbols = append([]string(nil), symbols...)
	return provider.quotes, provider.err
}

func TestRootReturnsTOON(t *testing.T) {
	app := NewRouter(&fakeQuoteProvider{}, "1.0.0")
	request := httptest.NewRequest(http.MethodGet, "/", nil)
	response := httptest.NewRecorder()

	app.ServeHTTP(response, request)

	assertStatus(t, response, http.StatusOK)
	assertHeader(t, response, "Content-Type", "text/plain; charset=utf-8")
	assertEqual(t, response.Body.String(), "message: Stock Prices API\nversion: 1.0.0")
}

func TestQuotesReturnsTOON(t *testing.T) {
	name := "NVIDIA Corporation"
	market := "us_market"
	currentPrice := 188.54
	change := -1.5
	regularMarketTime := "2026-02-11T13:49:16.000Z"
	provider := &fakeQuoteProvider{
		quotes: []models.Quote{{
			Symbol:            "NVDA",
			Name:              &name,
			Market:            &market,
			CurrentPrice:      &currentPrice,
			Change:            &change,
			RegularMarketTime: &regularMarketTime,
		}},
	}
	app := NewRouter(provider, "1.0.0")
	request := httptest.NewRequest(http.MethodGet, "/quotes?symbols=NVDA,%20MSFT", nil)
	response := httptest.NewRecorder()

	app.ServeHTTP(response, request)

	assertStatus(t, response, http.StatusOK)
	assertStringSliceEqual(t, provider.symbols, []string{"NVDA", "MSFT"})

	body := response.Body.String()
	if !strings.HasPrefix(body, "quotes[1]{symbol,name,market,currentPrice") {
		t.Fatalf("expected TOON quote table, got %q", body)
	}
	if !strings.Contains(body, "\n  NVDA,NVIDIA Corporation,us_market,188.54,-1.5,") {
		t.Fatalf("expected quote row, got %q", body)
	}
	if !strings.Contains(body, "forwardPE") {
		t.Fatalf("expected forwardPE column, got %q", body)
	}
}

func TestQuotesRequiresSymbols(t *testing.T) {
	app := NewRouter(&fakeQuoteProvider{}, "1.0.0")
	request := httptest.NewRequest(http.MethodGet, "/quotes?symbols=,%20", nil)
	response := httptest.NewRecorder()

	app.ServeHTTP(response, request)

	assertStatus(t, response, http.StatusBadRequest)
	assertHeader(t, response, "Content-Type", "application/json")
	assertEqual(t, response.Body.String(), `{"error":"symbols is required"}`)
}

func TestQuotesMapsProviderErrorsToBadGateway(t *testing.T) {
	app := NewRouter(&fakeQuoteProvider{err: errors.New("upstream failed")}, "1.0.0")
	request := httptest.NewRequest(http.MethodGet, "/quotes?symbols=NVDA", nil)
	response := httptest.NewRecorder()

	app.ServeHTTP(response, request)

	assertStatus(t, response, http.StatusBadGateway)
	assertEqual(t, response.Body.String(), `{"error":"Failed to fetch quotes"}`)
}

func assertStatus(t *testing.T, response *httptest.ResponseRecorder, expected int) {
	t.Helper()
	if response.Code != expected {
		t.Fatalf("expected status %d, got %d", expected, response.Code)
	}
}

func assertHeader(t *testing.T, response *httptest.ResponseRecorder, name string, expected string) {
	t.Helper()
	if actual := response.Header().Get(name); actual != expected {
		t.Fatalf("expected %s header %q, got %q", name, expected, actual)
	}
}

func assertEqual[T comparable](t *testing.T, actual T, expected T) {
	t.Helper()
	if actual != expected {
		t.Fatalf("expected %#v, got %#v", expected, actual)
	}
}

func assertStringSliceEqual(t *testing.T, actual []string, expected []string) {
	t.Helper()
	if !slices.Equal(actual, expected) {
		t.Fatalf("expected %#v, got %#v", expected, actual)
	}
}
