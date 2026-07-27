PROJECT_NAME=mkk_basis
GIT_COMMIT=$(shell git rev-parse --short HEAD)

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

#migration_run:
#	sqlx migrate run --database-url "sqlite:./data/db.sqlite3"
