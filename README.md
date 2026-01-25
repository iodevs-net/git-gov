# Cliff-Craft: Proof of Focus & Code Provenance 🛡️🧠

> **"Confianza descentralizada en la era de la IA Generativa."**

**Cliff-Craft** es un estándar abierto y una suite de herramientas para certificar la **atención humana** en el desarrollo de software.

En un mundo donde el código puede generarse en milisegundos a coste cero, la **atención humana** se convierte en el recurso más escaso y valioso. Cliff-Craft permite a los desarrolladores probar criptográficamente que han dedicado tiempo y foco a su trabajo, sin sacrificar su privacidad.

---

## 🌪️ El Problema: La Crisis de Entropía

La adopción masiva de LLMs (Modelos de Lenguaje) ha creado un nuevo desafío para los mantenedores de Open Source:

1. **Spam de Alta Velocidad:** Pull Requests (PRs) generados por bots o usuarios que "copian y pegan" sin revisar.
2. **Ilusión de Competencia:** Código sintácticamente correcto pero lógicamente frágil o alucinatorio.
3. **Fatiga del Mantenedor:** Imposibilidad de distinguir entre un PR cuidado artesanalmente y uno generado en 3 segundos.

Los detectores de IA actuales analizan el *texto* (y fallan a menudo). **Cliff-Craft analiza el *proceso*.**

---

## 💡 La Solución: Proof of Focus (PoF)

Cliff-Craft implementa un protocolo de **"Testigo Silencioso"** que certifica el esfuerzo cognitivo sin espiar al usuario.

En lugar de bloquear commits o exigir permisos invasivos, Cliff-Craft actúa como un notario digital:

1. **El Testigo (IDE Extension):** Una extensión ligera en tu editor (VS Code, JetBrains) detecta si estás trabajando activamente en un archivo (foco de ventana, patrones de edición, scroll de lectura). **No registra lo que escribes.**
2. **El Notario (Local Daemon):** Un proceso en segundo plano (sin privilegios de root) acumula "Créditos de Foco" basados en tu actividad real.
3. **La Insignia (Git Trailer):** Al hacer commit, si tienes suficientes créditos, Cliff-Craft firma criptográficamente el commit añadiendo metadatos verificables.

### El Resultado:

Un commit firmado que le dice al mantenedor:

> *"Este código fue editado y revisado por un humano durante 25 minutos antes de ser enviado."*

---

## 🏗️ Arquitectura v2.0 (Privacidad por Diseño)

A diferencia de herramientas de monitoreo intrusivas, Cliff-Craft está diseñado bajo principios estrictos de privacidad:

* ✅ **Sin Root:** No requiere permisos de administrador ni acceso al Kernel (`/dev/input`).
* ✅ **Sin Keylogger:** No registramos teclas ni contenido del código. Solo métricas de metadatos (tiempo de foco, frecuencia de edición).
* ✅ **Local-First:** Todos los datos se procesan en tu máquina. Nada sale de tu red.

```mermaid
graph LR
    IDE[VS Code / Editor] -- "Actividad de Foco" --> Daemon[Cliff-Craft Daemon (User Space)]
    Daemon -- "Firma Criptográfica (Ed25519)" --> Git[Git Trailer]
    Git -- "Commit Verificado" --> Repo[Repositorio Remoto]
    CI[CI/CD] -- "Verifica Firma" --> Badge[Insignia de Calidad]

```

---

## 🚀 Guía de Inicio Rápido

### Para Desarrolladores (Demuestra tu Trabajo)

1. **Instala el CLI:**
```bash
cargo install cliff-craft

```


2. **Instala la Extensión en tu IDE:**
Busca "Cliff-Craft Witness" en el marketplace de VS Code (o tu editor favorito).
3. **Inicializa en tu Repo:**
```bash
cliff-craft init

```


*¡Listo! Trabaja normalmente. Tus commits ahora llevarán la firma de "Human Verified".*

### Para Mantenedores (Filtra el Ruido)

Integra Cliff-Craft en tu pipeline de CI/CD (GitHub Actions, GitLab CI) para priorizar PRs humanos.

```yaml
# Ejemplo en GitHub Actions
steps:
  - uses: iodevs-net/cliff-craft-action@v2
    with:
      policy: "require-human-focus"
      min-minutes: 15

```

* **Verificado:** El PR recibe una etiqueta verde `human-verified`.
* **No Verificado:** El PR se marca para revisión exhaustiva o se etiqueta como `unverified-source`.

---

## 📜 El Estándar del Trailer

Cliff-Craft utiliza el estándar de **Git Trailers** para asegurar la compatibilidad universal. La firma es inmutable y viaja con el commit.

```git
commit 9a1b2c3d...
Author: Jane Doe <jane@example.com>
Date:   Mon Jan 20 14:00:00 2026 -0300

    Implement new authentication logic

    Signed-off-by: Jane Doe <jane@example.com>
    Cliff-Craft-Witness: {"v":2,"focus_min":24,"bursts":12,"sig":"a1b2..."}

```

---

## 🔮 Roadmap

Estamos pivotando activamente hacia la versión 2.0.

* [x] **Fase 1: El Protocolo & Backend (Completado)** - Definición del esquema JSON IPC e integración en el Daemon.
* [ ] **Fase 2: El Testigo (Siguiente)** - Primera extensión oficial para VS Code.
* [ ] **Fase 3: El Verificador** - GitHub Action para automatizar la revisión de PRs.

---

## 🤝 Contribuye

Cliff-Craft es 100% Open Source y construido en Rust. Buscamos colaboradores que crean en un futuro donde la IA asiste, pero el humano certifica.

* ¿Eres experto en **VS Code Extensions**?
* ¿Te apasiona **Rust** y la criptografía?

¡Únete a la discusión en [Issues] o envía un PR!

---

*Garantizando la soberanía humana en la frontera del bit.*