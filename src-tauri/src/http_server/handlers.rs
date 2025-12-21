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
        // 🆕 Version dynamique depuis Cargo.toml (Phase 1)
        version: env!("CARGO_PKG_VERSION").to_string(),
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
    
    if !validate_api_key(&state.db, api_key) {
        println!("❌ [HTTP] Clé API invalide");
        request_info.log_access(&state.db, 401, "unauthorized", Some("Invalid API key"));
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid API key".to_string(),
            field: None,
        });
    }
    
    // 2. Validation technical_id
    if let Err(msg) = validate_technical_id(&body.technical_id) {
        println!("❌ [HTTP] Invalid technical_id: {}", msg);
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
            println!("✅ [HTTP] Rapport stocké: tid={}, patient_id={:?}", 
                     body.technical_id, body.patient_id);
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(StoreSuccessResponse {
                success: true,
                technical_id: body.technical_id.clone(),
                retrieval_url: format!("https://airadcr.com/app?tid={}", body.technical_id),
                expires_at: expires_at.to_rfc3339(),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur insertion: {}", e);
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
            
            println!("✅ [HTTP] Rapport récupéré: tid={}, patient_id={:?}", tid, report.patient_id);
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
            println!("⚠️ [HTTP] Rapport non trouvé: tid={}", tid);
            request_info.log_access(&state.db, 404, "not_found", Some("Report not found or expired"));
            HttpResponse::NotFound().json(GetReportResponse {
                success: false,
                data: None,
                error: Some("Report not found or expired".to_string()),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur lecture: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// DELETE /pending-report?tid=XXX - Supprime un rapport
pub async fn delete_pending_report(
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
    
    match state.db.delete_pending_report(tid) {
        Ok(deleted) => {
            println!("🗑️ [HTTP] Rapport supprimé: tid={} (deleted={})", tid, deleted);
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(DeleteResponse {
                success: true,
                deleted,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur suppression: {}", e);
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
        println!("❌ [HTTP] Clé admin invalide pour création API key");
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
            println!("✅ [HTTP] Nouvelle clé API créée: name={}, prefix={}", body.name, key_prefix);
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
            eprintln!("❌ [HTTP] Erreur création clé API: {}", e);
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
        println!("❌ [HTTP] Clé admin invalide pour liste API keys");
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
            
            println!("✅ [HTTP] Liste API keys: {} clé(s)", api_keys.len());
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(ListApiKeysResponse {
                success: true,
                keys: api_keys,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur liste API keys: {}", e);
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
        println!("❌ [HTTP] Clé admin invalide pour révocation API key");
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
                println!("✅ [HTTP] Clé API révoquée: prefix={}", prefix);
            } else {
                println!("⚠️ [HTTP] Clé API non trouvée: prefix={}", prefix);
            }
            request_info.log_access(&state.db, 200, "success", None);
            HttpResponse::Ok().json(RevokeApiKeyResponse {
                success: true,
                revoked,
                prefix,
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur révocation API key: {}", e);
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
    
    println!("🔍 [HTTP] Recherche rapport: patient_id={:?}, accession={:?}, exam_uid={:?}",
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
            
            println!("✅ [HTTP] Rapport trouvé: tid={}", report.technical_id);
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
            println!("⚠️ [HTTP] Aucun rapport trouvé pour ces identifiants");
            request_info.log_access(&state.db, 404, "not_found", Some("No report found"));
            HttpResponse::NotFound().json(FindReportResponse {
                success: false,
                data: None,
                retrieval_url: None,
                error: Some("No report found matching the provided identifiers".to_string()),
            })
        }
        Err(e) => {
            eprintln!("❌ [HTTP] Erreur recherche: {}", e);
            request_info.log_access(&state.db, 500, "error", Some(&format!("Database error: {}", e)));
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Database error: {}", e),
                field: None,
            })
        }
    }
}

/// POST /open-report - Ouvre un rapport dans l'iframe AIRADCR (navigation depuis RIS)
/// Recherche par tid OU par identifiants RIS (accession_number, patient_id, exam_uid)
pub async fn open_report(
    req: HttpRequest,
    query: web::Query<OpenReportQuery>,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    let request_info = RequestInfo::from_request(&req);
    
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
        let has_exam = query.exam_uid.as_ref().map_or(false, |s| !s.is_empty());
        
        if has_accession || has_patient || has_exam {
            match state.db.find_pending_report_by_identifiers(
                query.patient_id.as_deref(),
                query.accession_number.as_deref(),
                query.exam_uid.as_deref(),
            ) {
                Ok(Some(report)) => Some(report.technical_id),
                Ok(None) => None,
                Err(e) => {
                    eprintln!("❌ [HTTP] Erreur recherche pour open-report: {}", e);
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
            });
        }
    };
    
    // Émettre l'événement Tauri pour naviguer vers le rapport
    if let Some(app_handle) = APP_HANDLE.get() {
        if let Some(window) = app_handle.get_window("main") {
            // Émettre l'événement avec le tid
            match window.emit("airadcr:navigate_to_report", &tid) {
                Ok(_) => {
                    println!("✅ [HTTP] Navigation émise: tid={}", tid);
                    
                    // Afficher et focus la fenêtre
                    let _ = window.show();
                    let _ = window.set_focus();
                    
                    request_info.log_access(&state.db, 200, "success", None);
                    HttpResponse::Ok().json(OpenReportResponse {
                        success: true,
                        message: Some("Navigation triggered successfully".to_string()),
                        technical_id: Some(tid.clone()),
                        navigated_to: Some(format!("https://airadcr.com/app?tid={}", tid)),
                        error: None,
                    })
                }
                Err(e) => {
                    eprintln!("❌ [HTTP] Erreur émission événement: {}", e);
                    request_info.log_access(&state.db, 500, "error", Some(&format!("Event emit error: {}", e)));
                    HttpResponse::InternalServerError().json(OpenReportResponse {
                        success: false,
                        message: None,
                        technical_id: Some(tid),
                        navigated_to: None,
                        error: Some(format!("Failed to trigger navigation event: {}", e)),
                    })
                }
            }
        } else {
            eprintln!("❌ [HTTP] Fenêtre main non trouvée");
            request_info.log_access(&state.db, 500, "error", Some("Main window not found"));
            HttpResponse::InternalServerError().json(OpenReportResponse {
                success: false,
                message: None,
                technical_id: Some(tid),
                navigated_to: None,
                error: Some("Main window not found".to_string()),
            })
        }
    } else {
        eprintln!("❌ [HTTP] AppHandle non disponible (app pas encore démarrée)");
        request_info.log_access(&state.db, 503, "error", Some("Application not yet ready"));
        HttpResponse::ServiceUnavailable()
            .insert_header(("Retry-After", "2"))
            .json(OpenReportResponse {
                success: false,
                message: None,
                technical_id: Some(tid),
                navigated_to: None,
                error: Some("Application not yet ready. Please try again in a moment.".to_string()),
            })
    }
}
