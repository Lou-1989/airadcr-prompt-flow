// ============================================================================
// AIRADCR Desktop - Handlers HTTP
// ============================================================================

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{Utc, Duration};
use uuid::Uuid;
use tauri::Manager;

use super::HttpServerState;
use super::middleware::{validate_api_key, validate_admin_key, RequestInfo};
use crate::APP_HANDLE;
use crate::teo_client;
use crate::config::get_config;

// ============================================================================
// Fonctions utilitaires de sécurité
// ============================================================================

/// ⚠️ Vérifie si l'authentification API est désactivée (mode demo/test)
fn is_auth_disabled() -> bool {
    let disabled = get_config().disable_api_auth;
    if disabled {
        log::warn!("⚠️ [SECURITY] API authentication DISABLED - demo/test mode!");
    }
    disabled
}

/// 🛡️ Masque un identifiant sensible pour les logs (affiche seulement les 4 premiers caractères)
fn mask_sensitive_id(id: &str) -> String {
    if id.len() <= 4 {
        "****".to_string()
    } else {
        format!("{}****", &id[..4])
    }
}

// ============================================================================
// Structures de données
// ============================================================================

#[derive(Deserialize)]
pub struct StorePendingReportRequest {
    pub technical_id: String,
    // Identifiants patients - ACCEPTÉS EN LOCAL (données ne quittent pas la machine)
    pub patient_id: Option<String>,
    pub exam_uid: Option<String>,
    pub accession_number: Option<String>,
    pub study_instance_uid: Option<String>,
    // Données structurées
    pub structured: Value,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    pub ai_modules: Option<Vec<String>>,
    pub modality: Option<String>,
    pub metadata: Option<Value>,
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
    // 🆕 Version dynamique depuis Cargo.toml (Phase 1)
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
    pub technical_id: String,
    // Identifiants patients (LOCAL UNIQUEMENT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exam_uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accession_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_instance_uid: Option<String>,
    // Données structurées
    pub structured: Value,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_modules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    pub status: String,
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
pub struct FindReportQuery {
    pub patient_id: Option<String>,
    pub accession_number: Option<String>,
    pub exam_uid: Option<String>,
}

#[derive(Serialize)]
pub struct FindReportResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ReportData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenReportQuery {
    pub tid: Option<String>,
    pub accession_number: Option<String>,
    pub patient_id: Option<String>,
    pub exam_uid: Option<String>,
    pub study_uid: Option<String>,
}

#[derive(Serialize)]
pub struct OpenReportResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigated_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Deserialize)]
pub struct TeoHubFetchQuery {
    pub patient_id: Option<String>,
    pub study_uid: Option<String>,
    pub exam_uid: Option<String>,
}

#[derive(Serialize)]
pub struct TeoHubFetchResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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

/// GET /health - Vérification de l'état du serveur (minimal, sans info sensible)
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: "".to_string(), // 🔒 SÉCURITÉ: Ne pas exposer la version sans auth
    })
}

