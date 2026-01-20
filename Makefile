.PHONY: build test demo daemon cli help

# Colores para la terminal
BLUE=\033[0;34m
NC=\033[0m

help:
	@echo "$(BLUE)Git-Gov - Automatización$(NC)"
	@echo "Comandos disponibles:"
	@echo "  make build    - Compila todo el proyecto"
	@echo "  make test     - Corre todas las pruebas unitarias y de integración"
	@echo "  make demo     - Corre la suite de validación científica (Turing Test)"
	@echo "  make daemon   - Inicia el daemon de git-gov"
	@echo "  make cli      - Muestra la ayuda de la CLI"

build:
	cargo build

test:
	cargo test --workspace

demo:
	cargo run --example scientific_validation -p git-gov-core

daemon:
	cargo run -p git-gov-daemon

cli:
	cargo run -p git-gov-cli -- --help
