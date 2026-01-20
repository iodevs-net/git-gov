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

    // 1. Hook de preparación de mensaje (para añadir trailers)
    let prepare_hook_path = hooks_dir.join("prepare-commit-msg");
    let prepare_hook_content = r#"#!/bin/bash
# git-gov hook: Añade el score al mensaje del commit
SCORE=$(git-gov metrics --short 2>/dev/null)
if [ $? -eq 0 ] && [ ! -z "$SCORE" ]; then
    git interpret-trailers --in-place --trailer "git-gov-score: $SCORE" "$1"
fi
"#;
    std::fs::write(&prepare_hook_path, prepare_hook_content).map_err(|e| e.to_string())?;

    // 2. Hook de pre-commit (La "Aduana Termodinámica")
    let pre_hook_path = hooks_dir.join("pre-commit");
    let pre_hook_content = r#"#!/bin/bash
# git-gov hook: Aduana Termodinámica
# Bloquea el commit si no hay suficiente energía acumulada.

# Calculamos el costo del diff staged
DIFF_OUT=$(git diff --cached)
if [ -z "$DIFF_OUT" ]; then
    exit 0
fi

# El CLI se encarga de calcular el costo entrópico real y pedir el ticket
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