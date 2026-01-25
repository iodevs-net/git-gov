use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::focus_session::FocusMetrics;

/// Abre un repositorio Git en la ruta especificada
pub fn open_repository(path: &Path) -> Result<Repository, String> {
    Repository::open(path).map_err(|e| format!("Failed to open repository: {}", e))
}

/// Obtiene el último commit de un repositorio
pub fn get_latest_commit(repo: &Repository) -> Result<git2::Commit<'_>, String> {
    let head = repo.head().map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let commit = head.peel_to_commit().map_err(|e| format!("Failed to peel to commit: {}", e))?;
    Ok(commit)
}

/// Crea una firma para commits
pub fn create_signature<'a>(name: &'a str, email: &'a str) -> Result<Signature<'a>, String> {
    Signature::now(name, email).map_err(|e| format!("Failed to create signature: {}", e))
}

/// Agrega un trailer a un mensaje de commit
pub fn add_trailer(message: &str, key: &str, value: &str) -> String {
    format!("{}\n{}: {}", message.trim(), key, value)
}

/// Verifica si un commit tiene un trailer específico
pub fn has_trailer(commit: &git2::Commit, key: &str) -> Result<bool, String> {
    let message = commit.message().ok_or("No commit message")?;
    Ok(message.lines().any(|line| line.starts_with(&format!("{}:", key))))
}

/// Instala los hooks de cliff-craft en el repositorio
pub fn install_hooks(repo: &Repository) -> Result<(), String> {
    let hooks_dir = repo.path().join("hooks");
    if !hooks_dir.exists() {
        std::fs::create_dir_all(&hooks_dir).map_err(|e| e.to_string())?;
    }

    // 1. Hook de preparación de mensaje (para añadir trailers firmados y certificación v2.0)
    let prepare_hook_path = hooks_dir.join("prepare-commit-msg");
    let prepare_hook_content = r#"#!/bin/bash
# cliff-craft hook: Añade el ticket firmado y certificación v2.0
GOV_DIR=".git/cliff-craft"

# v1.0: PoHW Score
TICKET_FILE="$GOV_DIR/latest_ticket"
if [ -f "$TICKET_FILE" ]; then
    TICKET_DATA=$(cat "$TICKET_FILE")
    git interpret-trailers --in-place --trailer "cliff-craft-score: $TICKET_DATA" "$1"
    rm "$TICKET_FILE"
fi

# v2.0: Proof of Focus Witness
WITNESS_FILE="$GOV_DIR/latest_witness"
if [ -f "$WITNESS_FILE" ]; then
    WITNESS_DATA=$(cat "$WITNESS_FILE")
    git interpret-trailers --in-place --trailer "Cliff-Craft-Witness: $WITNESS_DATA" "$1"
    rm "$WITNESS_FILE"
fi
"#;
    std::fs::write(&prepare_hook_path, prepare_hook_content).map_err(|e| e.to_string())?;

    // 2. Hook de pre-commit (La "Aduana Termodinámica")
    let pre_hook_path = hooks_dir.join("pre-commit");
    let pre_hook_content = r#"#!/bin/bash
# cliff-craft hook: Aduana Termodinámica
# Bloquea el commit si no hay suficiente energía acumulada.

# El CLI se encarga de calcular el costo, pedir el ticket y GUARDARLO
cliff-craft verify-work
if [ $? -ne 0 ]; then
    echo "--------------------------------------------------------"
    echo "❌ ERROR: ADUANA TERMODINÁMICA DE GIT-GOV"
    echo "Tu reserva de energía kinética es insuficiente para"
    echo "la complejidad de este código. Dedica más tiempo a la"
    echo "curaduría manual antes de intentar commitear."
    echo "--------------------------------------------------------"
    exit 1
fi
"#;
    std::fs::write(&pre_hook_path, pre_hook_content).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for hook in &["prepare-commit-msg", "pre-commit"] {
            let path = hooks_dir.join(hook);
            let mut perms = std::fs::metadata(&path).map_err(|e| e.to_string())?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Elimina los hooks de cliff-craft del repositorio
pub fn remove_hooks(repo: &Repository) -> Result<(), String> {
    let hooks_dir = repo.path().join("hooks");
    for hook in &["prepare-commit-msg", "pre-commit"] {
        let path = hooks_dir.join(hook);
        if path.exists() {
            // Solo borramos si el archivo contiene "cliff-craft" para no borrar hooks de terceros
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains("cliff-craft") {
                    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                }
            }
        }
    }
    Ok(())
}

/// Obtiene el diff de los archivos staged
pub fn get_staged_diff(repo: &Repository) -> Result<String, String> {
    let mut opts = git2::DiffOptions::new();
    let head = repo.head().ok();
    let diff = match head {
        Some(h) => {
            let tree = h.peel_to_tree().map_err(|e| e.to_string())?;
            repo.diff_tree_to_index(Some(&tree), None, Some(&mut opts))
        }
        None => {
            // Repositorio vacío, comparamos contra un árbol vacío
            repo.diff_tree_to_index(None, None, Some(&mut opts))
        }
    }.map_err(|e| e.to_string())?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_, _, line: git2::DiffLine| {
        diff_text.push(line.origin());
        if let Ok(s) = std::str::from_utf8(line.content()) {
            diff_text.push_str(s);
        }
        true
    }).map_err(|e| e.to_string())?;

    Ok(diff_text)
}


