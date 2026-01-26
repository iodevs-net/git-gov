.PHONY: build test demo daemon cli help watch stop restart status logs metrics ext-install

# Colores para la terminal
BLUE=\033[0;34m
GREEN=\033[0;32m
RED=\033[0;31m
YELLOW=\033[1;33m
NC=\033[0m

help:
	@echo "$(BLUE)Cliff-Watch - Sovereign Orchestrator$(NC)"
	@echo "Available targets:"
	@echo "  $(GREEN)make watch$(NC)      - Start daemon in background (nohup)"
	@echo "  $(GREEN)make stop$(NC)       - Stop the background daemon"
	@echo "  $(GREEN)make restart$(NC)    - Restart daemon"
	@echo "  $(GREEN)make metrics$(NC)    - Show current battery & focus stats"
	@echo "  $(GREEN)make logs$(NC)       - Show last 20 lines of daemon log"
	@echo "  make status      - Check if daemon is running"
	@echo "  make build       - Compile all crates"
	@echo "  make test        - Run all tests"
	@echo "  make ext-install - Manually install VS Code extension"

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

# Inicia el daemon en primer plano (para debug)
daemon:
	cargo run -p cliff-watch-daemon

# Comandos de Gestión del Daemon (Background)
watch:
	@if pgrep -f "cliff-watch-daemon" > /dev/null; then \
		echo "$(YELLOW)Daemon is already running.$(NC)"; \
	else \
		echo "$(BLUE)Starting Sentinel in background...$(NC)"; \
		nohup ./target/debug/cliff-watch-daemon > nohup.out 2>&1 & \
		sleep 1; \
		echo "$(GREEN)Sentinel is watching.$(NC)"; \
	fi

stop:
	@echo "$(RED)Stopping Sentinel...$(NC)"
	@pkill -f "cliff-watch-daemon" || echo "Sentinel was not running."

restart: stop watch

status:
	@./target/debug/cliff-watch-cli status

metrics:
	@./target/debug/cliff-watch-cli metrics

logs:
	@tail -n 20 -f nohup.out

# Utilidades de Desarrollo
ext-install:
	@echo "$(BLUE)Re-installing VS Code Extension...$(NC)"
	@ls -t *.vsix | head -n 1 | xargs code --install-extension --force
	@echo "$(GREEN)Extension installed. Please reload VS Code.$(NC)"

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
	rm -f db.sqlite3 nohup.out
