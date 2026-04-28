package services

import "fmt"

type APIError struct {
	Status int
	Err    error
}

func (err *APIError) Error() string {
	return err.Err.Error()
}

func badGateway(format string, args ...any) *APIError {
	return &APIError{
		Status: 502,
		Err:    fmt.Errorf(format, args...),
	}
}
