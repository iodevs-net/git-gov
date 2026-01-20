use git2::{Repository, Signature};
use std::path::Path;

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

/// Instala los hooks de git-gov en el repositorio
pub fn install_hooks(repo: &Repository) -> Result<(), String> {
    let hooks_dir = repo.path().join("hooks");
    if !hooks_dir.exists() {
        std::fs::create_dir_all(&hooks_dir).map_err(|e| e.to_string())?;
    }

    // 1. Hook de preparación de mensaje (para añadir trailers firmados)
    let prepare_hook_path = hooks_dir.join("prepare-commit-msg");
    let prepare_hook_content = r#"#!/bin/bash
# git-gov hook: Añade el ticket firmado al mensaje del commit
TICKET_FILE=".git/git-gov/latest_ticket"
if [ -f "$TICKET_FILE" ]; then
    TICKET_DATA=$(cat "$TICKET_FILE")
    git interpret-trailers --in-place --trailer "git-gov-score: $TICKET_DATA" "$1"
    rm "$TICKET_FILE"
fi
"#;
    std::fs::write(&prepare_hook_path, prepare_hook_content).map_err(|e| e.to_string())?;

    // 2. Hook de pre-commit (La "Aduana Termodinámica")
    let pre_hook_path = hooks_dir.join("pre-commit");
    let pre_hook_content = r#"#!/bin/bash
# git-gov hook: Aduana Termodinámica
# Bloquea el commit si no hay suficiente energía acumulada.

# El CLI se encarga de calcular el costo, pedir el ticket y GUARDARLO
git-gov verify-work
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

/// Elimina los hooks de git-gov del repositorio
pub fn remove_hooks(repo: &Repository) -> Result<(), String> {
    let hooks_dir = repo.path().join("hooks");
    for hook in &["prepare-commit-msg", "pre-commit"] {
        let path = hooks_dir.join(hook);
        if path.exists() {
            // Solo borramos si el archivo contiene "git-gov" para no borrar hooks de terceros
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains("git-gov") {
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

use serde::{Deserialize, Serialize};

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

    // 2. Fallback to local .git/git-gov/trusted_keys (Legacy)
    let gov_dir = repo.path().join("git-gov");
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

    // 2. Read legacy .git/git-gov/trusted_keys (Local override)
    let legacy_file = repo.path().join("git-gov").join("trusted_keys");
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

#[derive(Debug, Serialize)]
pub struct GovernanceEntry {
    pub commit: String,
    pub author: String,
    pub score: f64,
    pub timestamp: i64,
}

/// Obtiene el historial de gobernanza (commits firmados)
pub fn get_governance_history(repo: &git2::Repository, limit: usize) -> Result<Vec<GovernanceEntry>, String> {
    let mut entries = Vec::new();
    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;
    
    // Sort by time descending
    revwalk.set_sorting(git2::Sort::TIME).map_err(|e| e.to_string())?;

    for oid in revwalk.take(limit) {
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                let message = commit.message().unwrap_or("");
                for line in message.lines() {
                     if line.starts_with("git-gov-score:") {
                        let score_part = line.replace("git-gov-score:", "").trim().to_string();
                        // Format: score=0.85:sig=...
                        let parts: Vec<&str> = score_part.split(":sig=").collect();
                        if !parts.is_empty() {
                            let score_str = parts[0].replace("score=", "");
                             if let Ok(score) = score_str.parse::<f64>() {
                                 entries.push(GovernanceEntry {
                                     commit: oid.to_string(),
                                     author: commit.author().name().unwrap_or("Unknown").to_string(),
                                     score,
                                     timestamp: commit.time().seconds(),
                                 });
                             }
                        }
                     }
                }
            }
        }
    }
    
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_trailer() {
        let message = "Initial commit";
        let new_message = add_trailer(message, "git-gov-score", "0.85");
        assert!(new_message.contains("git-gov-score: 0.85"));
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
        let message = "Initial commit\ngit-gov-score: 0.85";
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let _commit = repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[]).unwrap();
         
        let commit = get_latest_commit(&repo).unwrap();
        let has_trailer = has_trailer(&commit, "git-gov-score").unwrap();
        assert!(has_trailer);
    }
}