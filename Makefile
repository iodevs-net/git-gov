.PHONY: build test demo daemon cli help

# Colores para la terminal
BLUE=\033[0;34m
NC=\033[0m

help:
	@echo "$(BLUE)Cliff-Craft - Automatización$(NC)"
	@echo "Comandos disponibles:"
	@echo "  make build    - Compila todo el proyecto"
	@echo "  make test     - Corre todas las pruebas unitarias y de integración"
	@echo "  make demo     - Corre la suite de validación científica (Turing Test)"
	@echo "  make daemon   - Inicia el daemon de cliff-craft"
	@echo "  make cli      - Muestra la ayuda de la CLI"

build:
	cargo build

test:
	cargo test --workspace

demo:
	cargo run --example scientific_validation -p cliff-craft-core

daemon:
	cargo run -p cliff-craft-daemon

cli:
	cargo run -p cliff-craft-cli -- --help

install:
	./install.sh

uninstall:
	sudo rm -f /usr/local/bin/cliff-craft
	sudo rm -f /usr/local/bin/cliff-craft-daemon
	@echo "Cliff-Craft desinstalado correctamente de /usr/local/bin"
