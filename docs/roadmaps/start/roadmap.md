# Roadmap: Implementación de cliff-craft

> **Estado**: Planificación Inicial
> **Objetivo**: Construir el protocolo de gobernanza descentralizada de código de forma segura, ordenada y eficiente.
> **Contexto**: La crisis de entropía en el desarrollo de software, causada por la proliferación de código generado por IA, requiere un sistema robusto para validar la autenticidad de las contribuciones humanas.

## Fase 0: Preparación del Entorno (Día 1)

### 0.1. Verificación del Sistema
- [ ] Verificar instalación de Rust (cargo/rustc)
- [ ] Verificar herramientas de compilación (gcc, make, etc.)
- [ ] Verificar Git instalado (versión >= 2.20)
- [ ] Configurar Rust para optimización de tamaño: `rustup target add wasm32-unknown-unknown` (opcional)

### 0.2. Estructura de Directorios Inicial
```
cliff-craft/
├── .gitignore
├── LICENSE
├── README.md
├── docs/
│   ├── research/
│   │   ├── rust-lean-dry-solid.md
│   │   └── sentinel-protocol.md
│   └── roadmap.md  ← (este archivo)
├── Cargo.toml
├── Cargo.lock
├── Makefile
└── crates/
    ├── cliff-craft-core/
    ├── cliff-craft-cli/
    └── cliff-craft-daemon/
```

### 0.3. Archivos de Configuración Base
- [ ] Crear `.gitignore` con patrones para Rust (`/target`, `/Cargo.lock`, etc.)
- [ ] Crear `LICENSE` (MIT)
- [ ] Actualizar `README.md` con estado actual y contribuciones

---

## Fase 1: Workspace y Configuración (Días 2-3)

### 1.1. Configuración del Workspace (Root Cargo.toml)
**Objetivo**: Definir la estructura de crates y centralizar dependencias.

- [ ] Crear `Cargo.toml` con:
  - Definición de `workspace.members`
  - `resolver = "2"`
  - `[workspace.dependencies]` con versiones exactas
  - Perfiles de optimización `[profile.release]`
  - Herramientas de desarrollo (cargo-watch, cargo-edit)

**Dependencias a definir**:

> **Nota de Privacidad**: Todas las dependencias deben ser auditadas para garantizar que no capturen contenido sensible ni realicen operaciones no autorizadas. Se priorizarán librerías con políticas claras de privacidad y sin telemetría oculta.
```toml
[workspace.dependencies]
# UI
clap = { version = "4.5", features = ["derive", "string", "env"] }

# Git
git2 = { version = "0.20", default-features = false, features = ["vendored-libgit2"] }

# Surveillance
notify = { version = "8.2", default-features = false, features = ["macos_fsevent"] }

# Entropy & Compression
zstd = { version = "0.13", default-features = false }

# Statistics
statrs = { version = "0.18", default-features = false }

# Cryptography
ed25519-dalek = { version = "2.1", features = ["fast", "rand_core"] }
sha2 = "0.10"

# Data
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error Handling
anyhow = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"

# Randomness
rand = "0.8"

# Internal
cliff-craft-core = { path = "crates/cliff-craft-core" }
```

**Perfiles de optimización**:
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 1.2. Creación de Crates
- [ ] Crear `crates/cliff-craft-core/` (libra)
- [ ] Crear `crates/cliff-craft-cli/` (binario)
- [ ] Crear `crates/cliff-craft-daemon/` (binario)

Para cada crate:
1. Crear `Cargo.toml` con `workspace = true` para heredar dependencias
2. Crear `src/` con archivos iniciales
3. Agregar `Cargo.toml` al workspace

### 1.3. Pruebas de Compilación y Tamaño
- [ ] Compilar en modo debug: `cargo build`
- [ ] Verificar estructura de workspace: `cargo metadata --format-version 1`
- [ ] Compilar en modo release: `cargo build --release`
- [ ] Verificar tamaño del binario: `ls -lh target/release/cliff-craft`
- [ ] Comparar con objetivo de tamaño (ideal < 5MB)

---

## Fase 2: cliff-craft-core - Fundamentos (Días 4-8)

### 2.1. Módulo de Manejo de Errores
**Archivo**: `crates/cliff-craft-core/src/error.rs`

- [ ] Definir enum `Error` con variantes:
  - GitError
  - CryptoError
  - StatError
  - IoError
  - EntropyError
