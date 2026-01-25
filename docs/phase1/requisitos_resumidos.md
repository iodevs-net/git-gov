# Requisitos Clave Resumidos para cliff-craft

## 1. Contexto y Objetivo
- **Crisis de Entropía**: La generación de código mediante IA ha reducido la barrera de entrada para contribuir código, lo que ha llevado a una crisis de entropía en el ecosistema de software de código abierto.
- **Objetivo**: Implementar un sistema de gobernanza de código descentralizado (DCG) llamado `cliff-craft` que certifique la autenticidad de las contribuciones humanas.

## 2. Arquitectura del Sistema
- **Sentinel Protocol**: Un protocolo basado en la física de la información para verificar la autenticidad de las contribuciones.
- **Componentes Principales**:
  - **cliff-craft-core**: Lógica central para el cálculo de métricas y criptografía.
  - **cliff-craft-cli**: Interfaz de línea de comandos para interactuar con el sistema.
  - **cliff-craft-daemon**: Proceso en segundo plano que monitorea la actividad de edición.

## 3. Métricas Clave
- **Burstiness (B)**: Mide la variabilidad en los tiempos de edición para distinguir entre patrones humanos y de máquina.
  - Fórmula: $B = \frac{\sigma_{\tau} - \mu_{\tau}}{\sigma_{\tau} + \mu_{\tau}}$
  - $B \approx 1$: Comportamiento humano (altamente variable).
  - $B \approx -1$: Comportamiento de máquina (uniforme).
- **Normalized Compression Distance (NCD)**: Utiliza compresión para evaluar la complejidad algorítmica del código y detectar redundancias típicas de la generación por IA.
  - Fórmula: $NCD(x, y) = \frac{C(xy) - \min(C(x), C(y))}{\max(C(x), C(y))}$

## 4. Tecnologías y Dependencias
- **Lenguaje**: Rust.
- **Librerías**:
  - `clap`: Para el manejo de la interfaz de línea de comandos.
  - `git2`: Para la interacción con repositorios Git.
  - `notify`: Para la monitorización del sistema de archivos.
  - `zstd`: Para la compresión y cálculo de entropía.
  - `statrs`: Para el análisis estadístico.
  - `ed25519-dalek`: Para la firma criptográfica.
  - `serde` y `serde_json`: Para la serialización de datos.

## 5. Estructura del Proyecto
- **Workspace de Cargo**: El proyecto debe organizarse como un workspace de Cargo con tres crates principales: `cliff-craft-core`, `cliff-craft-cli`, y `cliff-craft-daemon`.
- **Configuración**: Cada crate debe tener una configuración específica para optimizar el tamaño del binario y la eficiencia.

## 6. Funcionalidades Principales
- **Monitoreo de Actividad**: Capturar eventos de edición en tiempo real.
- **Cálculo de Métricas**: Evaluar la entropía y burstiness de las contribuciones.
- **Generación de Pruebas de Trabajo**: Implementar un sistema de prueba de trabajo (PoW) dinámico basado en las métricas calculadas.
- **Firma Criptográfica**: Firmar los metadatos de procedencia para garantizar su autenticidad.

## 7. Requisitos de Implementación
- **Integración con Git**: Utilizar trailers de Git para almacenar metadatos de procedencia de manera inmutable.
- **Optimización para Hardware de Bajos Recursos**: Asegurar que el sistema funcione eficientemente en dispositivos con recursos limitados.
- **Privacidad**: Evitar la captura de contenido sensible y enfocarse en métricas de proceso.

## 8. Pruebas y Validación
- **Pruebas Unitarias y de Integración**: Implementar pruebas para garantizar la correcta funcionalidad del sistema.
- **Validación de Métricas**: Validar que las métricas calculadas puedan distinguir efectivamente entre contribuciones humanas y generadas por IA.

## 9. Documentación y Compliance
- **Cumplimiento de Estándares**: Asegurar que el código cumpla con los estándares de calidad y las mejores prácticas de desarrollo.
- **Documentación**: Documentar el proceso de implementación y los resultados obtenidos.

## 10. Pasos Siguientes
- Implementar la estructura del workspace de Cargo.
- Desarrollar los módulos principales en `cliff-craft-core`.
- Implementar la interfaz de línea de comandos en `cliff-craft-cli`.
- Desarrollar el daemon de monitoreo en `cliff-craft-daemon`.
- Realizar pruebas exhaustivas y validar la funcionalidad del sistema.

Este documento sirve como guía para la implementación del sistema `cliff-craft` y debe ser actualizado a medida que avanza el desarrollo.