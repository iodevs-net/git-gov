#!/bin/bash
# cliff-craft Seed Bootstrap v2.1
set -e
echo "🌱 Sembrando el Centinela (cliff-craft)..."

# 1. Asegurar Rust
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 2. Delegar al Orquestador Nativo
echo "🚀 Lanzando Orquestador Soberano (Rust)..."
cargo run --quiet -- setup --yes