- [ ] Implementar `std::error::Error` y `Display`
- [ ] Usar `thiserror` para derivaciones automáticas

### 2.2. Módulo de Criptografía (Pruebas Iniciales)
**Archivo**: `crates/cliff-craft-core/src/crypto.rs`

- [ ] Implementar `sentinel_self_check()` para verificar:
  - Generación de claves Ed25519
  - Firma y verificación
  - Hash SHA256
- [ ] Configurar `rand::rngs::OsRng` para seguridad criptográfica

### 2.3. Módulo de Estadísticas (Sin nalgebra)
**Archivo**: `crates/cliff-craft-core/src/stats.rs`

- [ ] Implementar cálculo de Burstiness:
  - `calculate_burstiness(data: &[f64]) -> f64`
  - Fórmula: `B = (σ - μ) / (σ + μ)`
- [ ] Usar statrs con `default-features = false`
- [ ] Pruebas unitarias con datos conocidos

### 2.4. Módulo de Compresión y Entropía
**Archivo**: `crates/cliff-craft-core/src/entropy.rs`

- [ ] Implementar cálculo de NCD (Normalized Compression Distance):
  - `calculate_ncd(x: &[u8], y: &[u8]) -> f64`
  - Usar zstd con streaming API
  - Nivel de compresión: 1 (rápido, "dumb compression")
- [ ] Manejo de archivos grandes: procesar en chunks (4KB)
- [ ] Pruebas con datos sintéticos (alta/low entropía)

### 2.5. Módulo de Git (Abstracciones)
**Archivo**: `crates/cliff-craft-core/src/git.rs`

- [ ] Funciones base con git2:
  - `get_repository(path: &str) -> Result<Repository>`
  - `get_commit_message(commit: &Commit) -> Result<String>`
  - `inject_trailer(commit_msg: &str, trailer: &str) -> Result<String>`
- [ ] Manejo de errores específicos de git2
- [ ] Pruebas en un repo temporal

### 2.6. Módulo de Provenance (Esquema JSON)
**Archivo**: `crates/cliff-craft-core/src/provenance.rs`

- [ ] Definir structs con serde:
  - `ProvenanceManifest`
  - `EntropyMetrics`
  - `ProofOfWork`
  - `Metadata`
- [ ] Validación de campos obligatorios
- [ ] Serialización/deserialización

### 2.7. Libra Principal (lib.rs)
**Archivo**: `crates/cliff-craft-core/src/lib.rs`

- [ ] Exportar módulos públicos
- [ ] Implementar `sentinel_self_check()` completa que combine:
  - Verificación git2
  - Verificación criptográfica
  - Verificación compresión
  - Verificación estadística
- [ ] Documentación básica de la API

### 2.8. Pruebas del Core
- [ ] Tests unitarios para cada módulo
- [ ] Integración: `cargo test --workspace`
- [ ] Verificar que el core compila sin CLI ni daemon

---

## Fase 3: cliff-craft-cli - Interfaz de Usuario (Días 9-11)

### 3.1. Estructura de Comandos
**Archivo**: `crates/cliff-craft-cli/src/main.rs`

- [ ] Definir struct `Cli` con `#[derive(Parser)]`
- [ ] Definir enum `Commands` con subcomandos:
  - `SystemCheck`: Verifica integridad del stack
  - `Init`: Inicializa hooks git
  - `Daemon`: Inicia el monitor
  - `Verify { commit }`: Verifica un commit
  - `Help`: Ayuda general

### 3.2. Implementación de Comandos
- [ ] `SystemCheck`: Llamar a `cliff_craft_core::sentinel_self_check()`
- [ ] `Init`: (placeholder) Lógica futura
- [ ] `Daemon`: (placeholder) Lógica futura
- [ ] `Verify { commit }`: (placeholder) Lógica futura
- [ ] Manejo de argumentos con clap
- [ ] Salida estructurada (println, eprintln)

### 3.3. Pruebas del CLI
- [ ] Compilar binario: `cargo build --package cliff-craft-cli`
- [ ] Ejecutar comandos:
  - `./target/release/cliff-craft system-check`
  - Verificar que muestra reporte de integridad
- [ ] Verificar ayuda: `./target/release/cliff-craft --help`
- [ ] Verificar tamaño del binario (debe ser pequeño)

### 3.4. Documentación
- [ ] Agregar ejemplos de uso en README
- [ ] Documentar comandos y opciones
- [ ] Agregar metadatos de CLI (autor, versión, licencia)

