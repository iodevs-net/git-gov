#!/bin/bash

# Cliff-Craft Shadow Installer 🏛️⚙️
# Formal deployment script for Linux systems

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}Cliff-Craft: Iniciando instalación formal...${NC}"

# 1. Verificar dependencias
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo no detectado. Instálalo primero con rustup.rs${NC}"
    exit 1
fi

# 2. Compilar en modo Release
echo -e "${BLUE}Compilando binarios de alto rendimiento (Release)...${NC}"
cargo build --release

# 3. Instalación de binarios
echo -e "${BLUE}Desplegando binarios a /usr/local/bin (requiere sudo)...${NC}"
sudo cp target/release/cliff-craft-cli /usr/local/bin/cliff-craft
sudo cp target/release/cliff-craft-daemon /usr/local/bin/cliff-craft-daemon

# 4. Configuración de permisos
echo -e "${BLUE}Configurando permisos de hardware para el usuario actual...${NC}"
sudo usermod -a -G input $USER || true

# 5. Finalización
echo -e "--------------------------------------------------------"
echo -e "${GREEN}✅ Cliff-Craft instalado con éxito.${NC}"
echo -e "Comandos disponibles:"
echo -e "  - ${BLUE}cliff-craft-daemon${NC}: Inicia el centinela."
echo -e "  - ${BLUE}cliff-craft init${NC}: Activa la aduana en un repositorio."
echo -e ""
echo -e "${BLUE}IMPORTANTE:${NC} Para que los cambios de grupo tengan efecto,"
echo -e "debes cerrar sesión e iniciarla de nuevo."
echo -e "--------------------------------------------------------"
