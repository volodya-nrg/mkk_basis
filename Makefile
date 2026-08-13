PROJECT_NAME=mkk_basis
GIT_COMMIT=$(shell git rev-parse --short HEAD)
POSTGRES_DSN=postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable
PRIVATE_KEY ?= private.key # если переменная не определена, то присвоить значение

.PHONY: echo_version
echo_version:
	@echo commit:$(GIT_COMMIT)

.PHONY: install_deps
install_deps:
	cargo install sqlx-cli

.PHONY: build
build:
	cargo build --release

.PHONY: run_server
run_server: build
	./target/release/$(PROJECT_NAME)

.PHONY: migration_up
migration_up:
	sqlx migrate run --database-url "${POSTGRES_DSN}"

.PHONY: migration_down
migration_down:
	sqlx migrate revert --database-url "${POSTGRES_DSN}"

.PHONY: generate-private-key
generate-private-key:
	@if [ ! -f $(PRIVATE_KEY) ]; then \
		echo "Generating private key..."; \
		openssl genrsa -out $(PRIVATE_KEY) 2048; \
		chmod 600 $(PRIVATE_KEY); \
		echo "✅ private key created in $(PRIVATE_KEY)"; \
	else \
		echo "⚠️ private key already exists"; \
	fi

.PHONY: test_db
test_db:
	cargo test --test db -- --nocapture # --include-ignored

.PHONY: test_transport
test_transport:
	RUST_BACKTRACE=1 cargo test --test transport -- --nocapture # --include-ignored

.PHONY: test_units
test_units:
	cargo test adapter::jwt

.PHONY: cargo_reload
cargo_reload:
	cargo clean && cargo update && cargo build

.PHONY: cargo_check
cargo_check:
	cargo check

.PHONY: lint
lint:
	cargo clippy # --tests