---

## Fase 4: cliff-craft-daemon - Monitoreo en Segundo Plano (Días 12-16)

### 4.1. Estructura del Daemon
**Archivo**: `crates/cliff-craft-daemon/src/main.rs`

- [ ] Entrada principal con manejo de señales (Ctrl+C, SIGTERM)
- [ ] Estructura del loop principal
- [ ] Manejo de errores (crash seguro)

### 4.2. Módulo de Surveillance (notify)
**Archivo**: `crates/cliff-craft-daemon/src/surveillance.rs` (o dentro de main.rs)

- [ ] Configuración de notify:
  - Monitorear directorios de trabajo (recursivo)
  - Filtrar eventos: creación, modificación, borrado
- [ ] Debounce personalizado (10 segundos):
  - Agregar eventos a buffer
  - Procesar cada 10s (epoch)
  - Limpiar buffer después de procesar
- [ ] Calcular métricas:
  - Inter-arrival times
  - Burstiness en tiempo real

> **Validación de Métricas**: Las métricas calculadas deben ser validadas para distinguir entre contribuciones humanas y generadas por IA. Se implementará un sistema de umbrales dinámicos basado en estudios previos de patrones de edición humana.

### 4.3. Módulo de IPC (Comunicación)
**Archivo**: `crates/cliff-craft-daemon/src/ipc.rs`

- [ ] Implementar comunicación con CLI:
  - Socket Unix (Linux/macOS)
  - Named Pipes (Windows)
- [ ] Protocolo de mensajes simple (JSON RPC?)
- [ ] Manejo de concurrencia (threads)

### 4.4. Integración con cliff-craft-core
- [ ] Usar módulos de estadística del core
- [ ] Generar telemetría estructurada
- [ ] Cache temporal de métricas

### 4.5. Pruebas del Daemon
- [ ] Ejecutar en segundo plano: `./target/release/cliff-craft-daemon`
- [ ] Probar monitoreo de archivos reales
- [ ] Verificar consumo de recursos (CPU, memoria)
- [ ] Pruebas de estrés (múltiples eventos)

---

## Fase 5: Proof of Human Work (PoHW) (Días 17-20)

### 5.1. Implementación del Puzzle de PoW
**Archivo**: `crates/cliff-craft-core/src/pow.rs`

- [ ] Función `calculate_human_likelihood() -> f64`:
  - Usar burstiness y NCD
  - Ponderar métricas
- [ ] Función `calculate_difficulty(human_score: f64) -> u32`:
  - Alta probabilidad humana → D=10
  - Baja probabilidad humana → D=30
  - Gradiente continuo intermedio
- [ ] Implementación del puzzle:
  - Generar challenge (SHA256 de commit + nonce)
  - Encontrar nonce que cumpla condición
  - Verificar solución

### 5.2. Sistema de Clave Pública/Privada
- [ ] Generar clave de firma (Ed25519) en `Init`
- [ ] Almacenar clave privada en directorio seguro (`~/.config/cliff-craft/`)
- [ ] Clave pública en repositorio (opcional)

### 5.3. Firma de Manifesto
- [ ] Construir `ProvenanceManifest` con métricas
- [ ] Generar JSON
- [ ] Firmar con Ed25519
- [ ] Inyectar como Git Trailer

> **Validación de Métricas**: El `ProvenanceManifest` incluirá un campo `human_score` basado en la combinación de Burstiness y NCD. Este score será validado contra umbrales predefinidos para garantizar que solo contribuciones humanas sean certificadas.

### 5.4. Verificación
- [ ] Parsear manifest desde commit
- [ ] Verificar firma
- [ ] Re-calcular métricas (verificar frescura)
- [ ] Validar PoW

### 5.5. Pruebas de Integración
- [ ] Test end-to-end en repo temporal
- [ ] Generar commit con PoHW
- [ ] Verificar trailer inyectado
- [ ] Verificar firma y validación

---

## Fase 6: Hooks de Git (Días 21-23)

### 6.1. Hook pre-commit
- [ ] Llamar a daemon para comenzar monitoreo
- [ ] Esperar fin de edición
- [ ] Generar PoHW
- [ ] Inyectar trailer en mensaje
- [ ] Retornar 0 (éxito)

### 6.2. Hook post-commit
- [ ] Limpiar estado del daemon
- [ ] Cerrar registro de telemetría
- [ ] Opcional: enviar métricas (anonimizadas)

