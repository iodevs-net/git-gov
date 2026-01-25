.PHONY: build test demo daemon cli help

# Colores para la terminal
BLUE=\033[0;34m
NC=\033[0m

help:
	@echo "$(BLUE)Cliff-Watch - Automatización$(NC)"
	@echo "Comandos disponibles:"
	@echo "  make build    - Compila todo el proyecto"
	@echo "  make test     - Corre todas las pruebas unitarias y de integración"
	@echo "  make demo     - Corre la suite de validación científica (Turing Test)"
	@echo "  make daemon   - Inicia el daemon de cliff-watch"
	@echo "  make cli      - Muestra la ayuda de la CLI"

build:
	cargo build

test:
	cargo test --workspace

demo:
	cargo run --example scientific_validation -p cliff-watch-core

daemon:
	cargo run -p cliff-watch-daemon

cli:
	cargo run -p cliff-watch-cli -- --help

install:
	./install.sh

uninstall:
	sudo rm -f /usr/local/bin/cliff-watch
	sudo rm -f /usr/local/bin/cliff-watch-daemon
	@echo "Cliff-Watch desinstalado correctamente de /usr/local/bin"