#[derive(Debug, Serialize, Deserialize)]
struct TrustConfig {
    keys: Vec<TrustedKey>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrustedKey {
    alias: String,
    public_key: String,
    #[serde(default)]
    role: String,
}

/// Registra una clave pública en el repositorio (preferentemente en trust.toml)
pub fn register_public_key(repo: &git2::Repository, key_hex: &str, alias: &str) -> Result<(), String> {
    // 1. Try to use trust.toml first (Distributed Trust)
    let trust_toml_path = repo.workdir().map(|w| w.join("trust.toml"));
    
    if let Some(path) = trust_toml_path {
        let mut config = if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            toml::from_str::<TrustConfig>(&content).unwrap_or(TrustConfig { keys: vec![] })
        } else {
            TrustConfig { keys: vec![] }
        };

        // Check for duplicates
        if !config.keys.iter().any(|k| k.alias == alias) {
            config.keys.push(TrustedKey {
                alias: alias.to_string(),
                public_key: key_hex.to_string(),
                role: "Contributor".to_string(),
            });

            let toml_string = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
            std::fs::write(&path, toml_string).map_err(|e| e.to_string())?;
            return Ok(());
        } else {
             // Update existing? For now, just skip or error.
             return Err(format!("Alias '{}' already exists in trust.toml", alias));
        }
    }

    // 2. Fallback to local .git/cliff-craft/trusted_keys (Legacy)
    let gov_dir = repo.path().join("cliff-craft");
    if !gov_dir.exists() {
        std::fs::create_dir_all(&gov_dir).map_err(|e| e.to_string())?;
    }
    
    let keys_file = gov_dir.join("trusted_keys");
    let entry = format!("{}:{}\n", alias, key_hex);
    
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(keys_file)
        .map_err(|e| e.to_string())?;
        
