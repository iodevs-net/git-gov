/// Cliff-Watch UI Templates
/// v3.5 - Aura Premium (High-Fidelity AI Forge)

pub const BOLD: &str = "\\033[1m";
pub const ITALIC: &str = "\\033[3m";
pub const CYAN: &str = "\\033[38;5;51m";    // Cyan vibrante
pub const VIOLET: &str = "\\033[38;5;141m";  // Violeta suave
pub const GREEN: &str = "\\033[38;5;82m";    // Verde neón
pub const ORANGE: &str = "\\033[38;5;214m";  // Naranja cálido
pub const RED: &str = "\\033[38;5;196m";     // Rojo intenso
pub const GREY: &str = "\\033[38;5;244m";    // Gris neutro
pub const NC: &str = "\\033[0m";

/// Genera el contenido del hook pre-commit al estilo Aura Premium.
pub fn render_pre_commit_hook() -> String {
    format!(
        r#"#!/bin/bash
# cliff-watch hook: Protocolo de Soberanía Técnica
# v3.5 - Aura Premium Layout

# Colores y Formatos ANSI
BOLD='{BOLD}'
ITALIC='{ITALIC}'
CYAN='{CYAN}'
VIOLET='{VIOLET}'
GREEN='{GREEN}'
ORANGE='{ORANGE}'
RED='{RED}'
GREY='{GREY}'
NC='{NC}'

# Identificamos archivos en el área de preparación (staged)
STAGED_FILES=$(git diff --cached --name-only)

# Verificación de Binario
CLI_CMD="cliff-watch"
if ! command -v $CLI_CMD &> /dev/null; then
    CLI_CMD="cliff-watch-cli"
    if ! command -v $CLI_CMD &> /dev/null; then
        if [ -f "target/debug/cliff-watch-cli" ]; then CLI_CMD="./target/debug/cliff-watch-cli"
        elif [ -f "target/debug/cliff-watch" ]; then CLI_CMD="./target/debug/cliff-watch"
        else echo -e "${{ORANGE}}* Cliff-Watch: Binario no detectado. Omitiendo auditoría.${{NC}}"; exit 0; fi
    fi
fi

$CLI_CMD verify-work &> /dev/null
if [ $? -ne 0 ]; then
    echo -e ""
    echo -e "  ${{VIOLET}}┌──────────────────────────────────────────────────────────┐${{NC}}"
    echo -e "  ${{VIOLET}}│${{NC}}  ${{BOLD}}${{CYAN}}CLIFF-WATCH${{NC}}  ${{GREY}}// Governance Protocol v3.5${{NC}}         ${{VIOLET}}│${{NC}}"
    echo -e "  ${{VIOLET}}└──────────────────────────────────────────────────────────┘${{NC}}"
    echo -e ""
    echo -e "  ${{CYAN}}      _                        ${{VIOLET}}  __  __ ${{NC}}"
    echo -e "  ${{CYAN}}     | |      _ _   __   __    ${{VIOLET}} |  \/  |${{NC}}"
    echo -e "  ${{CYAN}}     | |     | '_| | _| | _|   ${{VIOLET}} | @  @ |${{NC}}"
    echo -e "  ${{CYAN}}     | |___  | |   | _| | _|   ${{VIOLET}} |  /\  |${{NC}}"
    echo -e "  ${{CYAN}}     |_____| |_|   |_|  |_|    ${{VIOLET}}  \____/ ${{NC}}"
    echo -e ""
    echo -e "  ${{BOLD}}ADVISORY${{NC}}  ${{GREY}}──────────────────────────────────────────────${{NC}}"
    echo -e "  ${{GREY}}Session:${{NC}}  $(date +'%H:%M:%S @ %Y-%m-%d')"
    echo -e "  ${{GREY}}Status:${{NC}}   ${{RED}}INTERRUPTED${{NC}}"
    echo -e ""
    echo -e "  ${{BOLD}}STAGED COMPONENTS:${{NC}}"
    while read -r file; do
        if [ ! -z "$file" ]; then echo -e "  ${{GREEN}}›${{NC}} ${{GREY}}$file${{NC}}"; fi
    done <<< "$STAGED_FILES"
    echo -e ""
    echo -e "  ${{BOLD}}DEFICIT DETECTED:${{NC}}"
    echo -e "  ${{ITALIC}}${{ORANGE}}Technical focus evidence is below the required sovereignty threshold.${{NC}}"
    echo -e "  This commit was blocked to prevent technical debt and ensure craftsmanship."
    echo -e ""
    echo -e "  ${{BOLD}}GUIDANCE:${{NC}}"
    echo -e "  • Re-check your logic for complex patterns or hidden bugs."
    echo -e "  • Ensure the changes reflect the high-fidelity standards of ioDesk."
    echo -e "  • Curation is the only path to true sovereignty."
    echo -e ""
    echo -e "  ${{GREY}}──────────────────────────────────────────────────────────────${{NC}}"
    echo -e ""
    exit 1
fi
"#,
        BOLD=BOLD, ITALIC=ITALIC, CYAN=CYAN, VIOLET=VIOLET, 
        GREEN=GREEN, ORANGE=ORANGE, RED=RED, GREY=GREY, NC=NC
    )
}
