TARGET = vterm

ASSETS_DIR = assets

all: help

help: ## Print this help message
	@grep -E '^[a-zA-Z._-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

copy_assets: ## Copy images to the home directory
	@./scripts/copy_images.sh

clean: ## Remove all build artifacts
	@cargo clean

build: ## Build optimized binary for current platform
	@platform=$$(uname -s | tr '[:upper:]' '[:lower:]'); \
	if [ "$$platform" != "linux" ]; then \
		echo "Unsupported platform: $$platform. Only Linux is supported."; \
		exit 1; \
	fi; \
	command="cargo build --release --"; \
	$$command

.PHONY: build clean help run copy_assets
