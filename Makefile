PROJECT_NAME=mkk_basis
GIT_COMMIT=$(shell git rev-parse --short HEAD)
POSTGRES_DSN=postgres://postgres:postgres@127.0.0.1:5432/postgres?search_path=mkk_basis&sslmode=disable

.PHONY: echo_version
echo_version:
	@echo commit:$(GIT_COMMIT)

install_deps:
	cargo install sqlx-cli

.PHONY: build
build:
	cargo build --release

.PHONY: run_server
run_server: build
	./target/release/$(PROJECT_NAME)

.PHONY: check
check:
	cargo check

.PHONY: update
update:
	cargo update

migration_up:
	sqlx migrate run --database-url "${POSTGRES_DSN}"

migration_down:
	sqlx migrate revert --database-url "${POSTGRES_DSN}"