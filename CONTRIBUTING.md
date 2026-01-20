# Contributing to Git-Gov

## Testing Localmente

Para probar Git-Gov en tus propios repositorios ("Dogfooding"):

1.  **Compilar Release:**
    ```bash
    cargo build --release
    ```

2.  **Instalar en PATH (Temporal):**
    ```bash
    export PATH=$PATH:$(pwd)/target/release
    ```

3.  **Inicializar en otro repo:**
    ```bash
    cd /path/to/other/repo
    git-gov init
    ```

## Desarrollo

- Usamos **SemVer** y **Conventional Commits**.
- **Zero Warnings Policy**: El código no debe tener warnings de `cargo check`.
- **Lean Philosophy**: No over-engineering. Soluciones atómicas.
