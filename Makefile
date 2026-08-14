PROJECT_NAME=mkk_basis
GIT_COMMIT=$(shell git rev-parse --short HEAD)
POSTGRES_DSN=postgres://postgres:postgres@127.0.0.1:5432/postgres?options=-c%20search_path%3Dmkk_basis
PRIVATE_KEY_FILEPATH ?= ./data/private.key # если переменная не определена, то присвоить значение
CONFIGS_PATH=./data

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
	@if [ ! -f $(PRIVATE_KEY_FILEPATH) ]; then \
		echo "Generating private key..."; \
		openssl genrsa -out $(PRIVATE_KEY) 2048; \
		chmod 600 $(PRIVATE_KEY_FILEPATH); \
		echo "✅ private key created in $(PRIVATE_KEY_FILEPATH)"; \
	else \
		echo "⚠️ private key already exists"; \
	fi

.PHONY: gen_tls_certs
gen_tls_certs:
	$(eval DAYS=365)
	$(eval SUBJECT_DESC=/C=RU/ST=Moscow/L=Moscow/O=WB/OU=Cloud)

	mkdir -p $(CONFIGS_PATH) # not deleted

	# generate a self-signed rootCA
	echo "subjectAltName=IP:127.0.0.1" > $(CONFIGS_PATH)/server_cert_ext.cnf
	openssl req -newkey rsa:2048 -new -nodes -x509 -days $(DAYS) -out $(CONFIGS_PATH)/ca.crt -keyout $(CONFIGS_PATH)/ca.key -subj "$(SUBJECT_DESC)/CN=MyCA"

	for service in "http"; do \
		for opt in "Server" "Client"; do \
			openssl genrsa -out $(CONFIGS_PATH)/$$service$$opt.key 2048; \
			openssl req -new -key $(CONFIGS_PATH)/$$service$$opt.key -days $(DAYS) -out $(CONFIGS_PATH)/$$service$$opt.csr -subj "$(SUBJECT_DESC)/CN=My_$$service$$opt"; \
			openssl x509 -req -in $(CONFIGS_PATH)/$$service$$opt.csr -extfile $(CONFIGS_PATH)/server_cert_ext.cnf -CA $(CONFIGS_PATH)/ca.crt -CAkey $(CONFIGS_PATH)/ca.key -days $(DAYS) -sha256 -CAcreateserial -out $(CONFIGS_PATH)/$$service$$opt.crt; \
		done; \
	done

.PHONY: check_version_certs
check_version_certs:
	openssl x509 -in ./data/server.crt -text -noout | grep "Version"

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