### 6.3. Instalación Automática
**Script**: `scripts/install_hooks.sh`

- [ ] Detectar repo git
- [ ] Copiar hooks a `.git/hooks/`
- [ ] Configurar permisos
- [ ] Verificar instalación

### 6.4. Pruebas
- [ ] Instalar hooks manualmente
- [ ] Hacer commit normal
- [ ] Verificar que se genera PoHW
- [ ] Verificar que el commit falla si PoHW inválido

---

## Fase 7: Validación y Pruebas (Días 24-26)

### 7.1. Pruebas Unitarias
- [ ] Cobertura > 80% en core
- [ ] Tests de integración para cada crate
- [ ] Tests de rendimiento (benchmarks)

### 7.2. Pruebas de Integración
- [ ] Flujo completo: init → edit → commit → verify
- [ ] Edge cases: archivos vacíos, grandes repos
- [ ] Plataformas: Linux, macOS, Windows (si es posible)

### 7.3. Pruebas de Seguridad
- [ ] Ataques de replay (verificar que no se reutilice nonce)
- [ ] Falsificación de métricas (detectar intentos)
- [ ] Inyección de trailers maliciosos

### 7.4. Pruebas de Rendimiento
- [ ] Latencia del commit (debe ser < 1s para usuarios humanos)
- [ ] Consumo de memoria (debe ser < 50MB)
- [ ] Escalabilidad (múltiples repos simultáneos)

### 7.5. Benchmark de Hardware
- [ ] Probar en Raspberry Pi (referencia low-end)
- [ ] Probar en máquina de desarrollo estándar
- [ ] Comparar tiempos de PoW

---

## Fase 8: Documentación y Publicación (Días 27-30)

### 8.1. Documentación Técnica
- [ ] API Reference (cargo doc)
- [ ] Guía de arquitectura
- [ ] Especificaciones de protocolo (formales)
- [ ] Guía de usuario (instalación, uso)

### 8.2. Documentación para Contribuidores
- [ ] README con setup de desarrollo
- [ ] Guía de codificación (estilo, patrones)
- [ ] Roadmap de contribuciones

### 8.3. Scripts de Build/Instalación
- [ ] `Makefile` con targets:
  - `make dev`: desarrollo rápido
  - `make release`: build optimizado
  - `make test`: ejecutar pruebas
  - `make install`: instalar en sistema
  - `make clean`: limpiar artefactos

### 8.4. Paquetes de Distribución
- [ ] Binarios estáticos (musl) para Linux
- [ ] Paquetes .deb/.rpm
- [ ] Homebrew formula (macOS)
- [ ] Archiso/PPA (opcional)

### 8.5. Publicación Inicial
- [ ] Tag de versión v0.1.0
- [ ] Release en GitHub
- [ ] README actualizado con enlaces
- [ ] Anuncio en README sobre estado pre-alpha

---

## Fase 9: Roadmap Post-Lanzamiento (Futuro)

### 9.1. Integraciones (MVP 0.2)
- [ ] GitHub Action para verificación
- [ ] Webhook para notificaciones
- [ ] Dashboard web (opcional)

### 9.2. Mejoras de Métricas (MVP 0.3)
- [ ] Análisis de patrones de código (estilo)
- [ ] Detección de copia-pega
- [ ] Perfiles de usuario (human score acumulativo)

### 9.3. Protocolo de gobernanza (MVP 0.4)
- [ ] Sistema de votación para métricas
- [ ] Delegación de confianza
- [ ] Red descentralizada (P2P)

### 9.4. Comunidad (Continuo)
- [ ] Sistema de contribuciones
- [ ] Bug bounty
- [ ] Bug tracking (issues)
- [ ] Roadmap público

---

## Consideraciones de Seguridad

### 1. Protección de Datos
- [ ] Nunca registrar contenido de archivos
- [ ] Encriptar métricas temporales
- [ ] Limpiar cachés después de commit
- [ ] No enviar datos a servidores externos (opt-in)

### 2. Integridad del Sistema
- [ ] Validar checksums de binarios
- [ ] Verificar firma de commits en hook
- [ ] Protección contra race conditions
- [ ] Manejo seguro de señales (graceful shutdown)

### 3. Privacidad
- [ ] No keylogger (solo eventos de sistema de archivos)
- [ ] Mínimo de datos necesarios
- [ ] Anonimización en reportes (opcional)
- [ ] Local-first processing

