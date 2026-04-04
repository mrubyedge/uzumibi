.PHONY: help run docker-build docker-run build-image deploy

APP_NAME ?= $$PROJECT_NAME_KEBAB$$
PROJECT_ID ?= $(shell gcloud config get-value project 2>/dev/null)
REGION ?= asia-northeast1
PLATFORM ?= managed
SERVICE ?= $(APP_NAME)
IMAGE ?= gcr.io/$(PROJECT_ID)/$(APP_NAME)
ALLOW_UNAUTH ?= --allow-unauthenticated
DEPLOY_FLAGS ?=

help:
	@echo "Targets:"
	@echo "  make run           - Run locally with cargo"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-run    - Run Docker image locally"
	@echo "  make build-image   - Build and push image via Cloud Build"
	@echo "  make deploy        - Deploy to Cloud Run"

run:
	cargo run

docker-build:
	docker build -t $(APP_NAME) .

docker-run:
	docker run -p 8080:8080 $(APP_NAME)

build-image:
	@if [ -z "$(PROJECT_ID)" ]; then echo "PROJECT_ID is empty. Set with 'export PROJECT_ID=...'."; exit 1; fi
	gcloud builds submit --tag $(IMAGE)

deploy: build-image
	gcloud run deploy $(SERVICE) \
		--image $(IMAGE) \
		--platform $(PLATFORM) \
		--region $(REGION) \
		$(ALLOW_UNAUTH) \
		$(DEPLOY_FLAGS)
