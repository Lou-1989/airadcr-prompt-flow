// ============================================================================
// AIRADCR Desktop - Middleware HTTP (Validation & Sécurité)
// ============================================================================

use serde_json::Value;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use crate::database::Database;

// ============================================================================
// Patterns interdits pour la validation Patient-Safe
// ============================================================================

/// Liste des champs/patterns interdits (données nominatives)
const FORBIDDEN_PATTERNS: &[&str] = &[
    // Identifiants patient
    "patient_id", "patientid", "patient_name", "patientname",
    "patient_identifier", "patientidentifier",
    
    // Données démographiques
    "birth_date", "birthdate", "date_of_birth", "dob", "date_naissance",
    "first_name", "last_name", "firstname", "lastname",
    "nom", "prenom", "nom_patient", "prenom_patient",
    "full_name", "fullname", "nom_complet",
    
    // Identifiants administratifs
    "ssn", "social_security", "social_security_number",
    "insurance_number", "numero_securite_sociale", "nss",
    "ipp", "nis", "nh", "numero_hopital",
    
    // Coordonnées
    "address", "adresse", "street", "rue", "city", "ville",
    "postal_code", "code_postal", "zip", "zipcode",
    "phone", "telephone", "mobile", "email", "mail",
    
    // Identifiants DICOM/études
    "study_uid", "studyuid", "study_instance_uid",
    "series_uid", "seriesuid", "sop_instance_uid",
    "accession_number", "accessionnumber", "accession",
    
    // Autres identifiants sensibles
    "mrn", "medical_record_number", "numero_dossier",
    "encounter_id", "visit_id", "sejour_id",
];

// ============================================================================
// Validation Patient-Safe
// ============================================================================

/// Valide qu'un payload JSON ne contient pas de données nominatives
/// Retourne Ok(()) si valide, Err((field, message)) si violation
pub fn validate_patient_safe(json: &Value) -> Result<(), (String, String)> {
    validate_json_recursive(json, "")
}

fn validate_json_recursive(json: &Value, path: &str) -> Result<(), (String, String)> {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let key_lower = key.to_lowercase();
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                
                // Vérifier si la clé correspond à un pattern interdit
                for pattern in FORBIDDEN_PATTERNS {
                    if key_lower.contains(pattern) {
                        return Err((
                            current_path,
                            format!("nominative data detected (key contains '{}')", pattern),
                        ));
                    }
                }
                
                // Récursion sur les valeurs
                validate_json_recursive(value, &current_path)?;
            }
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                let current_path = format!("{}[{}]", path, i);
                validate_json_recursive(item, &current_path)?;
            }
        }
        // Les valeurs primitives sont acceptées
        _ => {}
    }
    
    Ok(())
}

// ============================================================================
// Validation API Key
// ============================================================================

/// Valide une clé API contre la base de données
pub fn validate_api_key(db: &Arc<Database>, api_key: &str) -> bool {
    if api_key.is_empty() {
        return false;
    }
    
    // Calcul du hash SHA-256
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = hex::encode(hasher.finalize());
    
    // Préfixe pour recherche rapide (8 premiers caractères)
    let key_prefix = if api_key.len() >= 8 {
        &api_key[..8]
    } else {
        api_key
    };
    
    // Vérification en base
    match db.validate_api_key(key_prefix, &key_hash) {
        Ok(valid) => valid,
        Err(e) => {
            eprintln!("❌ [Middleware] Erreur validation API key: {}", e);
            false
        }
    }
}

// ============================================================================
// Validation Admin Key (Sécurisée via ENV ou fichier)
// ============================================================================

use std::sync::OnceLock;
use std::fs;

/// Cache pour la clé admin hashée (initialisée une seule fois)
static ADMIN_KEY_HASH: OnceLock<[u8; 32]> = OnceLock::new();

/// Récupère le hash de la clé admin depuis ENV ou fichier
fn get_admin_key_hash() -> &'static [u8; 32] {
    ADMIN_KEY_HASH.get_or_init(|| {
        // 1. Essayer la variable d'environnement AIRADCR_ADMIN_KEY
        if let Ok(key) = std::env::var("AIRADCR_ADMIN_KEY") {
            if !key.is_empty() {
                println!("🔐 [Security] Clé admin chargée depuis AIRADCR_ADMIN_KEY");
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                return hasher.finalize().into();
            }
        }
        
        // 2. Essayer le fichier ~/.airadcr/admin.key
        if let Some(home) = dirs::home_dir() {
            let key_path = home.join(".airadcr").join("admin.key");
            if let Ok(key) = fs::read_to_string(&key_path) {
                let key = key.trim();
                if !key.is_empty() {
                    println!("🔐 [Security] Clé admin chargée depuis {:?}", key_path);
                    let mut hasher = Sha256::new();
                    hasher.update(key.as_bytes());
                    return hasher.finalize().into();
                }
            }
        }
        
        // 3. Fallback: clé par défaut (seulement pour dev, affiché avec warning)
        eprintln!("⚠️ [Security] ATTENTION: Utilisation de la clé admin par défaut!");
        eprintln!("⚠️ [Security] Définissez AIRADCR_ADMIN_KEY ou créez ~/.airadcr/admin.key");
        let default_key = "airadcr_admin_master_9x7w5v3t1r8p6n4m2k0j";
        let mut hasher = Sha256::new();
        hasher.update(default_key.as_bytes());
        hasher.finalize().into()
    })
}

/// Valide une clé admin avec comparaison en temps constant
pub fn validate_admin_key(admin_key: &str) -> bool {
    if admin_key.is_empty() {
        return false;
    }
    
    // Hash de la clé fournie
    let mut hasher = Sha256::new();
    hasher.update(admin_key.as_bytes());
    let provided_hash: [u8; 32] = hasher.finalize().into();
    
    // Comparaison en temps constant
    let expected_hash = get_admin_key_hash();
    constant_time_compare(&provided_hash, expected_hash)
}

/// Comparaison en temps constant pour éviter les timing attacks
fn constant_time_compare(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ============================================================================
// Tests unitaires
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_valid_payload() {
        let payload = json!({
            "title": "IRM Cérébrale",
            "indication": "Céphalées chroniques",
            "technique": "Séquences T1, T2, FLAIR",
            "results": "Pas d'anomalie",
            "conclusion": "Examen normal"
        });
        
        assert!(validate_patient_safe(&payload).is_ok());
    }
    
    #[test]
    fn test_forbidden_patient_id() {
        let payload = json!({
            "title": "IRM",
            "patient_id": "12345"
        });
        
        let result = validate_patient_safe(&payload);
        assert!(result.is_err());
        let (field, _) = result.unwrap_err();
        assert_eq!(field, "patient_id");
    }
    
    #[test]
    fn test_forbidden_nested() {
        let payload = json!({
            "title": "IRM",
            "metadata": {
                "patient_name": "John Doe"
            }
        });
        
        let result = validate_patient_safe(&payload);
        assert!(result.is_err());
        let (field, _) = result.unwrap_err();
        assert_eq!(field, "metadata.patient_name");
    }
    
    #[test]
    fn test_forbidden_in_array() {
        let payload = json!({
            "reports": [
                { "title": "IRM" },
                { "birth_date": "1990-01-01" }
            ]
        });
        
        let result = validate_patient_safe(&payload);
        assert!(result.is_err());
    }
}