    file.write_all(entry.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Obtiene el mapa de claves públicas confiables (trust.toml + legacy)
pub fn get_trusted_keys(repo: &git2::Repository) -> Result<std::collections::HashMap<String, String>, String> {
    let mut keys = std::collections::HashMap::new();

    // 1. Read trust.toml (Distributed)
    if let Some(workdir) = repo.workdir() {
        let trust_toml = workdir.join("trust.toml");
        if trust_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(trust_toml) {
                if let Ok(config) = toml::from_str::<TrustConfig>(&content) {
                    for k in config.keys {
                        keys.insert(k.alias, k.public_key);
                    }
                }
            }
        }
    }

    // 2. Read legacy .git/cliff-craft/trusted_keys (Local override)
    let legacy_file = repo.path().join("cliff-craft").join("trusted_keys");
    if legacy_file.exists() {
        if let Ok(content) = std::fs::read_to_string(legacy_file) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    keys.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
    }

    Ok(keys)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceEntry {
    pub commit: String,
    pub author: String,
    pub score: f64,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MetricsCache {
    entries: std::collections::HashMap<String, GovernanceEntry>,
}

/// Obtiene el historial de gobernanza (commits firmados) optimizado con Cache
pub fn get_governance_history(repo: &git2::Repository, limit: usize) -> Result<Vec<GovernanceEntry>, String> {
    let mut entries = Vec::new();
    let mut cache = MetricsCache::default();
    let gov_dir = repo.path().join("cliff-craft");
    let cache_file = gov_dir.join("metrics_cache.json");

    // 1. Load Cache
    if cache_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_file) {
           if let Ok(c) = serde_json::from_str::<MetricsCache>(&content) {
               cache = c;
           }
        }
    }

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;
    revwalk.set_sorting(git2::Sort::TIME).map_err(|e| e.to_string())?;

    let mut new_entries_found = false;

    for oid in revwalk.take(limit) {
        if let Ok(oid) = oid {
            let sha = oid.to_string();
            
            // 2. Check Cache
            if let Some(cached_entry) = cache.entries.get(&sha) {
                // Cloning is cheap compared to parsing git objects
                entries.push(GovernanceEntry {
                    commit: cached_entry.commit.clone(),
                    author: cached_entry.author.clone(),
                    score: cached_entry.score,
                    timestamp: cached_entry.timestamp,
                });
                continue;
            }

            // 3. Parse if miss
            if let Ok(commit) = repo.find_commit(oid) {
                let message = commit.message().unwrap_or("");
                for line in message.lines() {
                     if line.starts_with("cliff-craft-score:") {
                        let score_part = line.replace("cliff-craft-score:", "").trim().to_string();
                        let parts: Vec<&str> = score_part.split(":sig=").collect();
                        if !parts.is_empty() {
                            let score_str = parts[0].replace("score=", "");
                             if let Ok(score) = score_str.parse::<f64>() {
                                 let entry = GovernanceEntry {
                                     commit: sha.clone(),
                                     author: commit.author().name().unwrap_or("Unknown").to_string(),
                                     score,
                                     timestamp: commit.time().seconds(),
                                 };
                                 // Add to result
                                 entries.push(GovernanceEntry {
                                    commit: entry.commit.clone(),
                                    author: entry.author.clone(),
                                    score: entry.score,
                                    timestamp: entry.timestamp,
                                 });
                                 // Add to cache
                                 cache.entries.insert(sha.clone(), entry);
                                 new_entries_found = true;
                             }
                        }
                     }
                }
            }
        }
    }
    
    // 4. Update Cache (Atomicish write)
    if new_entries_found {
        if !gov_dir.exists() {
             let _ = std::fs::create_dir_all(&gov_dir);
        }
        if let Ok(json) = serde_json::to_string(&cache) {
            let _ = std::fs::write(cache_file, json);
        }
    }
    
    Ok(entries)
}

// =============================================================================
// SECCIÓN: Cliff-Craft Witness (Certificación v2.0)
// =============================================================================

/// Niveles de probabilidad de que el código fue escrito por un humano
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HumanProbability {
    /// Alta probabilidad: tiempo de foco significativo + ediciones
    High,
    /// Probabilidad media: algo de foco, pocas ediciones
    Medium,
    /// Probabilidad baja: copy-paste o asistente IA
    Low,
    /// Sin datos: daemon no detectó actividad
    Unknown,
}

impl std::fmt::Display for HumanProbability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HumanProbability::High => write!(f, "High"),
            HumanProbability::Medium => write!(f, "Medium"),
            HumanProbability::Low => write!(f, "Low"),
            HumanProbability::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Datos del Witness para el trailer Cliff-Craft-Witness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessData {
    /// Minutos totales de foco activo durante la sesión
    pub focus_time_mins: f64,
    /// Cantidad de ráfagas de edición detectadas
    pub edit_bursts: usize,
    /// Cantidad de archivos únicos tocados
    pub files_touched: usize,
    /// Probabilidad calculada de autoría humana
    pub human_probability: HumanProbability,
    /// Versión del protocolo de certificación
    pub version: String,
}

impl WitnessData {
    /// Crea WitnessData desde FocusMetrics
    pub fn from_metrics(metrics: &FocusMetrics) -> Self {
        Self {
            focus_time_mins: (metrics.total_focus_mins * 100.0).round() / 100.0,
            edit_bursts: metrics.edit_burst_count,
            files_touched: metrics.unique_files,
            human_probability: calculate_human_probability(metrics),
            version: "2.0".to_string(),
        }
    }

