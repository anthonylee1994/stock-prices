package models

type RootResponse struct {
	Message string `toon:"message"`
	Version string `toon:"version"`
}

type QuotesResponse struct {
	Quotes []Quote `toon:"quotes"`
}

type ErrorResponse struct {
	Error string `json:"error"`
}
