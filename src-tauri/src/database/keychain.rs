// ============================================================================
// AIRADCR Desktop - Gestion des clés de chiffrement via Keychain OS
// ============================================================================
// Stocke la clé de chiffrement SQLCipher dans le keychain natif de l'OS :
// - Windows : Credential Manager
// - macOS   : Keychain
// - Linux   : Secret Service (GNOME Keyring / KWallet)
// ============================================================================

use log::{info, warn};
use rand::Rng;

const SERVICE_NAME: &str = "airadcr-desktop";
const DB_KEY_ENTRY: &str = "sqlcipher-encryption-key";
const TEO_TOKEN_ENTRY: &str = "teo-hub-api-token";

/// Génère une clé de chiffrement aléatoire de 64 caractères hex (256 bits)
fn generate_encryption_key() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen::<u8>()).collect();
    hex::encode(bytes)
}

/// Récupère ou crée la clé de chiffrement SQLCipher depuis le keychain OS
pub fn get_or_create_db_encryption_key() -> Result<String, String> {
    match get_keychain_value(DB_KEY_ENTRY) {
        Ok(key) if !key.is_empty() => {
            info!("[Keychain] Clé de chiffrement SQLCipher récupérée depuis le keychain OS");
            Ok(key)
        }
        _ => {
            // Première utilisation : générer et stocker la clé
            let new_key = generate_encryption_key();
            set_keychain_value(DB_KEY_ENTRY, &new_key)?;
            info!("[Keychain] Nouvelle clé de chiffrement SQLCipher créée et stockée dans le keychain OS");
            Ok(new_key)
        }
    }
}

/// Stocke le token TEO Hub dans le keychain OS
pub fn store_teo_token(token: &str) -> Result<(), String> {
    if token.is_empty() {
        // Ne pas stocker de token vide
        return Ok(());
    }
    set_keychain_value(TEO_TOKEN_ENTRY, token)?;
    info!("[Keychain] Token TEO Hub stocké dans le keychain OS");
    Ok(())
}

/// Récupère le token TEO Hub depuis le keychain OS
pub fn get_teo_token() -> Result<Option<String>, String> {
    match get_keychain_value(TEO_TOKEN_ENTRY) {
        Ok(token) if !token.is_empty() => Ok(Some(token)),
        Ok(_) => Ok(None),
        Err(e) => {
            warn!("[Keychain] Token TEO Hub non trouvé dans le keychain: {}", e);
            Ok(None)
        }
    }
}

/// Supprime le token TEO Hub du keychain OS
#[allow(dead_code)]
pub fn delete_teo_token() -> Result<(), String> {
    delete_keychain_value(TEO_TOKEN_ENTRY)
}

// ============================================================================
// Opérations bas niveau sur le keychain
// ============================================================================

fn get_keychain_value(entry_name: &str) -> Result<String, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, entry_name)
        .map_err(|e| format!("Erreur création entrée keychain: {}", e))?;
    
    entry.get_password()
        .map_err(|e| format!("Erreur lecture keychain '{}': {}", entry_name, e))
}

fn set_keychain_value(entry_name: &str, value: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, entry_name)
        .map_err(|e| format!("Erreur création entrée keychain: {}", e))?;
    
    entry.set_password(value)
        .map_err(|e| format!("Erreur écriture keychain '{}': {}", entry_name, e))
}

#[allow(dead_code)]
fn delete_keychain_value(entry_name: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, entry_name)
        .map_err(|e| format!("Erreur création entrée keychain: {}", e))?;
    
    match entry.delete_password() {
        Ok(_) => {
            info!("[Keychain] Entrée '{}' supprimée", entry_name);
            Ok(())
        }
        Err(e) => {
            warn!("[Keychain] Suppression '{}' échouée (peut-être absente): {}", entry_name, e);
            Ok(()) // Ne pas échouer si l'entrée n'existe pas
        }
    }
}
