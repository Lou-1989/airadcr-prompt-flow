// ============================================================================
// AIRADCR Desktop - Handlers HTTP
// ============================================================================

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{Utc, Duration};
use uuid::Uuid;

use super::HttpServerState;
use super::middleware::{validate_api_key, validate_admin_key, validate_patient_safe};

// ============================================================================
// Structures de données
// ============================================================================

#[derive(Deserialize)]
pub struct StorePendingReportRequest {
    pub technical_id: String,
    pub structured: Value,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    pub ai_modules: Option<Vec<String>>,
    #[serde(default = "default_expires_hours")]
    pub expires_in_hours: i64,
}

fn default_source_type() -> String {
    "tauri_local".to_string()
}

fn default_expires_hours() -> i64 {
    24
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct StoreSuccessResponse {
    pub success: bool,
    pub technical_id: String,
    pub retrieval_url: String,
    pub expires_at: String,
}

#[derive(Serialize)]
pub struct GetReportResponse {
    pub success: bool,
    pub data: Option<ReportData>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ReportData {
    pub structured: Value,
    pub source_type: String,
    pub ai_modules: Option<Vec<String>>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub deleted: bool,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Deserialize)]
pub struct TidQuery {
    pub tid: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[serde(default = "generate_random_key")]
    pub key: String,
}

fn generate_random_key() -> String {
    use rand::Rng;
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();
    format!("airadcr_{}", suffix.to_lowercase())
}

#[derive(Serialize)]
pub struct CreateApiKeyResponse {
    pub success: bool,
    pub id: String,
    pub key: String,
    pub name: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ListApiKeysResponse {
    pub success: bool,
    pub keys: Vec<ApiKeyInfo>,
}

#[derive(Serialize)]
pub struct RevokeApiKeyResponse {
    pub success: bool,
    pub revoked: bool,
    pub prefix: String,
}

// ============================================================================
// Validation des entrées
// ============================================================================

use regex::Regex;
use std::sync::OnceLock;

static TECHNICAL_ID_REGEX: OnceLock<Regex> = OnceLock::new();
static API_KEY_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_technical_id_regex() -> &'static Regex {
    TECHNICAL_ID_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap()
    })
}

fn get_api_key_name_regex() -> &'static Regex {
    API_KEY_NAME_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9\s\-_]{1,100}$").unwrap()
    })
}

fn validate_technical_id(tid: &str) -> Result<(), String> {
    if tid.is_empty() {
        return Err("technical_id cannot be empty".to_string());
    }
    if tid.len() > 64 {
        return Err("technical_id must be 64 characters or less".to_string());
    }
    if !get_technical_id_regex().is_match(tid) {
        return Err("technical_id must contain only alphanumeric characters, hyphens, and underscores".to_string());
    }
    Ok(())
}

fn validate_api_key_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("API key name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("API key name must be 100 characters or less".to_string());
    }
    if !get_api_key_name_regex().is_match(name) {
        return Err("API key name must contain only alphanumeric characters, spaces, hyphens, and underscores".to_string());
    }
    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /health - Vérification de l'état du serveur
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    })
}

