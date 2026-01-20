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

    let hook_path = hooks_dir.join("prepare-commit-msg");
    let hook_content = r#"#!/bin/bash
# git-gov hook
# Captura métricas y añade el trailer de score al commit

# Intentar obtener métricas del daemon de git-gov
SCORE=$(git-gov metrics --short 2>/dev/null)

if [ $? -eq 0 ] && [ ! -z "$SCORE" ]; then
    # Añadir el trailer al mensaje del commit
    git interpret-trailers --in-place --trailer "git-gov-score: $SCORE" "$1"
fi
"#;

    std::fs::write(&hook_path, hook_content).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms).map_err(|e| e.to_string())?;
    }

    Ok(())
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