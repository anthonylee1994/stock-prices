package controllers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"stock-prices/src/models"
	"stock-prices/src/views"
)

type RootController struct {
	Version string
}

func (controller RootController) Show(ctx *gin.Context) {
	views.RenderTOON(ctx, http.StatusOK, models.RootResponse{
		Message: "Stock Prices API",
		Version: controller.Version,
	})
}
