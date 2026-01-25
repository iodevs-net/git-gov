# Estructura del Proyecto cliff-watch

## 1. Introducción
Este documento describe la estructura del proyecto cliff-watch, incluyendo la organización de directorios, archivos y módulos, asegurando la adherencia a los principios DRY, LEAN y SOLID.

## 2. Estructura de Directorios

```
cliff-watch/
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock                  # Locked dependencies
├── Makefile                    # Automation for cross-compilation and installation
├── README.md                   # Project documentation
├── docs/
│   ├── phase1/
│   │   ├── requisitos_resumidos.md
│   │   ├── dependencias_validadas.md
│   │   └── plan_mitigacion_riesgos.md
│   ├── phase2/
│   │   ├── arquitectura_sistema.md
│   │   ├── definicion_modulos.md
│   │   └── diseno_mouse_sentinel.md
│   ├── principios/
│   │   └── dry_lean_solid.md
│   └── estructura_proyecto.md
├── crates/
│   ├── cliff-watch-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── crypto.rs
│   │       ├── entropy.rs
│   │       ├── git.rs
│   │       ├── monitor.rs
│   │       ├── provenance.rs
│   │       └── stats.rs
│   ├── cliff-watch-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   └── cliff-watch-daemon/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── ipc.rs
└── scripts/
    └── install_hooks.sh
```

## 3. Descripción de Archivos y Directorios

### 3.1 Archivos Raíz

#### 3.1.1 Cargo.toml
- **Descripción**: Archivo de configuración del workspace de Cargo.
- **Contenido**: Definición de miembros del workspace, versiones de dependencias y perfiles de compilación.

#### 3.1.2 Cargo.lock
- **Descripción**: Archivo de bloqueo de dependencias.
- **Contenido**: Versiones exactas de todas las dependencias del proyecto.

#### 3.1.3 Makefile
- **Descripción**: Archivo de automatización para compilación e instalación.
- **Contenido**: Comandos para compilación cruzada, instalación de hooks y pruebas.

#### 3.1.4 README.md
- **Descripción**: Documentación principal del proyecto.
- **Contenido**: Descripción del proyecto, instrucciones de instalación y uso.

### 3.2 Documentación

#### 3.2.1 docs/phase1/
- **Descripción**: Documentación de la Fase 1 (Investigación y Análisis).
- **Contenido**:
  - Requisitos resumidos.
  - Dependencias validadas.
  - Plan de mitigación de riesgos.

#### 3.2.2 docs/phase2/
- **Descripción**: Documentación de la Fase 2 (Diseño y Arquitectura).
- **Contenido**:
  - Arquitectura del sistema.
  - Definición de módulos.
  - Diseño del Mouse Sentinel.

#### 3.2.3 docs/principios/
- **Descripción**: Documentación de principios de diseño.
- **Contenido**:
  - Aplicación de principios DRY, LEAN y SOLID.

#### 3.2.4 docs/estructura_proyecto.md
- **Descripción**: Este documento.
- **Contenido**: Estructura del proyecto y descripción de archivos.

### 3.3 Módulos de Rust

#### 3.3.1 crates/cliff-watch-core/
- **Descripción**: Lógica central del sistema.
- **Contenido**:
  - Módulos para criptografía, entropía, Git, monitoreo, procedencia y estadísticas.

#### 3.3.2 crates/cliff-watch-cli/
- **Descripción**: Interfaz de línea de comandos.
- **Contenido**:
  - Punto de entrada de la CLI y subcomandos.

#### 3.3.3 crates/cliff-watch-daemon/
- **Descripción**: Proceso en segundo plano.
- **Contenido**:
  - Punto de entrada del daemon y comunicación inter-procesos.

### 3.4 Scripts

#### 3.4.1 scripts/install_hooks.sh
- **Descripción**: Script para instalación de hooks de Git.
- **Contenido**: Comandos para configurar hooks en repositorios Git.

## 4. Configuración del Workspace

### 4.1 Cargo.toml

