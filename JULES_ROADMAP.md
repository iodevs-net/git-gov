# Jules Roadmap: Git-Gov Witness (VS Code Extension) 🛡️🧠

Este roadmap define el plan de ejecución atómico para implementar la extensión de VS Code que actúa como sensor de foco para Git-Gov.

## Contexto Técnico
- **Backend**: Daemon en Rust escuchando en `/tmp/git-gov-sensor.sock` (Unix Domain Socket).
- **Frontend**: Extensión VS Code (TypeScript).
- **Protocolo**: [focus_protocol.rs](git-gov/crates/git-gov-core/src/focus_protocol.rs) (Fuente de verdad).

---

## Fase 1: Scaffolding & Certification-First (Pareto 20%)

### 1.1 Estructura base
- [ ] Inicializar en `/clients/vscode-witness`.
- [ ] Instalar dependencias: `vitest`, `fast-check`, `typescript`.

### 1.2 Paridad de Tipos (The Schema)
Implementar exactamente estas interfaces para asegurar paridad con Rust (`serde(tag = "type", rename_all = "snake_case")`):

```typescript
export type NavigationType = 'scroll' | 'file_switch' | 'go_to_definition' | 'hover';

export type SensorEvent =
  | { type: 'focus_gained'; file_path: string | null; timestamp_ms: number }
  | { type: 'focus_lost'; timestamp_ms: number }
  | { type: 'edit_burst'; file_path: string; chars_delta: number; timestamp_ms: number }
  | { type: 'navigation'; file_path: string; nav_type: NavigationType; timestamp_ms: number }
  | { type: 'heartbeat'; timestamp_ms: number }
  | { type: 'disconnect'; timestamp_ms: number };
```

### 1.3 Test de Certificación (Property-Based)
**Misión**: El sistema NUNCA debe emitir JSON inválido, sea cual sea el input.
- [ ] Crear `tests/certification.test.ts`.
- [ ] Usar `fast-check` para generar `SensorEvent` aleatorios.
- [ ] Validar que `JSON.stringify(event)` genera una cadena que cumple el esquema de Rust.

---

## Fase 2: Captura de Eventos (The Witness)

### 2.1 Focus Tracking
- [ ] `onDidChangeWindowState`: Enviar `focus_gained` o `focus_lost`.
- [ ] `onDidChangeActiveTextEditor`: Enviar `focus_gained` con el nuevo path.

### 2.2 Edit Tracking (Bursting)
- [ ] `onDidChangeTextDocument`: Acumular cambios durante 500ms y enviar un único `edit_burst`.
- [ ] **Crucial**: No enviar el contenido, solo el `chars_delta`.

### 2.3 Navigation Tracking
- [ ] `onDidChangeTextEditorVisibleRanges`: Enviar `navigation` tipo `scroll` (throttled).

---

## Fase 3: Transporte (The Pipe)

### 3.1 Unix Socket Client
- [ ] Usar `net.createConnection({ path: '/tmp/git-gov-sensor.sock' })`.
- [ ] **Nota**: Aunque el prompt inicial mencionaba HTTP, hemos decidido usar **Unix Sockets** directamente por razones de seguridad, privacidad y menor latencia. Evita levantar servidores HTTP innecesarios.
- [ ] Implementar reconexión automática (exponential backoff).
- [ ] Fallos silenciosos: Si el daemon no está, la extensión sigue funcionando sin molestar al usuario.

### 3.2 Buffer de Emergencia
- [ ] Pequeño buffer en memoria (max 100 eventos) si el socket está caído momentáneamente.

---

## Fase 4: Pulido & Seguridad

### 4.1 Minimización de Datos
- [ ] Auditoría de logs: Asegurar que NO se imprimen rutas sensibles o pedazos de código en la consola de salida.

### 4.2 Async Safety
- [ ] Asegurar que el envío de eventos es "fire and forget" para no bloquear el loop principal de VS Code.

---

## Cómo Validar tu Progreso (Jules):
1. `npm test` debe ejecutar la suite de `fast-check` con 10,000 runs sin fallos.
2. Al abrir VS Code en el repo de `git-gov`, `nc -l -U /tmp/git-gov-sensor.sock` debe mostrar el flujo de JSONs correcto.
