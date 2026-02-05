 // ============================================================================
 // AIRADCR Desktop - Modèles TÉO Hub
 // ============================================================================
 // Structures de données pour la communication avec TÉO Hub
 // ============================================================================
 
 use serde::{Deserialize, Serialize};
 use serde_json::Value;
 
 /// Réponse du health check TÉO Hub
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct TeoHealthResponse {
     pub status: String,
     #[serde(default)]
     pub version: Option<String>,
     #[serde(default)]
     pub uptime_seconds: Option<u64>,
 }
 
 /// Analyse IA incluse dans le rapport
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct TeoAiAnalysis {
     /// Findings détectés par l'IA
     #[serde(default)]
     pub findings: Vec<String>,
     
     /// Mesures (volumétrie, dimensions, etc.)
     #[serde(default)]
     pub measurements: Option<Value>,
     
     /// Score de confiance global (0.0 - 1.0)
     #[serde(default)]
     pub confidence_score: f64,
     
     /// Modules IA utilisés (ex: ["b-rayz", "volumetry"])
     #[serde(default)]
     pub ai_modules: Vec<String>,
     
     /// Texte brut généré par sr_to_text()
     #[serde(default)]
     pub raw_text: Option<String>,
     
     /// Données DICOM SR originales (optionnel)
     #[serde(default)]
     pub dicom_sr: Option<Value>,
 }
 
 /// Rapport IA complet reçu de TÉO Hub (GET /th_get_ai_report)
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct TeoAiReport {
     /// Identifiant unique du rapport dans TÉO Hub
     pub report_id: String,
     
     /// Numéro d'accession (identifiant RIS)
     pub accession_number: String,
     
     /// Identifiant patient (masqué dans les logs)
     #[serde(default)]
     pub patient_id: String,
     
     /// Study Instance UID DICOM
     #[serde(default)]
     pub study_instance_uid: String,
     
     /// Modalité DICOM (MR, CT, MG, etc.)
     #[serde(default)]
     pub modality: String,
     
     /// Analyse IA structurée
     pub ai_analysis: TeoAiAnalysis,
     
     /// Template ID AIRADCR à utiliser (optionnel)
     #[serde(default)]
     pub template_id: Option<String>,
     
     /// Statut du rapport (pending, approved, etc.)
     #[serde(default)]
     pub status: String,
     
     /// Date de création
     #[serde(default)]
     pub created_at: String,
     
     /// Métadonnées additionnelles
     #[serde(default)]
     pub metadata: Option<Value>,
 }
 
 /// Rapport approuvé à envoyer à TÉO Hub (POST /th_post_approved_report)
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct TeoApprovedReport {
     /// Identifiant du rapport original
     pub report_id: String,
     
     /// Numéro d'accession
     pub accession_number: String,
     
     /// Texte du rapport final validé
     pub approved_text: String,
     
     /// Identifiant du radiologue (optionnel)
     #[serde(default)]
     pub radiologist_id: Option<String>,
     
     /// Nom du radiologue (optionnel)
     #[serde(default)]
     pub radiologist_name: Option<String>,
     
     /// Timestamp d'approbation (ISO 8601)
     pub approval_timestamp: String,
     
     /// Indique si le radiologue a modifié le rapport IA
     #[serde(default)]
     pub modifications_made: bool,
     
     /// Sections modifiées (optionnel)
     #[serde(default)]
     pub modified_sections: Option<Vec<String>>,
     
     /// Signature électronique (optionnel)
     #[serde(default)]
     pub signature: Option<String>,
 }
 
 /// Réponse après soumission d'un rapport approuvé
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct TeoApprovalResponse {
     /// Statut de la soumission
     pub status: String,
     
     /// Message de confirmation
     #[serde(default)]
     pub message: Option<String>,
     
     /// Identifiant de transaction (optionnel)
     #[serde(default)]
     pub transaction_id: Option<String>,
     
     /// Timestamp de réception
     #[serde(default)]
     pub received_at: Option<String>,
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
     /// Indique si une API key est configurée (sans révéler la valeur)
     pub has_api_key: bool,
     /// Indique si un bearer token est configuré
     pub has_bearer_token: bool,
     /// Indique si des certificats TLS sont configurés
     pub has_tls_certs: bool,
 }