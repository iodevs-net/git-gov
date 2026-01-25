# Arquitectura cliff-craft v3.0: Mandamientos DRY, LEAN y SOLID

Este documento define las leyes de diseño para la fase de **Gobernanza Soberana v3.0**. Cada línea de código debe ser un testimonio de eficiencia (LEAN), modularidad (SOLID) y elegancia técnica (DRY).

## 1. DRY (Don't Repeat Yourself) - Criptografía y Lógica

### 1.1 Módulos Atómicos de Criptografía
- **Mandamiento**: Ninguna lógica de Bulletproofs o TPM debe estar fuera de `cliff-craft-core::crypto`.
- **Implementación**: El Daemon y la CLI invocan abstracciones limpias. La criptografía es una "Caja Negra" verificable.

### 1.2 Unificación de Protocolos (UDS)
- **Mandamiento**: El protocolo de comunicación entre el IDE (Witness) y el Daemon debe usar un esquema de tipado compartido.
- **Implementación**: Uso de `serde` con tipos definidos en `core` para garantizar que el "Testigo" y el "Juez" hablen el mismo idioma sin duplicar estructuras.

## 2. LEAN (Optimización para Soberanía)

### 2.1 Procesamiento Zero-Footprint
- **Mandamiento**: El Daemon no debe ser un invasor de recursos. Uso de `tokio` para concurrencia no bloqueante.
- **Implementación**: Los cálculos de entropía (Zstd) y estadística (statrs) se realizan en ráfagas efímeras. No se almacena nada que no sea estrictamente necesario para la prueba de rango (ZKP).

### 2.2 Hardware-First (TPM & Enclave)
- **Mandamiento**: No emular en software lo que el hardware puede hacer mejor y más seguro.
- **Implementación**: Delegar la seguridad de claves al TPM 2.0. LEAN significa menos código "fake" de seguridad y más confianza en el silicio.

## 3. SOLID (Separación de Concerns v3)

### 3.1 Single Responsibility: El Testigo vs. El Verificador
- **cliff-craft-witness (IDE)**: Única responsabilidad: Capturar el ritmo humano (CNS). No firma, no juzga.
- **cliff-craft-daemon**: Única responsabilidad: Transformar ritmo en Prueba Criptográfica (ZKP).
- **cliff-craft-cli**: Única responsabilidad: Certificar la prueba antes del commit.

### 3.2 Dependency Inversion: Abstracción de Hardware
- **Mandamiento**: El núcleo no debe saber si estás en Linux o macOS.
- **Implementación**: Creación de un `Trait` de `HardwareSecureEnclave`. La implementación concreta (TPM2 vs Apple SEP) se inyecta en tiempo de compilación.

## 4. El Mandamiento Supremo: Proof of Human Work (PoHW)
Todo código v3 debe contribuir a la misión: **Detectar y bloquear la Bio-Puppetry**. 
- Si un refactor no mejora la detección de entropía o el blindaje criptográfico, es superfluo (Antipattern LEAN).

---
*Actualizado para cliff-craft v2.1/v3.0 - 2026*