    /// Serializa a JSON compacto para el trailer
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Calcula la probabilidad de que el código fue escrito por un humano
/// basándose en las métricas de foco recolectadas.
/// 
/// ## Criterios
/// - **High**: >= 5 min de foco O >= 10 edit bursts O >= 3 archivos
/// - **Medium**: >= 1 min de foco O >= 3 edit bursts O >= 1 archivo
/// - **Low**: < 1 min de foco Y < 3 edit bursts
/// - **Unknown**: Sin métricas
pub fn calculate_human_probability(metrics: &FocusMetrics) -> HumanProbability {
    let score = metrics.total_focus_mins * 10.0 
        + metrics.edit_burst_count as f64 * 5.0 
        + metrics.unique_files as f64 * 3.0
        + metrics.navigation_events as f64 * 1.0;

    if score >= 50.0 {
        HumanProbability::High
    } else if score >= 15.0 {
        HumanProbability::Medium
    } else if score > 0.0 {
        HumanProbability::Low
    } else {
        HumanProbability::Unknown
    }
}

/// Genera un trailer Cliff-Craft-Witness para agregar al mensaje de commit
/// 
/// ## Formato
/// ```text
/// Cliff-Craft-Witness: {"focus_time_mins":5.25,"edit_bursts":12,"files_touched":3,"human_probability":"high","version":"2.0"}
/// ```
/// 
/// ## Uso
/// El hook `prepare-commit-msg` o la CLI pueden usar esta función para
/// inyectar el trailer automáticamente.
pub fn generate_witness_trailer(metrics: &FocusMetrics) -> String {
    let witness = WitnessData::from_metrics(metrics);
    format!("Cliff-Craft-Witness: {}", witness.to_json())
}

/// Agrega el trailer Cliff-Craft-Witness a un mensaje de commit existente
/// 
/// Si el mensaje ya tiene un trailer Cliff-Craft-Witness, no lo duplica.
pub fn inject_witness_trailer(message: &str, metrics: &FocusMetrics) -> String {
    // No duplicar si ya existe
    if message.contains("Cliff-Craft-Witness:") {
        return message.to_string();
    }
    
    add_trailer(message, "Cliff-Craft-Witness", &WitnessData::from_metrics(metrics).to_json())
}

/// Extrae los datos de Cliff-Craft-Witness de un mensaje de commit
pub fn extract_witness_data(message: &str) -> Option<WitnessData> {
    for line in message.lines() {
        if let Some(json_str) = line.strip_prefix("Cliff-Craft-Witness: ") {
            if let Ok(data) = serde_json::from_str::<WitnessData>(json_str) {
                return Some(data);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_trailer() {
        let message = "Initial commit";
        let new_message = add_trailer(message, "cliff-craft-score", "0.85");
        assert!(new_message.contains("cliff-craft-score: 0.85"));
    }
    
    #[test]
    fn test_has_trailer() {
        let dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let repo_path = dir.join(format!("test_repo_{}", timestamp));
        let repo = Repository::init(&repo_path).unwrap();
        let signature = create_signature("Test User", "test@example.com").unwrap();
         
        // Crear un archivo primero
        let file_path = repo_path.join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();
        
        // Añadir el archivo al índice
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
         
        // Crear un commit con trailer
        let message = "Initial commit\ncliff-craft-score: 0.85";
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let _commit = repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[]).unwrap();
         
        let commit = get_latest_commit(&repo).unwrap();
        let has_trailer = has_trailer(&commit, "cliff-craft-score").unwrap();
        assert!(has_trailer);
    }
}