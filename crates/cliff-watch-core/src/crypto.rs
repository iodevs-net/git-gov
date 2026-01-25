pub mod zkp;
pub mod tpm;
pub use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Genera un par de claves Ed25519
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Firma datos usando una clave privada Ed25519
pub fn sign_data(signing_key: &SigningKey, data: &[u8]) -> Result<Vec<u8>, String> {
    let signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Verifica una firma usando una clave pública Ed25519
pub fn verify_signature(verifying_key: &VerifyingKey, data: &[u8], signature: &[u8]) -> Result<bool, String> {
    if signature.len() != 64 {
        return Err("Invalid signature length".to_string());
    }
    let signature_bytes: [u8; 64] = signature.try_into().map_err(|_| "Invalid signature format".to_string())?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
    Ok(verifying_key.verify(data, &signature).is_ok())
}

/// Calcula el hash SHA256 de los datos
pub fn calculate_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Carga la identidad del daemon desde ~/.config/cliff-watch/daemon.key o crea una nueva
pub fn load_or_create_identity() -> Result<SigningKey, String> {
    let home = std::env::var("HOME").map_err(|_| "No env var HOME found")?;
    let config_dir = PathBuf::from(home).join(".config").join("cliff-watch");
    let key_path = config_dir.join("daemon.key");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    if key_path.exists() {
        let mut file = fs::File::open(&key_path).map_err(|e| format!("Failed to open key file: {}", e))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| format!("Failed to read key file: {}", e))?;
        
        if bytes.len() != 32 {
            return Err("Invalid key file size (expected 32 bytes)".to_string());
        }

        let key_bytes: [u8; 32] = bytes.try_into().map_err(|_| "Failed to parse key bytes")?;
        Ok(SigningKey::from_bytes(&key_bytes))
    } else {
        let (signing_key, _) = generate_keypair();
        let bytes = signing_key.to_bytes();
        
        let mut file = fs::File::create(&key_path).map_err(|e| format!("Failed to create key file: {}", e))?;
        
        #[cfg(unix)]
        {
            let mut perms = file.metadata().map_err(|e| e.to_string())?.permissions();
            perms.set_mode(0o600);
            file.set_permissions(perms).map_err(|e| e.to_string())?;
        }

        file.write_all(&bytes).map_err(|e| format!("Failed to write key file: {}", e))?;
        Ok(signing_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sign_verify() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"test data";
        let signature = sign_data(&signing_key, data).unwrap();
        let is_valid = verify_signature(&verifying_key, data, &signature).unwrap();
        assert!(is_valid, "Signature should be valid");
    }
    
    #[test]
    fn test_sha256() {
        let data = b"test";
        let hash = calculate_sha256(data);
        assert_eq!(hash.len(), 32, "SHA256 hash should be 32 bytes");
    }
}