```toml
[workspace]
members = [
    "crates/cliff-watch-core",
    "crates/cliff-watch-cli",
    "crates/cliff-watch-daemon",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["cliff-watch team"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/cliff-watch/cliff-watch"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive", "string", "env"] }
git2 = { version = "0.20", default-features = false, features = ["vendored-libgit2"] }
notify = { version = "8.2", default-features = false, features = ["macos_fsevent"] }
zstd = { version = "0.13", default-features = false }
statrs = { version = "0.18", default-features = false }
ed25519-dalek = { version = "2.1", features = ["fast", "rand_core"] }
sha2 = "0.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
rand = "0.8"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 4.2 Perfiles de Compilación

#### 4.2.1 Perfil de Release
- **opt-level = "z"**: Optimización para tamaño de binario.
- **lto = true**: Optimización en tiempo de enlace.
- **codegen-units = 1**: Reduce el paralelismo para mejor optimización global.
- **panic = "abort"**: Elimina el manejo de stack unwinding.
- **strip = true**: Elimina símbolos de depuración.

## 5. Módulos de Rust

### 5.1 cliff-watch-core

#### 5.1.1 Cargo.toml
```toml
[package]
name = "cliff-watch-core"
version.workspace = true
edition.workspace = true
description = "Core logic for Decentralized Code Governance metrics and crypto."

[dependencies]
git2 = { workspace = true }
notify = { workspace = true }
zstd = { workspace = true }
statrs = { workspace = true }
ed25519-dalek = { workspace = true }
sha2 = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
rand = "0.8"
```

#### 5.1.2 src/lib.rs
```rust
pub mod crypto;
pub mod entropy;
pub mod git;
pub mod monitor;
pub mod provenance;
pub mod stats;
```

### 5.2 cliff-watch-cli

#### 5.2.1 Cargo.toml
```toml
[package]
name = "cliff-watch"
version.workspace = true
edition.workspace = true
default-run = "cliff-watch"

[dependencies]
cliff-watch-core = { workspace = true }
clap = { workspace = true }
anyhow = { workspace = true }
```

#### 5.2.2 src/main.rs
```rust
use clap::{Parser, Subcommand};
use cliff_watch_core::sentinel_self_check;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "cliff-watch")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    SystemCheck,
    Daemon,
    Verify { commit: Option<String> },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::SystemCheck) => {
            match sentinel_self_check() {
                Ok(report) => {
                    println!("{}", report);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("CRITICAL FAILURE: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        _ => ExitCode::SUCCESS,
    }
}
```

### 5.3 cliff-watch-daemon

#### 5.3.1 Cargo.toml
```toml
[package]
name = "cliff-watch-daemon"
version.workspace = true
edition.workspace = true

[dependencies]
cliff-watch-core = { workspace = true }
anyhow = { workspace = true }
ctrlc = "3.4"
```

#### 5.3.2 src/main.rs
```rust
use cliff_watch_core::monitor::start_monitoring;
use std::process::ExitCode;

fn main() -> ExitCode {
    match start_monitoring() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Daemon failed: {}", e);
            ExitCode::FAILURE
        }
    }
}
```

## 6. Scripts

### 6.1 install_hooks.sh
```bash
#!/bin/bash

# Script para instalar hooks de Git
HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(dirname "$0")"

# Crear directorio de hooks si no existe
mkdir -p "$HOOKS_DIR"

# Instalar hook de prepare-commit-msg
cp "$SCRIPT_DIR/prepare-commit-msg" "$HOOKS_DIR/prepare-commit-msg"
chmod +x "$HOOKS_DIR/prepare-commit-msg"

echo "Hooks instalados correctamente."
```

## 7. Conclusión
Este documento describe la estructura del proyecto cliff-watch, asegurando que el diseño cumpla con los principios DRY, LEAN y SOLID. La estructura propuesta proporciona una base sólida para la implementación del sistema, garantizando la estabilidad, seguridad y eficiencia del mismo.