/// POST /pending-report - Stocke un rapport en attente
pub async fn store_pending_report(
    req: HttpRequest,
    body: web::Json<StorePendingReportRequest>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // 1. Validation API Key
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_api_key(&state.db, api_key) {
        println!("❌ [HTTP] Clé API invalide");
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid API key".to_string(),
            field: None,
        });
    }
    
    // 2. Validation technical_id
    if let Err(msg) = validate_technical_id(&body.technical_id) {
        println!("❌ [HTTP] Invalid technical_id: {}", msg);
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: msg,
            field: Some("technical_id".to_string()),
        });
    }
    
    // 3. Validation Patient-Safe
    if let Err((field, message)) = validate_patient_safe(&body.structured) {
        println!("❌ [HTTP] Patient-Safe violation: {} ({})", message, field);
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Patient-Safe violation: {}", message),
            field: Some(field),
        });
    }
    
    // 4. Préparer les données
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::hours(body.expires_in_hours);
    
    let structured_json = serde_json::to_string(&body.structured).unwrap_or_default();
    let ai_modules_json = body.ai_modules
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());
    
    // 5. Insérer en base
    match state.db.insert_pending_report(
        &id,
        &body.technical_id,
        &structured_json,
        &body.source_type,
        ai_modules_json.as_deref(),
        &now.to_rfc3339(),
        &expires_at.to_rfc3339(),
    ) {
        Ok(_) => {
            println!("✅ [HTTP] Rapport stocké: tid={}", body.technical_id);
            HttpResponse::Ok().json(StoreSuccessResponse {
                success: true,
                technical_id: body.technical_id.clone(),
                retrieval_url: format!("https://airadcr.com/app?tid={}", body.technical_id),
                expires_at: expires_at.to_rfc3339(),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur insertion: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// GET /pending-report?tid=XXX - Récupère un rapport en attente
pub async fn get_pending_report(
    query: web::Query<TidQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let tid = match &query.tid {
        Some(tid) if !tid.is_empty() => tid,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing 'tid' parameter".to_string(),
                field: None,
            });
        }
    };
    
    match state.db.get_pending_report(tid) {
        Ok(Some(report)) => {
            // Parser le JSON stocké
            let structured: Value = serde_json::from_str(&report.structured_data)
                .unwrap_or(Value::Null);
            let ai_modules: Option<Vec<String>> = report.ai_modules
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok());
            
            // Marquer comme récupéré
            let _ = state.db.mark_as_retrieved(tid);
            
            println!("✅ [HTTP] Rapport récupéré: tid={}", tid);
            HttpResponse::Ok().json(GetReportResponse {
                success: true,
                data: Some(ReportData {
                    structured,
                    source_type: report.source_type,
                    ai_modules,
                    created_at: report.created_at,
                }),
                error: None,
            })
        }
        Ok(None) => {
            println!("⚠️ [HTTP] Rapport non trouvé: tid={}", tid);
            HttpResponse::NotFound().json(GetReportResponse {
                success: false,
                data: None,
                error: Some("Report not found or expired".to_string()),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur lecture: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// DELETE /pending-report?tid=XXX - Supprime un rapport
pub async fn delete_pending_report(
    query: web::Query<TidQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let tid = match &query.tid {
        Some(tid) if !tid.is_empty() => tid,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing 'tid' parameter".to_string(),
                field: None,
            });
        }
    };
    
    match state.db.delete_pending_report(tid) {
        Ok(deleted) => {
            println!("🗑️ [HTTP] Rapport supprimé: tid={} (deleted={})", tid, deleted);
            HttpResponse::Ok().json(DeleteResponse {
                success: true,
                deleted,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur suppression: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// POST /api-keys - Crée une nouvelle clé API (requiert authentification admin)
pub async fn create_api_key(
    req: HttpRequest,
    body: web::Json<CreateApiKeyRequest>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // 1. Validation clé Admin (X-Admin-Key header)
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        println!("❌ [HTTP] Clé admin invalide pour création API key");
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or missing admin key".to_string(),
            field: None,
        });
    }
    
    // 2. Validation du nom avec regex
    if let Err(msg) = validate_api_key_name(&body.name) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: msg,
            field: Some("name".to_string()),
        });
    }
    
    // 3. Générer ou utiliser la clé fournie
    let api_key = if body.key.is_empty() {
        generate_random_key()
    } else {
        body.key.clone()
    };
    
    // 4. Calculer le hash SHA-256
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let key_hash = hex::encode(hasher.finalize());
    
    // 5. Préfixe pour recherche rapide
    let key_prefix = if api_key.len() >= 8 {
        &api_key[..8]
    } else {
        &api_key
    };
    
    // 6. Générer un ID unique
    let id = Uuid::new_v4().to_string();
    
    // 7. Insérer en base
    match state.db.add_api_key(&id, key_prefix, &key_hash, &body.name) {
        Ok(_) => {
            println!("✅ [HTTP] Nouvelle clé API créée: name={}, prefix={}", body.name, key_prefix);
            HttpResponse::Created().json(CreateApiKeyResponse {
                success: true,
                id,
                key: api_key,
                name: body.name.clone(),
                message: "API key created successfully. Store this key securely - it won't be shown again.".to_string(),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur création clé API: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// GET /api-keys - Liste toutes les clés API (requiert authentification admin)
pub async fn list_api_keys(
    req: HttpRequest,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // Validation clé Admin
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        println!("❌ [HTTP] Clé admin invalide pour liste API keys");
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or missing admin key".to_string(),
            field: None,
        });
    }
    
    match state.db.list_api_keys() {
        Ok(keys) => {
            let api_keys: Vec<ApiKeyInfo> = keys.into_iter().map(|(id, name, prefix, is_active, created_at)| {
                ApiKeyInfo { id, name, prefix, is_active, created_at }
            }).collect();
            
            println!("✅ [HTTP] Liste API keys: {} clé(s)", api_keys.len());
            HttpResponse::Ok().json(ListApiKeysResponse {
                success: true,
                keys: api_keys,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur liste API keys: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// DELETE /api-keys/{prefix} - Révoque une clé API (soft-delete)
pub async fn revoke_api_key(
    req: HttpRequest,
    path: web::Path<String>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // Validation clé Admin
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        println!("❌ [HTTP] Clé admin invalide pour révocation API key");
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or missing admin key".to_string(),
            field: None,
        });
    }
    
    let prefix = path.into_inner();
    
    if prefix.is_empty() || prefix.len() > 16 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid API key prefix".to_string(),
            field: Some("prefix".to_string()),
        });
    }
    
    match state.db.revoke_api_key(&prefix) {
        Ok(revoked) => {
            if revoked {
                println!("✅ [HTTP] Clé API révoquée: prefix={}", prefix);
            } else {
                println!("⚠️ [HTTP] Clé API non trouvée: prefix={}", prefix);
            }
            HttpResponse::Ok().json(RevokeApiKeyResponse {
                success: true,
                revoked,
                prefix,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur révocation API key: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}
