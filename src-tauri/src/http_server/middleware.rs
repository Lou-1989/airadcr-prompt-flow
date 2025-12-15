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