/// POST /pending-report - Stocke un rapport en attente (avec identifiants patients LOCAL)
pub async fn store_pending_report(
    req: HttpRequest,
    body: web::Json<StorePendingReportRequest>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    // 1. Validation API Key
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !is_auth_disabled() && !validate_api_key(&state.db, api_key) {
        log::warn!("❌ [HTTP] Clé API invalide");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid API key"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid API key".to_string(),
            field: None,
        });
    }
    
    // 2. Validation technical_id
    if let Err(msg) = validate_technical_id(&body.technical_id) {
        log::warn!("❌ [HTTP] Invalid technical_id: {}", msg);
        request_info.log_access(&state.db, 400, "bad_request", Some(&msg));
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: msg,
            field: Some("technical_id".to_string()),
        });
    }
    
    // 3. NOTE: Patient-Safe validation désactivée pour le serveur LOCAL
    // En local, les identifiants patients sont autorisés car les données
    // ne quittent jamais la machine de l'utilisateur.
    // La validation Patient-Safe reste active pour le cloud (Supabase).
    
    // 4. Préparer les données
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::hours(body.expires_in_hours);
    
    let structured_json = serde_json::to_string(&body.structured).unwrap_or_default();
    let ai_modules_json = body.ai_modules
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());
    let metadata_json = body.metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());
    
    // 5. Insérer en base avec identifiants patients
    match state.db.insert_pending_report(
        &id,
        &body.technical_id,
        body.patient_id.as_deref(),
        body.exam_uid.as_deref(),
        body.accession_number.as_deref(),
        body.study_instance_uid.as_deref(),
        &structured_json,
        &body.source_type,
        ai_modules_json.as_deref(),
        body.modality.as_deref(),
        metadata_json.as_deref(),
        &now.to_rfc3339(),
        &expires_at.to_rfc3339(),
    ) {
        Ok(_) => {
            // 🛡️ SÉCURITÉ: Masquer les identifiants sensibles dans les logs
            let masked_patient_id = body.patient_id.as_ref().map(|id| mask_sensitive_id(id));
            log::info!("✅ [HTTP] Rapport stocké: tid={}, patient_id={:?}",
                     body.technical_id, masked_patient_id);
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(StoreSuccessResponse {
                success: true,
                technical_id: body.technical_id.clone(),
                retrieval_url: format!("https://airadcr.com/app?tori=true&tid={}", body.technical_id),
                expires_at: expires_at.to_rfc3339(),
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur insertion: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// GET /pending-report?tid=XXX - Récupère un rapport en attente (avec identifiants patients)
pub async fn get_pending_report(
    req: HttpRequest,
    query: web::Query<TidQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    let tid = match &query.tid {
        Some(tid) if !tid.is_empty() => tid,
        _ => {
            request_info.log_access(&state.db, 400, "bad_request", Some("Missing 'tid' parameter"));
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
            let metadata: Option<Value> = report.metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok());
            
            // Marquer comme récupéré
            let _ = state.db.mark_as_retrieved(tid);
            
            // 🛡️ SÉCURITÉ: Masquer les identifiants sensibles dans les logs
            let masked_patient_id = report.patient_id.as_ref().map(|id| mask_sensitive_id(id));
            log::info!("✅ [HTTP] Rapport récupéré: tid={}, patient_id={:?}", tid, masked_patient_id);
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(GetReportResponse {
                success: true,
                data: Some(ReportData {
                    technical_id: report.technical_id,
                    patient_id: report.patient_id,
                    exam_uid: report.exam_uid,
                    accession_number: report.accession_number,
                    study_instance_uid: report.study_instance_uid,
                    structured,
                    source_type: report.source_type,
                    ai_modules,
                    modality: report.modality,
                    metadata,
                    status: report.status,
                    created_at: report.created_at,
                }),
                error: None,
            })
        }
        Ok(None) => {
            log::warn!("⚠️ [HTTP] Rapport non trouvé: tid={}", tid);
            request_info.log_access(&state.db, 404, "not_found", Some("Report not found or expired"));
            HttpResponse::NotFound().json(GetReportResponse {
                success: false,
                data: None,
                error: Some("Report not found or expired".to_string()),
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur lecture: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// DELETE /pending-report?tid=XXX - Supprime un rapport (🔒 requiert API key)
pub async fn delete_pending_report(
    req: HttpRequest,
    query: web::Query<TidQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    // 🔒 SÉCURITÉ: Exiger une clé API pour les suppressions
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !is_auth_disabled() && !validate_api_key(&state.db, api_key) {
        log::warn!("❌ [HTTP] DELETE sans API key valide");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid API key for DELETE"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "API key required for DELETE operations".to_string(),
            field: None,
        });
    }
    
    let tid = match &query.tid {
        Some(tid) if !tid.is_empty() => tid,
        _ => {
            request_info.log_access(&state.db, 400, "bad_request", Some("Missing 'tid' parameter"));
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing 'tid' parameter".to_string(),
                field: None,
            });
        }
    };
    
    match state.db.delete_pending_report(tid) {
        Ok(deleted) => {
            log::info!("🗑️ [HTTP] Rapport supprimé: tid={} (deleted={})", tid, deleted);
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(DeleteResponse {
                success: true,
                deleted,
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur suppression: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
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
    let request_info = RequestInfo::from_request(&req);
    
    // 1. Validation clé Admin (X-Admin-Key header)
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        log::warn!("❌ [HTTP] Clé admin invalide pour création API key");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid admin key"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or missing admin key".to_string(),
            field: None,
        });
    }
    
    // 2. Validation du nom avec regex
    if let Err(msg) = validate_api_key_name(&body.name) {
        request_info.log_access(&state.db, 400, "bad_request", Some(&msg));
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
            log::info!("✅ [HTTP] Nouvelle clé API créée: name={}, prefix={}", body.name, key_prefix);
            request_info.log_access(&state.db, 201, "success", None);
            HttpResponse::Created().json(CreateApiKeyResponse {
                success: true,
                id,
                key: api_key,
                name: body.name.clone(),
                message: "API key created successfully. Store this key securely - it won't be shown again.".to_string(),
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur création clé API: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
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
    let request_info = RequestInfo::from_request(&req);
    
    // Validation clé Admin
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        log::warn!("❌ [HTTP] Clé admin invalide pour liste API keys");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid admin key"));
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
            
            log::info!("✅ [HTTP] Liste API keys: {} clé(s)", api_keys.len());
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(ListApiKeysResponse {
                success: true,
                keys: api_keys,
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur liste API keys: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
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
    let request_info = RequestInfo::from_request(&req);
    
    // Validation clé Admin
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !validate_admin_key(admin_key) {
        log::warn!("❌ [HTTP] Clé admin invalide pour révocation API key");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid admin key"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or missing admin key".to_string(),
            field: None,
        });
    }
    
    let prefix = path.into_inner();
    
    if prefix.is_empty() || prefix.len() > 16 {
        request_info.log_access(&state.db, 400, "bad_request", Some("Invalid API key prefix"));
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid API key prefix".to_string(),
            field: Some("prefix".to_string()),
        });
    }
    
    match state.db.revoke_api_key(&prefix) {
        Ok(revoked) => {
            if revoked {
                log::info!("✅ [HTTP] Clé API révoquée: prefix={}", prefix);
            } else {
                log::warn!("⚠️ [HTTP] Clé API non trouvée: prefix={}", prefix);
            }
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(RevokeApiKeyResponse {
                success: true,
                revoked,
                prefix,
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur révocation API key: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// GET /find-report - Recherche un rapport par identifiants RIS (patient_id, accession_number, exam_uid)
pub async fn find_report(
    req: HttpRequest,
    query: web::Query<FindReportQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    // Vérifier qu'au moins un identifiant est fourni
    let has_patient_id = query.patient_id.as_ref().map_or(false, |s| !s.is_empty());
    let has_accession = query.accession_number.as_ref().map_or(false, |s| !s.is_empty());
    let has_exam_uid = query.exam_uid.as_ref().map_or(false, |s| !s.is_empty());
    
    if !has_patient_id && !has_accession && !has_exam_uid {
        request_info.log_access(&state.db, 400, "bad_request", Some("At least one identifier required"));
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "At least one identifier required: patient_id, accession_number, or exam_uid".to_string(),
            field: None,
        });
    }
    
    log::info!("🔍 [HTTP] Recherche rapport: patient_id={:?}, accession={:?}, exam_uid={:?}",
             query.patient_id, query.accession_number, query.exam_uid);
    
    match state.db.find_pending_report_by_identifiers(
        query.patient_id.as_deref(),
        query.accession_number.as_deref(),
        query.exam_uid.as_deref(),
    ) {
        Ok(Some(report)) => {
            // Parser le JSON stocké
            let structured: Value = serde_json::from_str(&report.structured_data)
                .unwrap_or(Value::Null);
            let ai_modules: Option<Vec<String>> = report.ai_modules
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok());
            let metadata: Option<Value> = report.metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok());
            
            log::info!("✅ [HTTP] Rapport trouvé: tid={}", report.technical_id);
            let tid = report.technical_id.clone();
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(FindReportResponse {
                success: true,
                data: Some(ReportData {
                    technical_id: report.technical_id,
                    patient_id: report.patient_id,
                    exam_uid: report.exam_uid,
                    accession_number: report.accession_number,
                    study_instance_uid: report.study_instance_uid,
                    structured,
                    source_type: report.source_type,
                    ai_modules,
                    modality: report.modality,
                    metadata,
                    status: report.status,
                    created_at: report.created_at,
                }),
                retrieval_url: Some(format!("http://localhost:8741/pending-report?tid={}", tid)),
                error: None,
            })
        }
        Ok(None) => {
            log::warn!("⚠️ [HTTP] Aucun rapport trouvé pour ces identifiants");
            request_info.log_access(&state.db, 404, "not_found", Some("No report found"));
            HttpResponse::NotFound().json(FindReportResponse {
                success: false,
                data: None,
                retrieval_url: None,
                error: Some("No report found matching the provided identifiers".to_string()),
            })
        }
        Err(e) => {
            log::error!("❌ [HTTP] Erreur recherche: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// POST /open-report - Ouvre un rapport dans l'iframe AIRADCR (navigation depuis RIS)
/// 🔒 Requiert une clé API valide
/// Recherche par tid OU par identifiants RIS (accession_number, patient_id, exam_uid)
pub async fn open_report(
    req: HttpRequest,
    query: web::Query<OpenReportQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    // 🔒 SÉCURITÉ: Exiger une clé API pour la navigation
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !is_auth_disabled() && !validate_api_key(&state.db, api_key) {
        log::warn!("❌ [HTTP] POST /open-report sans API key valide");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid API key for open-report"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "API key required for open-report".to_string(),
            field: None,
        });
    }
    
    // Déterminer le technical_id à utiliser
    let technical_id: Option<String> = if let Some(tid) = &query.tid {
        if !tid.is_empty() {
            Some(tid.clone())
        } else {
            None
        }
    } else {
        // Rechercher par identifiants RIS
        let has_accession = query.accession_number.as_ref().map_or(false, |s| !s.is_empty());
        let has_patient = query.patient_id.as_ref().map_or(false, |s| !s.is_empty());
        let has_exam = query.exam_uid.as_ref().map_or(false, |s| !s.is_empty())
            || query.study_uid.as_ref().map_or(false, |s| !s.is_empty());
        
        // Résoudre exam_uid ou study_uid (study_uid est un alias DICOM standard)
        let resolved_exam_uid = query.exam_uid.as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| query.study_uid.as_deref().filter(|s| !s.is_empty()));
        
        if has_accession || has_patient || has_exam {
            match state.db.find_pending_report_by_identifiers(
                query.patient_id.as_deref(),
                query.accession_number.as_deref(),
                resolved_exam_uid,
            ) {
                Ok(Some(report)) => Some(report.technical_id),
                Ok(None) => {
                    // 🆕 Fallback: tenter de récupérer depuis TÉO Hub
                    let config = get_config();
                    if config.teo_hub.enabled && has_patient && has_exam {
                        let patient_id_val = query.patient_id.as_deref().unwrap_or("");
                        let exam_uid_val = resolved_exam_uid.unwrap_or("");
                        
                        log::info!("🔄 [HTTP] Fallback TÉO Hub: patient_id={}, exam_uid={}...",
                            mask_sensitive_id(patient_id_val),
                            &exam_uid_val[..exam_uid_val.len().min(20)]);
                        
                        match teo_client::fetch_ai_report(patient_id_val, exam_uid_val).await {
                            Ok(teo_report) => {
                                match store_teo_report_locally(
                                    &state,
                                    &teo_report,
                                    query.patient_id.as_deref(),
                                    query.exam_uid.as_deref(),
                                    query.accession_number.as_deref(),
                                ) {
                                    Some(tid) => {
                                        log::info!("✅ [HTTP] Rapport TÉO Hub stocké localement: tid={}", tid);
                                        Some(tid)
                                    }
                                    None => None,
                                }
                            }
                            Err(e) => {
                                log::warn!("⚠️ [HTTP] Fallback TÉO Hub échoué: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::error!("❌ [HTTP] Erreur recherche pour open-report: {}", e);
                    request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Database error: {}", e),
                        field: None,
                    });
                }
            }
        } else {
            None
        }
    };
    
    // Vérifier qu'on a un technical_id
    let tid = match technical_id {
        Some(tid) => tid,
        None => {
            request_info.log_access(&state.db, 400, "bad_request", Some("No identifier provided or report not found"));
            return HttpResponse::BadRequest().json(OpenReportResponse {
                success: false,
                message: None,
                technical_id: None,
                navigated_to: None,
                error: Some("At least one identifier required: tid, accession_number, patient_id, or exam_uid".to_string()),
                source: None,
            });
        }
    };
    
    // 🔒 SÉCURITÉ: Valider le TID avant émission dans l'événement Tauri
    if let Err(msg) = validate_technical_id(&tid) {
        log::warn!("❌ [HTTP] TID invalide dans open-report: {}", msg);
        request_info.log_access(&state.db, 400, "bad_request", Some(&msg));
        return HttpResponse::BadRequest().json(OpenReportResponse {
            success: false,
            message: None,
            technical_id: Some(tid),
            navigated_to: None,
            error: Some(format!("Invalid technical_id: {}", msg)),
            source: None,
        });
    }
    
    // Émettre l'événement Tauri pour naviguer vers le rapport
    if let Some(app_handle) = APP_HANDLE.get() {
        if let Some(window) = app_handle.get_window("main") {
            // Émettre l'événement avec le tid
            match window.emit("airadcr:navigate_to_report", &tid) {
                Ok(_) => {
                    log::info!("✅ [HTTP] Navigation émise: tid={}", tid);
                    
                    // Afficher et focus la fenêtre
                    let _ = window.show();
                    let _ = window.set_focus();
                    
                    request_info.log_access(&state.db, 200, "success", None);
                    HttpResponse::Ok().json(OpenReportResponse {
                        success: true,
                        message: Some("Navigation triggered successfully".to_string()),
                        technical_id: Some(tid.clone()),
                        navigated_to: Some(format!("https://airadcr.com/app?tori=true&tid={}", tid)),
                        error: None,
                        source: Some("local".to_string()),
                    })
                }
                Err(e) => {
                    log::error!("❌ [HTTP] Erreur émission événement: {}", e);
                    request_info.log_access(&state.db, 500, "error", Some(&format!("Event emit error: {}", e)));
                    HttpResponse::InternalServerError().json(OpenReportResponse {
                        success: false,
                        message: None,
                        technical_id: Some(tid),
                        navigated_to: None,
                        error: Some(format!("Failed to trigger navigation event: {}", e)),
                        source: None,
                    })
                }
            }
        } else {
            log::error!("❌ [HTTP] Fenêtre main non trouvée");
            request_info.log_access(&state.db, 500, "error", Some("Main window not found"));
            HttpResponse::InternalServerError().json(OpenReportResponse {
                success: false,
                message: None,
                technical_id: Some(tid),
                navigated_to: None,
                error: Some("Main window not found".to_string()),
                source: None,
            })
        }
    } else {
        log::error!("❌ [HTTP] AppHandle non disponible (app pas encore démarrée)");
        request_info.log_access(&state.db, 503, "error", Some("Application not yet ready"));
        HttpResponse::ServiceUnavailable()
            .insert_header(("Retry-After", "2"))
            .json(OpenReportResponse {
                success: false,
                message: None,
                technical_id: Some(tid),
                navigated_to: None,
                error: Some("Application not yet ready. Please try again in a moment.".to_string()),
                source: None,
            })
    }
}

// ============================================================================
// Helper: Stocker un rapport TÉO Hub localement
// ============================================================================

/// Convertit un rapport TÉO Hub en pending_report local et le stocke en SQLite
fn store_teo_report_locally(
    state: &web::Data<HttpServerState>,
    teo_report: &teo_client::models::TeoAiReportResponse,
    patient_id: Option<&str>,
    exam_uid: Option<&str>,
    accession_number: Option<&str>,
) -> Option<String> {
    // Générer un technical_id unique
    let tid = format!("teo_{}", &Uuid::new_v4().to_string()[..8]);
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::hours(24);
    
    // Construire le JSON structuré depuis la réponse TÉO
    let structured = serde_json::json!({
        "title": teo_report.result.structured_report.title,
        "results": teo_report.result.structured_report.results,
        "conclusion": teo_report.result.structured_report.conclusion,
        "translated_text": teo_report.result.translation.translated_text,
        "translation_language": teo_report.result.translation.language,
    });
    let structured_json = serde_json::to_string(&structured).unwrap_or_default();
    
    match state.db.insert_pending_report(
        &id,
        &tid,
        patient_id,
        exam_uid,
        accession_number,
        None, // study_instance_uid
        &structured_json,
        "teo_hub_auto",
        None, // ai_modules
        None, // modality
        None, // metadata
        &now.to_rfc3339(),
        &expires_at.to_rfc3339(),
    ) {
        Ok(_) => Some(tid),
        Err(e) => {
            log::error!("❌ [HTTP] Erreur stockage rapport TÉO Hub: {}", e);
            None
        }
    }
}

// ============================================================================
// GET /teo-hub/fetch - Récupère un rapport depuis TÉO Hub sans navigation
// ============================================================================

/// GET /teo-hub/fetch - Fetch un rapport depuis TÉO Hub, le stocke localement,
/// retourne le technical_id et retrieval_url sans déclencher la navigation iframe.
pub async fn fetch_from_teo_hub(
    req: HttpRequest,
    query: web::Query<TeoHubFetchQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
    // 🔒 Authentification requise
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !is_auth_disabled() && !validate_api_key(&state.db, api_key) {
        log::warn!("❌ [HTTP] GET /teo-hub/fetch sans API key valide");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid API key"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "API key required".to_string(),
            field: None,
        });
    }
    
    // Vérifier que TÉO Hub est activé
    let config = get_config();
    if !config.teo_hub.enabled {
        request_info.log_access(&state.db, 503, "error", Some("TÉO Hub disabled"));
        return HttpResponse::ServiceUnavailable().json(TeoHubFetchResponse {
            success: false,
            technical_id: None,
            retrieval_url: None,
            source: None,
            error: Some("TÉO Hub is not enabled in configuration".to_string()),
        });
    }
    
    // Résoudre patient_id et study_uid
    let patient_id = query.patient_id.as_deref().unwrap_or("");
    let study_uid = query.study_uid.as_deref()
        .or(query.exam_uid.as_deref())
        .unwrap_or("");
    
    if patient_id.is_empty() || study_uid.is_empty() {
        request_info.log_access(&state.db, 400, "bad_request", Some("Missing patient_id or study_uid"));
        return HttpResponse::BadRequest().json(TeoHubFetchResponse {
            success: false,
            technical_id: None,
            retrieval_url: None,
            source: None,
            error: Some("Both patient_id and study_uid (or exam_uid) are required".to_string()),
        });
    }
    
    log::info!("🔄 [HTTP] TÉO Hub fetch: patient_id={}, study_uid={}...",
        mask_sensitive_id(patient_id), &study_uid[..study_uid.len().min(20)]);
    
    match teo_client::fetch_ai_report(patient_id, study_uid).await {
        Ok(teo_report) => {
            match store_teo_report_locally(
                &state,
                &teo_report,
                Some(patient_id),
                Some(study_uid),
                None,
            ) {
                Some(tid) => {
                    log::info!("✅ [HTTP] TÉO Hub fetch réussi: tid={}", tid);
                    request_info.log_access(&state.db, 200, "success", None);
                    HttpResponse::Ok().json(TeoHubFetchResponse {
                        success: true,
                        technical_id: Some(tid.clone()),
                        retrieval_url: Some(format!("https://airadcr.com/app?tori=true&tid={}", tid)),
                        source: Some("teo_hub".to_string()),
                        error: None,
                    })
                }
                None => {
                    request_info.log_access(&state.db, 500, "error", Some("Failed to store TÉO report locally"));
                    HttpResponse::InternalServerError().json(TeoHubFetchResponse {
                        success: false,
                        technical_id: None,
                        retrieval_url: None,
                        source: None,
                        error: Some("Failed to store report in local database".to_string()),
                    })
                }
            }
        }
        Err(e) => {
            let error_msg = format!("TÉO Hub fetch failed: {}", e);
            log::error!("❌ [HTTP] {}", error_msg);
            request_info.log_access(&state.db, 502, "error", Some(&error_msg));
            HttpResponse::BadGateway().json(TeoHubFetchResponse {
                success: false,
                technical_id: None,
                retrieval_url: None,
                source: None,
                error: Some(error_msg),
            })
        }
    }
}
