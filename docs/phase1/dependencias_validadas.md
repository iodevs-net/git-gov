# Dependencias Validadas para cliff-watch

## 1. Introducción
Este documento detalla las dependencias críticas del proyecto cliff-watch, validando su compatibilidad, seguridad y versión específica para garantizar la estabilidad del sistema.

## 2. Dependencias Principales

### 2.1 Lenguaje y Herramientas
- **Rust**: Versión 1.70 o superior (recomendado 1.75 para mejor soporte de características modernas).
- **Cargo**: Gestor de paquetes y sistema de construcción de Rust.

### 2.2 Librerías de Rust

#### 2.2.1 Interfaz de Línea de Comandos (CLI)
- **clap**: Versión 4.5.4
  - **Descripción**: Librería para el manejo de argumentos de línea de comandos.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Implementación de comandos como `cliff-watch init`, `cliff-watch verify`, y `cliff-watch daemon`.

#### 2.2.2 Interacción con Git
- **git2**: Versión 0.20.0
  - **Descripción**: Bindings de Rust para libgit2, una implementación de Git en C.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades críticas.
  - **Configuración**: Se utilizará con `vendored-libgit2` para compilación estática y portabilidad.
  - **Uso**: Interacción con repositorios Git, cálculo de hashes de árboles, e inyección de trailers.

#### 2.2.3 Monitorización del Sistema de Archivos
- **notify**: Versión 8.2.0
  - **Descripción**: Librería para monitorización de eventos del sistema de archivos.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Configuración**: Se utilizará con `macos_fsevent` para soporte mejorado en macOS.
  - **Uso**: Captura de eventos de edición en tiempo real para el cálculo de métricas.

#### 2.2.4 Compresión y Entropía
- **zstd**: Versión 0.13.0
  - **Descripción**: Bindings de Rust para Zstandard, un algoritmo de compresión de alta eficiencia.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Cálculo de la Normalized Compression Distance (NCD) para evaluar la complejidad algorítmica del código.

#### 2.2.5 Análisis Estadístico
- **statrs**: Versión 0.18.0
  - **Descripción**: Librería para análisis estadístico en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Configuración**: Se utilizará con `default-features = false` para evitar dependencias innecesarias como `nalgebra`.
  - **Uso**: Cálculo de métricas como Burstiness, media, y desviación estándar.

#### 2.2.6 Criptografía
- **ed25519-dalek**: Versión 2.1.1
  - **Descripción**: Implementación de la firma digital Ed25519 en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Configuración**: Se utilizará con `fast` para optimización de rendimiento.
  - **Uso**: Firma criptográfica de metadatos de procedencia.

- **sha2**: Versión 0.10.8
  - **Descripción**: Implementación de funciones hash SHA-2 en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Generación de hashes para el Proof of Work (PoW) y verificación de integridad.

#### 2.2.7 Serialización de Datos
- **serde**: Versión 1.0.197
  - **Descripción**: Framework para serialización y deserialización de datos en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Configuración**: Se utilizará con `derive` para soporte de macros.
  - **Uso**: Serialización de metadatos de procedencia en formato JSON.

- **serde_json**: Versión 1.0.114
  - **Descripción**: Implementación de serialización JSON para serde.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Generación y parsing de manifiestos de procedencia en formato JSON.

#### 2.2.8 Utilidades
- **anyhow**: Versión 1.0.81
  - **Descripción**: Librería para manejo de errores flexible en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Manejo de errores en la CLI y el daemon.

- **thiserror**: Versión 2.0.0
  - **Descripción**: Librería para definición de tipos de error personalizados en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Definición de errores específicos del dominio en `cliff-watch-core`.

- **chrono**: Versión 0.4.35
  - **Descripción**: Librería para manejo de fechas y horas en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Configuración**: Se utilizará con `serde` para soporte de serialización.
  - **Uso**: Registro de marcas de tiempo en metadatos de procedencia.

- **dirs**: Versión 5.0.1
  - **Descripción**: Librería para acceso a directorios estándar del sistema.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Localización de directorios de configuración y almacenamiento.

- **rand**: Versión 0.8.5
  - **Descripción**: Librería para generación de números aleatorios en Rust.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Generación de claves criptográficas.

## 3. Dependencias para el Daemon

### 3.1 Manejo de Señales
- **ctrlc**: Versión 3.4.4
  - **Descripción**: Librería para manejo de señales de interrupción (Ctrl+C, SIGTERM).
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Cierre elegante del daemon ante señales de interrupción.

## 4. Dependencias para el Mouse Sentinel

### 4.1 Captura de Eventos de Entrada
- **rdev**: Versión 0.5.5
  - **Descripción**: Librería para captura global de eventos de entrada (teclado y ratón).
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Captura de eventos de movimiento del ratón en Windows y macOS.

- **evdev**: Versión 0.12.1
  - **Descripción**: Bindings de Rust para el subsistema de entrada del kernel de Linux.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Captura de eventos de movimiento del ratón en Linux/Wayland.

- **nix**: Versión 0.26.4
  - **Descripción**: Bindings de Rust para funciones del sistema operativo Unix.
  - **Validación**: Compatible con Rust 1.70+, sin vulnerabilidades conocidas.
  - **Uso**: Manejo de descriptores de archivo y permisos en Linux.

## 5. Validación de Seguridad

### 5.1 Análisis de Vulnerabilidades
Todas las dependencias han sido validadas utilizando las siguientes herramientas:
- **cargo-audit**: Para detección de vulnerabilidades conocidas en dependencias.
- **cargo-deny**: Para análisis de licencias y vulnerabilidades.
- **dependabot**: Para monitoreo continuo de actualizaciones de seguridad.

### 5.2 Resultados
- **Vulnerabilidades Críticas**: Ninguna detectada en las versiones seleccionadas.
- **Vulnerabilidades Moderadas**: Ninguna detectada en las versiones seleccionadas.
- **Licencias**: Todas las dependencias utilizan licencias compatibles (MIT, Apache 2.0).

## 6. Configuración del Workspace

### 6.1 Estructura del Proyecto
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
```

### 6.2 Perfiles de Compilación
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## 7. Conclusión
Este documento valida las dependencias críticas para el proyecto cliff-watch, asegurando que todas las librerías sean compatibles, seguras y estén optimizadas para el rendimiento. La configuración del workspace garantiza la coherencia y la eficiencia en la gestión de dependencias, adheriéndose a los principios DRY, LEAN y SOLID.