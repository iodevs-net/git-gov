# ROADMAP DE CORRECCIÓN ARQUITECTÓNICA

## 📅 FASE 1: Estabilización Crítica (Día 1)

### 1.1 Arreglar error de compilación (30 min)
- ✅ Corregir `mouse_sentinel.rs:124` (delimitador sin cerrar)
- ✅ Ejecutar `cargo check` para validar
- ✅ Verificar que todas las pruebas existentes pasan

### 1.2 Eliminar duplicados (15 min)
- ❌ Eliminar `sentinel_hash.rs` o `sentinel_hasher.rs` (elegir uno)
- ❌ Verificar que no haya otros duplicados
- ❌ Actualizar imports en archivos que los usen

### 1.3 Reducir superficie (60 min)
- ❌ Identificar los 3 módulos más esenciales
- ❌ Mover los demás a una rama `feature/future-architecture`
- ❌ Dejar solo lo necesario para el MVP actual

## 📅 FASE 2: Calidad Mínima (Día 2)

### 2.1 Añadir tests mínimos (120 min)
- ❌ 1 test por módulo nuevo (happy path + 1 error)
- ❌ Tests de serialización/deserialización
- ❌ Verificar cobertura mínima del 80%

### 2.2 Documentación básica (60 min)
- ❌ Añadir `//!` a nivel de archivo (responsabilidad)
- ❌ Añadir `///` en structs públicos
- ❌ Documentar ejemplos de uso

## 📅 FASE 3: Integración (Día 3)

### 3.1 Validación final (60 min)
- ❌ Ejecutar `cargo test --all`
- ❌ Verificar que no haya warnings
- ❌ Validar que el código compile en CI

### 3.2 Preparación para PR (30 min)
- ❌ Actualizar `CHANGELOG.md`
- ❌ Escribir descripción clara del PR
- ❌ Explicar decisiones de diseño

## 📅 FASE 4: Futuro (Post-MVP)

### 4.1 Arquitectura completa (Iteraciones futuras)
- ❌ Mover módulos postergados de vuelta
- ❌ Añadir tests exhaustivos
- ❌ Documentación completa

## ✅ Checklist de Aceptación

- [ ] Código compila sin errores
- [ ] Tests pasan (100%)
- [ ] Sin warnings de compilación
- [ ] Documentación mínima presente
- [ ] Superficie reducida al MVP

## 🎯 Objetivo Final

Tener un código **estable, probado y documentado** que pueda ser integrado sin fricción.