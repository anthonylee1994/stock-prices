package models

type Quote struct {
	Symbol                  string   `toon:"symbol"`
	Name                    *string  `toon:"name"`
	Market                  *string  `toon:"market"`
	CurrentPrice            *float64 `toon:"currentPrice"`
	Change                  *float64 `toon:"change"`
	PercentChange           *float64 `toon:"percentChange"`
	HighPrice               *float64 `toon:"highPrice"`
	LowPrice                *float64 `toon:"lowPrice"`
	OpenPrice               *float64 `toon:"openPrice"`
	RegularMarketTime       *string  `toon:"regularMarketTime"`
	PreviousClosePrice      *float64 `toon:"previousClosePrice"`
	PreMarketPrice          *float64 `toon:"preMarketPrice"`
	PreMarketChange         *float64 `toon:"preMarketChange"`
	PreMarketTime           *string  `toon:"preMarketTime"`
	PreMarketChangePercent  *float64 `toon:"preMarketChangePercent"`
	PostMarketPrice         *float64 `toon:"postMarketPrice"`
	PostMarketChange        *float64 `toon:"postMarketChange"`
	PostMarketChangePercent *float64 `toon:"postMarketChangePercent"`
	PostMarketTime          *string  `toon:"postMarketTime"`
	ForwardPE               *float64 `toon:"forwardPE"`
	PriceToBook             *float64 `toon:"priceToBook"`
	DividendYield           *float64 `toon:"dividendYield"`
}
