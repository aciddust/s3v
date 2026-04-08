.PHONY: dev build check clean minio minio-down minio-logs bundle help bump patch minor major release deploy deploy-dry

# ── Development ──────────────────────────────────────

dev: ## Start Tauri dev (frontend + backend hot-reload)
	npm run tauri dev

check: ## Type-check Svelte + Rust
	npx svelte-check --threshold error
	cd src-tauri && cargo check

fmt: ## Format code
	./node_modules/.bin/prettier --write 'src/**/*.{svelte,ts,css}'
	cd src-tauri && cargo fmt

# ── Build ────────────────────────────────────────────

build: ## Build frontend only
	npm run build

bundle: ## Build distributable app (dmg/msi/deb)
	npm run tauri build

bundle-debug: ## Build debug bundle (faster, not optimized)
	npm run tauri build -- --debug

# ── Test infrastructure ──────────────────────────────

minio: ## Start MinIO (S3-compatible local server)
	docker compose up -d
	@echo ""
	@echo "  S3 API:       http://localhost:9000"
	@echo "  Web Console:  http://localhost:9001"
	@echo "  Credentials:  minioadmin / minioadmin"
	@echo ""
	@echo "  Buckets: test-small (10), test-large (2500), test-nested (hierarchy)"
	@echo ""

minio-down: ## Stop MinIO
	docker compose down

minio-clean: ## Stop MinIO and delete all data
	docker compose down -v

minio-logs: ## Tail MinIO logs
	docker compose logs -f minio

# ── Assets ───────────────────────────────────────────

logo: ## Generate app icons from static/s3v-logo.png
	npm run tauri icon static/s3v-logo.png

# ── Version & Deploy ─────────────────────────────────

bump: ## Bump version: make bump patch|minor|major (or make bump v=1.2.3)
	@current=$$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//'); \
	if [ -n "$(v)" ]; then \
		next="$(v)"; \
	elif [ "$(filter patch minor major,$(MAKECMDGOALS))" ]; then \
		part="$(filter patch minor major,$(MAKECMDGOALS))"; \
		IFS='.' read -r ma mi pa <<< "$$current"; \
		case $$part in \
			major) next="$$((ma+1)).0.0" ;; \
			minor) next="$$ma.$$((mi+1)).0" ;; \
			patch) next="$$ma.$$mi.$$((pa+1))" ;; \
		esac; \
	else \
		echo "Usage: make bump patch|minor|major  or  make bump v=X.Y.Z"; exit 1; \
	fi; \
	echo "$$current → $$next"; \
	sed -i '' "s/\"version\": \".*\"/\"version\": \"$$next\"/" src-tauri/tauri.conf.json; \
	sed -i '' "s/\"version\": \".*\"/\"version\": \"$$next\"/" package.json; \
	sed -i '' "s/^version = \".*\"/version = \"$$next\"/" src-tauri/Cargo.toml; \
	sed -i '' "s/APP_VERSION = '.*'/APP_VERSION = '$$next'/" src/lib/constants.ts; \
	echo "→ v$$next"

patch minor major: bump
	@true

release: ## Bump, verify, commit, tag, push: make release patch|minor|major
	@$(MAKE) bump $(filter patch minor major,$(MAKECMDGOALS)) $(if $(v),v=$(v))
	@version=$$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//'); \
	echo ""; \
	echo "Verifying v$$version across files..."; \
	ok=true; \
	for f in src-tauri/tauri.conf.json package.json; do \
		v=$$(grep '"version"' $$f | head -1 | sed 's/.*: "//;s/".*//'); \
		if [ "$$v" = "$$version" ]; then \
			echo "  ✓ $$f → $$v"; \
		else \
			echo "  ✗ $$f → $$v (expected $$version)"; ok=false; \
		fi; \
	done; \
	v=$$(grep '^version = ' src-tauri/Cargo.toml | head -1 | sed 's/.*= "//;s/".*//'); \
	if [ "$$v" = "$$version" ]; then \
		echo "  ✓ src-tauri/Cargo.toml → $$v"; \
	else \
		echo "  ✗ src-tauri/Cargo.toml → $$v (expected $$version)"; ok=false; \
	fi; \
	v=$$(grep "APP_VERSION" src/lib/constants.ts | sed "s/.*= '//;s/'.*//"); \
	if [ "$$v" = "$$version" ]; then \
		echo "  ✓ src/lib/constants.ts → $$v"; \
	else \
		echo "  ✗ src/lib/constants.ts → $$v (expected $$version)"; ok=false; \
	fi; \
	if [ "$$ok" = "false" ]; then echo "\nVersion mismatch. Aborting."; exit 1; fi; \
	echo ""; \
	echo "Committing & tagging v$$version..."; \
	git add -A; \
	git commit -m "chore: bump v$$version"; \
	git tag -a "v$$version" -m "Release v$$version"; \
	git push origin main "v$$version"; \
	echo ""; \
	echo "✓ Released v$$version — CI build triggered"

deploy: ## Tag current version and push to trigger release build (no bump)
	@version=$$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//' ); \
	echo "Releasing v$$version"; \
	git tag -a "v$$version" -m "Release v$$version"; \
	git push origin "v$$version"

deploy-dry: ## Show what version would be released (no action)
	@version=$$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "//;s/".*//' ); \
	echo "Would release: v$$version"

# ── Cleanup ──────────────────────────────────────────

clean: ## Remove build artifacts
	rm -rf build .svelte-kit
	cd src-tauri && cargo clean

# ── Install ──────────────────────────────────────────

install: ## Install all dependencies
	npm install
	cd src-tauri && cargo fetch

# ── Help ─────────────────────────────────────────────

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
