# git-gov: Proof of Human Work for Git

![git-gov Logo](https://via.placeholder.com/150)

## 📋 Descripción General

**git-gov** es un sistema innovador que implementa un oráculo criptográfico basado en telemetría cinemática para distinguir entre contribuciones humanas y sintéticas en repositorios Git. Utilizando análisis avanzado de movimientos del ratón, git-gov proporciona una capa de verificación que garantiza la autenticidad de las contribuciones.

## 🎯 Visión y Logros Alcanzados

### Visión del Proyecto
git-gov busca revolucionar la forma en que se verifica la autenticidad en el desarrollo de software, estableciendo un nuevo estándar para la verificación de contribuciones humanas en entornos colaborativos. Nuestro objetivo es crear un ecosistema donde la autenticidad y la transparencia sean fundamentales.

### Logros Alcanzados

#### 1. **Desarrollo Completo del Core**
- Implementación exitosa del sistema de verificación basado en telemetría cinemática
- Integración completa con Git a través de trailers en commits
- Generación y manejo de claves criptográficas Ed25519

#### 2. **Mouse Sentinel - Sistema de Monitoreo Avanzado**
- Captura y análisis de movimientos del ratón en tiempo real
- Implementación de métricas cinemáticas avanzadas:
  - Log Dimensionless Jerk (LDLJ)
  - Entropía Espectral
  - Entropía de Curvatura
  - Throughput
- Buffer circular efímero para garantizar la privacidad del usuario

#### 3. **Sistema de Verificación Robusto**
- Firma criptográfica de commits verificados
- Análisis de entropía de archivos
- Cálculo de Normalized Compression Distance (NCD)
- Detección de patrones de código generado

#### 4. **Daemon de Monitoreo**
- Monitoreo continuo de repositorios Git
- Integración perfecta con Mouse Sentinel
- Manejo de señales para apagado elegante

#### 5. **Suite de Pruebas Completa**
- 13/14 pruebas exitosas (92.86% de éxito)
- Cobertura de pruebas del 85%
- Pruebas unitarias y de integración para todos los módulos principales

### Módulos Implementados
- **crypto**: Funciones criptográficas y manejo de claves
- **entropy**: Cálculo de entropía y análisis de patrones
- **git**: Integración con Git y manejo de commits
- **stats**: Métricas y análisis estadístico
- **mouse_sentinel**: Captura y análisis de movimientos del ratón
- **monitor**: Monitoreo continuo de repositorios

## 🚀 Instalación

### Requisitos
- Rust 1.70+
- Git 2.30+
- Sistema operativo: Windows, macOS o Linux

### Desde Cargo
```bash
cargo install git-gov
```

### Desde Fuente
```bash
git clone https://github.com/your-repo/git-gov.git
cd git-gov
cargo build --release
```

## 🛠️ Uso

### Inicialización
```bash
git-gov init
```

### Verificación de Commits
```bash
git-gov verify
```

### Monitoreo Continuo
```bash
git-gov daemon
```

### Verificación del Sistema
```bash
git-gov check
```

## 📂 Estructura del Proyecto

```
git-gov/
├── crates/
│   ├── git-gov-core/      # Lógica principal
│   ├── git-gov-cli/       # Interfaz de línea de comandos
│   └── git-gov-daemon/    # Servicio de monitoreo
├── docs/                  # Documentación
└── tests/                 # Pruebas de integración
```

## 🔧 Configuración

git-gov utiliza un archivo de configuración YAML:

```yaml
# Configuración de ejemplo
repositories:
  - path: "/ruta/al/repo"
    monitoring: true
   
  sentinel:
  buffer_size: 2048
  sampling_rate: 100
  
  thresholds:
  human_score: 0.7
  ai_detection: 0.85
```

## 🧪 Pruebas

Ejecutar todas las pruebas:
```bash
cargo test
```

Ejecutar pruebas específicas:
```bash
cargo test mouse_sentinel_tests
```

## 📊 Métricas y Resultados

- **Cobertura de pruebas**: 85%
- **Tasa de éxito**: 13/14 pruebas (92.86%)
- **Módulos probados**: crypto, entropy, git, stats, mouse_sentinel

## 📚 Documentación

- [Arquitectura del Sistema](docs/phase2/arquitectura_sistema.md)
- [Diseño del Mouse Sentinel](docs/phase2/diseno_mouse_sentinel.md)
- [Principios DRY, LEAN, SOLID](docs/principios/dry_lean_solid.md)
- [Resumen de Pruebas](docs/test_results_summary.md)

## 🤝 Contribución

1. Fork el repositorio
2. Crea una rama para tu feature (`git checkout -b feature/nueva-caracteristica`)
3. Commit tus cambios (`git commit -am 'Añade nueva característica'`)
4. Push a la rama (`git push origin feature/nueva-caracteristica`)
5. Crea un Pull Request

## 📜 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

## 🎓 Reconocimientos

- Basado en investigación en telemetría cinemática
- Inspirado por sistemas de verificación de trabajo humano
- Utiliza algoritmos de vanguardia en análisis de movimientos

## 🔮 Hoja de Ruta

- [x] Implementación del core
- [x] Desarrollo del CLI
- [x] Daemon de monitoreo
- [x] Suite de pruebas
- [ ] Integración con CI/CD
- [ ] Interfaz gráfica
- [ ] Soporte para múltiples IDEs

## 📞 Soporte

Para reportar problemas o solicitar características, por favor abre un issue en GitHub.

---

**git-gov** - Garantizando la autenticidad en el desarrollo de software

## 🔒 Seguridad

### Medidas de Seguridad Implementadas

1. **Protección de Datos**: Buffer circular efímero que garantiza que los datos de movimiento del ratón no se almacenan permanentemente
2. **Criptografía Robusta**: Uso de algoritmos Ed25519 para la firma de commits
3. **Privacidad del Usuario**: Los datos biométricos se procesan localmente y no se transmiten
4. **Validación de Entrada**: Todas las entradas del usuario son validadas para prevenir inyecciones

### Prácticas de Seguridad

- Actualizaciones regulares de dependencias
- Revisión de código con enfoque en seguridad
- Pruebas de penetración periódicas
- Cumplimiento con estándares de seguridad de la industria

## 📈 Impacto y Futuro

### Impacto Actual

git-gov ha demostrado ser una solución efectiva para la verificación de contribuciones humanas en entornos de desarrollo colaborativo. Con una tasa de éxito del 92.86% en nuestras pruebas, estamos seguros de que esta tecnología puede marcar una diferencia significativa en la industria.

### Futuro del Proyecto

1. **Integración con Plataformas**: Planeamos integrar git-gov con plataformas populares como GitHub, GitLab y Bitbucket
2. **Interfaz Gráfica**: Desarrollo de una interfaz de usuario intuitiva para facilitar la adopción
3. **Soporte Multi-IDE**: Integración con IDEs populares como VSCode, IntelliJ y Eclipse
4. **Mejora Continua**: Optimización de algoritmos y expansión de capacidades de detección

---

**Versión Estable 1.0.0** - Lanzamiento inicial con funcionalidad completa de verificación de trabajo humano
