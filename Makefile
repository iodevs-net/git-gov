.PHONY: build test demo daemon cli help

# Colores para la terminal
BLUE=\033[0;34m
GREEN=\033[0;32m
RED=\033[0;31m
NC=\033[0m

help:
	@echo "$(BLUE)Cliff-Watch - Sovereign Orchestrator$(NC)"
	@echo "Available targets:"
	@echo "  make bootstrap - Install Rust (if missing) and setup project"
	@echo "  make build     - Compile all crates"
	@echo "  make test      - Run all unit and integration tests"
	@echo "  make demo      - Run scientific validation suite"
	@echo "  make daemon    - Start cliff-watch-daemon"
	@echo "  make install   - Compile in release and install to /usr/local/bin (sudo)"
	@echo "  make clean     - Purgue build artifacts and local database"

bootstrap:
	@echo "$(BLUE)🌱 Seeding Cliff-Watch...$(NC)"
	@if ! command -v rustup > /dev/null; then \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
	fi
	cargo run --quiet -- setup --yes

build:
	cargo build

test:
	cargo test --workspace

demo:
	cargo run --example scientific_validation -p cliff-watch-core

daemon:
	cargo run -p cliff-watch-daemon

install:
	@echo "$(BLUE)Compiling high-performance binaries (Release)...$(NC)"
	cargo build --release
	@echo "$(BLUE)Deploying to /usr/local/bin (requires sudo)...$(NC)"
	sudo cp target/release/cliff-watch-cli /usr/local/bin/cliff-watch
	sudo cp target/release/cliff-watch-daemon /usr/local/bin/cliff-watch-daemon
	@sudo usermod -a -G input $(USER) || true
	@echo "$(GREEN)✅ Cliff-Watch installed successfully.$(NC)"

uninstall:
	sudo rm -f /usr/local/bin/cliff-watch
	sudo rm -f /usr/local/bin/cliff-watch-daemon
	@echo "Cliff-Watch uninstalled from /usr/local/bin"

clean:
	@echo "$(RED)Purging artifacts...$(NC)"
	cargo clean
	rm -f db.sqlite3
