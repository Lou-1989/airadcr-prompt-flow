// ============================================================================
// AIRADCR Desktop - Modèles TÉO Hub
// ============================================================================
// Structures de données conformes à l'API réelle TÉO Hub (AiReportWebServer)
// Référence: try-airadcr_web_server.py (HCK HealthCare Konnect SA)
// ============================================================================

use serde::{Deserialize, Serialize};

/// Réponse du health check TÉO Hub (GET /th_health)
/// Format réel: {"ok": true, "service": "AiReportWebServer/1.0"}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoHealthResponse {
    pub ok: bool,
    #[serde(default)]
    pub service: String,
}

/// Traduction incluse dans le résultat du rapport IA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoTranslation {
    /// Langue de la traduction (ex: "fr")
    #[serde(default)]
    pub language: String,

    /// Texte traduit complet
    #[serde(default)]
    pub translated_text: String,
}

/// Rapport structuré inclus dans le résultat du rapport IA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoStructuredReport {
    /// Titre du rapport (ex: "Mammographie")
    #[serde(default)]
    pub title: String,

    /// Résultats détaillés
    #[serde(default)]
    pub results: String,

    /// Conclusion du rapport (ex: "Sein droit: BI-RADS 1...")
    #[serde(default)]
    pub conclusion: String,
}

/// Contenu imbriqué dans "result" de la réponse GET /th_get_ai_report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoAiReportResult {
    pub translation: TeoTranslation,
    pub structured_report: TeoStructuredReport,
}

/// Réponse complète de GET /th_get_ai_report
/// Format réel: {"result": {"translation": {...}, "structured_report": {...}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoAiReportResponse {
    pub result: TeoAiReportResult,
}

/// Rapport approuvé à envoyer à TÉO Hub (POST /th_post_approved_report)
/// Format réel: {"patient_id": "...", "study_uid": "...", "approved_report": "..."}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoApprovedReport {
    pub patient_id: String,
    pub study_uid: String,
    pub approved_report: String,
}

/// Réponse après soumission d'un rapport approuvé
/// Format réel: {"status": "OK"}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoApprovalResponse {
    pub status: String,
}

/// Configuration TÉO Hub sérialisable pour le frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoHubConfigInfo {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub timeout_secs: u64,
    pub retry_count: u32,
    /// Indique si un API_TOKEN est configuré (sans révéler la valeur)
    pub has_api_token: bool,
    /// Indique si des certificats TLS sont configurés
    pub has_tls_certs: bool,
}
