#!/bin/bash
set -e

# Setup paths
ROOT_DIR=$(pwd)
BIN="$ROOT_DIR/target/release/git-gov-cli"
DAEMON_BIN="$ROOT_DIR/target/release/git-gov-daemon"

# Clean previous run
rm -rf demo_repo
killall git-gov-daemon 2>/dev/null || true

echo "🟢 [1/6] Preparando Entorno..."
mkdir -p demo_repo
cd demo_repo
# Hack: Linkear binarios
mkdir -p bin
ln -s "$BIN" bin/git-gov
export PATH="$(pwd)/bin:$PATH"

# Configurar PRIMERO para que el daemon lo lea
$BIN config init
sed -i 's/min_entropy = 2.5/min_entropy = 0.0001/' git-gov.toml
echo "Configuración ajustada:"
grep "min_entropy" git-gov.toml

echo "🟢 [2/6] Iniciando Git-Gov Daemon..."
# Iniciar daemon DENTRO del repo para que lea git-gov.toml
$DAEMON_BIN &
DAEMON_PID=$!
sleep 2

echo "🟢 [3/6] Inicializando Git..."
git init -b main
git config user.name "AI Demo"
git config user.email "ai@example.com"
$BIN init

echo "🟢 [5/6] Creando Código (Entropy Injection)..."
echo "fn main() { println!(\"Hello Governance!\"); }" > main.rs
git add main.rs

echo "🟢 [6/6] Realizando Commit (The Magic Moment)..."
# Esto disparará el hook 'pre-commit' -> 'git-gov verify-work' -> Daemon
# Si funciona, el commit se firmará.
if git commit -m "feat: my first governed commit"; then
    echo "✅ COMMIT EXITOSO!"
else
    echo "❌ COMMIT FALLIDO (Customs Rejected)"
    kill $DAEMON_PID
    exit 1
fi

echo "🔍 Verificando Firma..."
# Verificamos que el trailer se añadió
git log -1

echo "🧹 Limpiando..."
kill $DAEMON_PID
cd ..
rm -rf demo_repo
echo "✨ DEMO COMPLETA ✨"