> **Implementación de Privacidad**: Se garantizará que el sistema no capture contenido sensible de los archivos. Solo se registrarán métricas de proceso (tiempos de edición, patrones de actividad) y se evitará el almacenamiento de datos personales o código fuente. Las métricas temporales se encriptarán y limpiarán después de cada commit.

### 4. Robustez
- [ ] Manejo de errores en daemon (crash ≠ data loss)
- [ ] Recovery si el daemon muere
- [ ] Consistencia de estado
- [ ] Timeout en operaciones

---

## Optimizaciones (LEAN)

### 1. Tamaño del Binario
- [ ] Compilación static (musl)
- [ ] LTO activado
- [ ] Strip de debug symbols
- [ ] Opt-level "z"

### 2. Rendimiento
- [ ] Streaming API en zstd
- [ ] Debounce eficiente
- [ ] No bloquear el main thread
- [ ] Uso de threads solo cuando es necesario

### 3. Dependencias
- [ ] Feature flags mínimas en cada crate
- [ ] Revisar `cargo tree` periódicamente
- [ ] Evitar transitive dependencies pesados
- [ ] Considerar reemplazos si hay bloat

---

## Checklist de Seguridad por Fase

### Pre-Compilación
- [ ] Revisar dependencias con `cargo audit`
- [ ] Verificar licencias de crates
- [ ] Revisar transitive dependencies

### En Desarrollo
- [ ] Clippy con nivel `pedantic`
- [ ] Formateo con rustfmt
- [ ] Tests obligatorios
- [ ] No usar `unsafe` sin justificación

### En Build
- [ ] `cargo deny` (licencias, vulnerabilidades)
- [ ] Build reproducible
- [ ] Verificar que no se incluyen credenciales

### En Runtime
- [ ] Sin leaks de memoria (miri)
- [ ] Sin panics inesperados
- [ ] Logs seguros (sin datos sensibles)
- [ ] Manejo de señales adecuado

---

## Métricas de Éxito por Fase

### Fase 1 (Workspace)
- [ ] Workspace compila sin warnings
- [ ] Binario release < 5MB
- [ ] Tiempo de compile < 2min

### Fase 2 (Core)
- [ ] Tests core pasan 100%
- [ ] `sentinel_self_check()` funciona
- [ ] No usa `unsafe`

### Fase 3 (CLI)
- [ ] Binario < 2MB
- [ ] Help funciona
- [ ] System-check muestra reporte

### Fase 4 (Daemon)
- [ ] Consume < 50MB RAM
- [ ] Latencia commit < 1s
- [ ] No bloquea I/O

### Fase 5 (PoHW)
- [ ] Puzzle resoluble en < 30s
- [ ] Firma criptográfica verificable
- [ ] No leaks de claves

### Fase 6 (Hooks)
- [ ] Hook no rompe workflow git
- [ ] Instalación automática funciona
- [ ] Rollback posible

### Fase 7 (Pruebas)
- [ ] Cobertura > 80%
- [ ] Tests de seguridad OK
- [ ] Benchmarks satisfactorios

### Fase 8 (Publicación)
- [ ] Docs generadas
- [ ] Release en GitHub
- [ ] README con ejemplos

---

## Notas Importantes

### 1. Evitar Over-Engineering
- [ ] Implementar MVP primero
- [ ] No añadir features no esenciales
- [ ] Refactorizar después de tests

### 2. Priorizar Seguridad
- [ ] No sacrificar seguridad por velocidad
- [ ] Revisar ataque surface
- [ ] Documentar supuestos de seguridad

### 3. Mantener LEAN
- [ ] Revisar tamaño de binarios
- [ ] Optimizar compilation time
- [ ] Minimizar dependencias

### 4. Documentación en Paralelo
- [ ] Escribir docs con código
- [ ] Mantener README actualizado
- [ ] Changelog de cada versión

---

## Próximos Pasos Inmediatos

1. **Hoy**: 
   - [ ] Crear estructura de directorios
   - [ ] Inicializar Git repo
   - [ ] Crear `.gitignore`

2. **Mañana**:
   - [ ] Crear `Cargo.toml` del workspace
   - [ ] Crear primer crate (cliff-craft-core)
   - [ ] Verificar compilación básica

3. **Semanal**:
   - [ ] Completar Fase 1
   - [ ] Comenzar Fase 2
   - [ ] Mantener checklist actualizado

---

*Roadmap generado: 19-01-2026 16:26*
*Revisar periódicamente y actualizar según avances*
