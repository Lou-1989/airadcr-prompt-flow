 // ============================================================================
 // AIRADCR Desktop - Client HTTP TÉO Hub
 // ============================================================================
 // Module client pour communiquer avec le serveur TÉO Hub
 // GET /th_get_ai_report - Récupérer un rapport IA
 // POST /th_post_approved_report - Envoyer un rapport validé
 // ============================================================================
 
 pub mod models;
 pub mod errors;
 
 use crate::config::get_config;
 use errors::TeoClientError;
 use models::{TeoHealthResponse, TeoAiReport, TeoApprovedReport, TeoApprovalResponse};
 use log::{info, warn, debug, error};
 use std::time::Duration;
 use std::sync::OnceLock;
 
 // Client HTTP global (réutilisable)
 static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
 
 /// Obtient ou crée le client HTTP singleton
 fn get_http_client() -> Result<&'static reqwest::Client, TeoClientError> {
     HTTP_CLIENT.get_or_try_init(|| {
         let config = get_config();
         let timeout = Duration::from_secs(config.teo_hub.timeout_secs);
         
         let mut builder = reqwest::Client::builder()
             .timeout(timeout)
             .connect_timeout(Duration::from_secs(10))
             .pool_max_idle_per_host(2);
         
         // Configuration TLS si activée
         if config.teo_hub.tls_enabled {
             debug!("[TÉO Client] Configuration TLS activée");
             
             // Charger le certificat CA personnalisé si fourni
             if !config.teo_hub.ca_file.is_empty() {
                 let ca_path = std::path::Path::new(&config.teo_hub.ca_file);
                 if ca_path.exists() {
                     let ca_cert = std::fs::read(ca_path)
                         .map_err(|e| TeoClientError::TlsError(format!("Erreur lecture CA: {}", e)))?;
                     let cert = reqwest::Certificate::from_pem(&ca_cert)
                         .map_err(|e| TeoClientError::TlsError(format!("Erreur parsing CA: {}", e)))?;
                     builder = builder.add_root_certificate(cert);
                     debug!("[TÉO Client] Certificat CA chargé: {:?}", ca_path);
                 } else {
                     warn!("[TÉO Client] Fichier CA introuvable: {:?}", ca_path);
                 }
             }
             
             // Charger le certificat client si fourni (mTLS)
             if !config.teo_hub.cert_file.is_empty() && !config.teo_hub.key_file.is_empty() {
                 let cert_path = std::path::Path::new(&config.teo_hub.cert_file);
                 let key_path = std::path::Path::new(&config.teo_hub.key_file);
                 
                 if cert_path.exists() && key_path.exists() {
                     let cert_pem = std::fs::read(cert_path)
                         .map_err(|e| TeoClientError::TlsError(format!("Erreur lecture cert: {}", e)))?;
                     let key_pem = std::fs::read(key_path)
                         .map_err(|e| TeoClientError::TlsError(format!("Erreur lecture key: {}", e)))?;
                     
                     // Combiner cert + key pour Identity
                     let mut identity_pem = cert_pem.clone();
                     identity_pem.extend_from_slice(&key_pem);
                     
                     let identity = reqwest::Identity::from_pem(&identity_pem)
                         .map_err(|e| TeoClientError::TlsError(format!("Erreur création identité: {}", e)))?;
                     builder = builder.identity(identity);
                     debug!("[TÉO Client] Certificat client (mTLS) chargé");
                 } else {
                     warn!("[TÉO Client] Certificat client ou clé introuvable");
                 }
             }
         }
         
         builder.build()
             .map_err(|e| TeoClientError::ClientError(format!("Erreur création client HTTP: {}", e)))
     })
 }
 
 /// Construit l'URL de base du serveur TÉO Hub
 fn build_base_url() -> String {
     let config = get_config();
     let protocol = if config.teo_hub.tls_enabled { "https" } else { "http" };
     format!("{}://{}:{}", protocol, config.teo_hub.host, config.teo_hub.port)
 }
 
 /// Masque les PII dans les logs (patient_id: 1234**** )
 fn mask_pii(patient_id: &str) -> String {
     if patient_id.len() <= 4 {
         "****".to_string()
     } else {
         format!("{}****", &patient_id[..4])
     }
 }
 
 /// Ajoute les headers d'authentification selon la configuration
 fn add_auth_headers(mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
     let config = get_config();
     
     // API Key header
     if !config.teo_hub.api_key.is_empty() {
         request = request.header("X-API-Key", &config.teo_hub.api_key);
         debug!("[TÉO Client] Header X-API-Key ajouté");
     }
     
     // Bearer token
     if !config.teo_hub.bearer_token.is_empty() {
         request = request.header("Authorization", format!("Bearer {}", config.teo_hub.bearer_token));
         debug!("[TÉO Client] Header Authorization Bearer ajouté");
     }
     
     request
 }
 
 // ============================================================================
 // API PUBLIQUE
 // ============================================================================
 
 /// Vérifie la disponibilité du serveur TÉO Hub
 pub async fn check_health() -> Result<TeoHealthResponse, TeoClientError> {
     let config = get_config();
     
     if !config.teo_hub.enabled {
         return Err(TeoClientError::Disabled);
     }
     
     let client = get_http_client()?;
     let url = format!("{}/{}", build_base_url(), config.teo_hub.health_endpoint);
     
     debug!("[TÉO Client] Health check: {}", url);
     
     let request = add_auth_headers(client.get(&url));
     
     let response = request.send().await
         .map_err(|e| TeoClientError::NetworkError(e.to_string()))?;
     
     if !response.status().is_success() {
         let status = response.status();
         let body = response.text().await.unwrap_or_default();
         return Err(TeoClientError::HttpError(status.as_u16(), body));
     }
     
     let health: TeoHealthResponse = response.json().await
         .map_err(|e| TeoClientError::ParseError(e.to_string()))?;
     
     info!("[TÉO Client] Health check OK: status={}", health.status);
     Ok(health)
 }
 
 /// Récupère un rapport IA depuis TÉO Hub par accession_number
 pub async fn fetch_ai_report(accession_number: &str) -> Result<TeoAiReport, TeoClientError> {
     let config = get_config();
     
     if !config.teo_hub.enabled {
         return Err(TeoClientError::Disabled);
     }
     
     let client = get_http_client()?;
     let url = format!("{}/{}?accession_number={}", 
         build_base_url(), 
         config.teo_hub.get_report_endpoint,
         accession_number
     );
     
     debug!("[TÉO Client] Fetch AI report: {}", url);
     
     let request = add_auth_headers(client.get(&url));
     
     // Retry logic avec backoff exponentiel
     let mut last_error = None;
     for attempt in 0..config.teo_hub.retry_count {
         if attempt > 0 {
             let delay = config.teo_hub.retry_delay_ms * (2_u64.pow(attempt as u32 - 1));
             tokio::time::sleep(Duration::from_millis(delay)).await;
             debug!("[TÉO Client] Retry {} après {}ms", attempt, delay);
         }
         
         let request_clone = add_auth_headers(client.get(&url));
         
         match request_clone.send().await {
             Ok(response) => {
                 if response.status().is_success() {
                     let report: TeoAiReport = response.json().await
                         .map_err(|e| TeoClientError::ParseError(e.to_string()))?;
                     
                     info!("[TÉO Client] Rapport IA récupéré: report_id={}, patient_id={}", 
                         report.report_id, mask_pii(&report.patient_id));
                     return Ok(report);
                 } else if response.status().as_u16() == 404 {
                     return Err(TeoClientError::NotFound(accession_number.to_string()));
                 } else if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
                     let body = response.text().await.unwrap_or_default();
                     return Err(TeoClientError::Unauthorized(body));
                 } else {
                     let status = response.status();
                     let body = response.text().await.unwrap_or_default();
                     last_error = Some(TeoClientError::HttpError(status.as_u16(), body));
                 }
             }
             Err(e) => {
                 last_error = Some(TeoClientError::NetworkError(e.to_string()));
             }
         }
     }
     
     Err(last_error.unwrap_or_else(|| TeoClientError::NetworkError("Erreur inconnue".to_string())))
 }
 
 /// Envoie un rapport validé à TÉO Hub
 pub async fn submit_approved_report(report: TeoApprovedReport) -> Result<TeoApprovalResponse, TeoClientError> {
     let config = get_config();
     
     if !config.teo_hub.enabled {
         return Err(TeoClientError::Disabled);
     }
     
     let client = get_http_client()?;
     let url = format!("{}/{}", build_base_url(), config.teo_hub.post_report_endpoint);
     
     debug!("[TÉO Client] Submit approved report: {} (report_id={})", url, report.report_id);
     
     // Retry logic avec backoff exponentiel
     let mut last_error = None;
     for attempt in 0..config.teo_hub.retry_count {
         if attempt > 0 {
             let delay = config.teo_hub.retry_delay_ms * (2_u64.pow(attempt as u32 - 1));
             tokio::time::sleep(Duration::from_millis(delay)).await;
             debug!("[TÉO Client] Retry {} après {}ms", attempt, delay);
         }
         
         let request = add_auth_headers(client.post(&url))
             .header("Content-Type", "application/json")
             .json(&report);
         
         match request.send().await {
             Ok(response) => {
                 if response.status().is_success() {
                     let result: TeoApprovalResponse = response.json().await
                         .map_err(|e| TeoClientError::ParseError(e.to_string()))?;
                     
                     info!("[TÉO Client] Rapport approuvé soumis: report_id={}, status={}", 
                         report.report_id, result.status);
                     return Ok(result);
                 } else if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
                     let body = response.text().await.unwrap_or_default();
                     return Err(TeoClientError::Unauthorized(body));
                 } else {
                     let status = response.status();
                     let body = response.text().await.unwrap_or_default();
                     last_error = Some(TeoClientError::HttpError(status.as_u16(), body));
                 }
             }
             Err(e) => {
                 last_error = Some(TeoClientError::NetworkError(e.to_string()));
             }
         }
     }
     
     Err(last_error.unwrap_or_else(|| TeoClientError::NetworkError("Erreur inconnue".to_string())))
 }
 
 /// Récupère le statut de connexion actuel
 pub fn get_connection_status() -> TeoConnectionStatus {
     let config = get_config();
     
     if !config.teo_hub.enabled {
         return TeoConnectionStatus::Disabled;
     }
     
     // On ne peut pas vérifier la connexion de manière synchrone
     // Retourne "Unknown" - le frontend doit appeler check_health() async
     TeoConnectionStatus::Unknown
 }
 
 /// Statut de connexion TÉO Hub
 #[derive(Debug, Clone, serde::Serialize)]
 pub enum TeoConnectionStatus {
     Connected,
     Disconnected,
     Disabled,
     Unknown,
 }