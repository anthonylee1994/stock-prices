package views

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
	toon "github.com/toon-format/toon-go"

	"stock-prices/src/models"
)

func RenderTOON(ctx *gin.Context, status int, value any) {
	body, err := toon.MarshalString(value)
	if err != nil {
		log.Printf("Error encoding TOON response: %v", err)
		RenderJSONError(ctx, http.StatusInternalServerError, "Failed to encode response")
		return
	}

	ctx.Data(status, "text/plain; charset=utf-8", []byte(body))
}

func RenderJSONError(ctx *gin.Context, status int, message string) {
	body, err := json.Marshal(models.ErrorResponse{Error: message})
	if err != nil {
		ctx.Status(status)
		return
	}

	ctx.Data(status, "application/json", body)
}
