// ============================================================================
// AIRADCR Desktop - Métriques Prometheus
// ============================================================================

use actix_web::{HttpResponse, web};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::HttpServerState;

/// Compteurs globaux pour métriques
pub struct Metrics {
    /// Nombre total de requêtes par endpoint
    pub requests_total: AtomicU64,
    pub requests_success: AtomicU64,
    pub requests_error: AtomicU64,
    pub requests_unauthorized: AtomicU64,
    
    /// Timestamp de démarrage du serveur
    pub start_time: u64,
    
    /// Durée totale des requêtes (pour calcul moyenne)
    pub total_duration_ms: AtomicU64,
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

impl Metrics {
    /// Initialise les métriques globales
    pub fn init() -> &'static Metrics {
        METRICS.get_or_init(|| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            Metrics {
                requests_total: AtomicU64::new(0),
                requests_success: AtomicU64::new(0),
                requests_error: AtomicU64::new(0),
                requests_unauthorized: AtomicU64::new(0),
                start_time: now,
                total_duration_ms: AtomicU64::new(0),
            }
        })
    }
    
    /// Obtient l'instance des métriques
    pub fn get() -> &'static Metrics {
        Self::init()
    }
    
    /// Enregistre une requête
    pub fn record_request(result: &str, duration_ms: u64) {
        let m = Self::get();
        m.requests_total.fetch_add(1, Ordering::Relaxed);
        m.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        match result {
            "success" => m.requests_success.fetch_add(1, Ordering::Relaxed),
            "unauthorized" => m.requests_unauthorized.fetch_add(1, Ordering::Relaxed),
            _ => m.requests_error.fetch_add(1, Ordering::Relaxed),
        };
    }
    
    /// Calcule l'uptime en secondes
    pub fn uptime_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.start_time)
    }
    
    /// Calcule la durée moyenne des requêtes
    pub fn avg_duration_ms(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.total_duration_ms.load(Ordering::Relaxed) as f64 / total as f64
    }
}

/// GET /metrics - Endpoint Prometheus (🔒 requiert clé admin)
pub async fn metrics_handler(
    req: actix_web::HttpRequest,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // 🔒 SÉCURITÉ: Exiger clé admin pour les métriques
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !super::middleware::validate_admin_key(admin_key) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Admin key required for metrics endpoint"
        }));
    }
    let m = Metrics::get();
    
    // Récupérer les stats depuis la base de données
    let (pending_count, api_keys_count, db_size) = {
        let pending = state.db.count_pending_reports().unwrap_or(0);
        let keys = state.db.count_active_api_keys().unwrap_or(0);
        let size = state.db.get_database_size().unwrap_or(0);
        (pending, keys, size)
    };
    
    // Générer le format Prometheus
    let mut output = String::new();
    
    // Métriques de requêtes
    output.push_str("# HELP airadcr_requests_total Total number of HTTP requests\n");
    output.push_str("# TYPE airadcr_requests_total counter\n");
    output.push_str(&format!("airadcr_requests_total {}\n", 
        m.requests_total.load(Ordering::Relaxed)));
    
    output.push_str("# HELP airadcr_requests_success_total Successful requests\n");
    output.push_str("# TYPE airadcr_requests_success_total counter\n");
    output.push_str(&format!("airadcr_requests_success_total {}\n", 
        m.requests_success.load(Ordering::Relaxed)));
    
    output.push_str("# HELP airadcr_requests_error_total Error requests\n");
    output.push_str("# TYPE airadcr_requests_error_total counter\n");
    output.push_str(&format!("airadcr_requests_error_total {}\n", 
        m.requests_error.load(Ordering::Relaxed)));
    
    output.push_str("# HELP airadcr_requests_unauthorized_total Unauthorized requests\n");
    output.push_str("# TYPE airadcr_requests_unauthorized_total counter\n");
    output.push_str(&format!("airadcr_requests_unauthorized_total {}\n", 
        m.requests_unauthorized.load(Ordering::Relaxed)));
    
    // Durée moyenne
    output.push_str("# HELP airadcr_request_duration_avg_ms Average request duration in milliseconds\n");
    output.push_str("# TYPE airadcr_request_duration_avg_ms gauge\n");
    output.push_str(&format!("airadcr_request_duration_avg_ms {:.2}\n", m.avg_duration_ms()));
    
    // Uptime
    output.push_str("# HELP airadcr_uptime_seconds Server uptime in seconds\n");
    output.push_str("# TYPE airadcr_uptime_seconds gauge\n");
    output.push_str(&format!("airadcr_uptime_seconds {}\n", m.uptime_seconds()));
    
    // Métriques base de données
    output.push_str("# HELP airadcr_pending_reports_count Number of pending reports\n");
    output.push_str("# TYPE airadcr_pending_reports_count gauge\n");
    output.push_str(&format!("airadcr_pending_reports_count {}\n", pending_count));
    
    output.push_str("# HELP airadcr_api_keys_active_count Number of active API keys\n");
    output.push_str("# TYPE airadcr_api_keys_active_count gauge\n");
    output.push_str(&format!("airadcr_api_keys_active_count {}\n", api_keys_count));
    
    output.push_str("# HELP airadcr_db_size_bytes Database size in bytes\n");
    output.push_str("# TYPE airadcr_db_size_bytes gauge\n");
    output.push_str(&format!("airadcr_db_size_bytes {}\n", db_size));
    
    // Version
    output.push_str("# HELP airadcr_info Application info\n");
    output.push_str("# TYPE airadcr_info gauge\n");
    output.push_str(&format!("airadcr_info{{version=\"{}\"}} 1\n", env!("CARGO_PKG_VERSION")));
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output)
}

/// Structure pour l'endpoint /health étendu
#[derive(serde::Serialize)]
pub struct ExtendedHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseHealth,
    pub requests: RequestsHealth,
}

#[derive(serde::Serialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub pending_reports: i64,
    pub api_keys_active: i64,
    pub size_bytes: u64,
}

#[derive(serde::Serialize)]
pub struct RequestsHealth {
    pub total: u64,
    pub success: u64,
    pub errors: u64,
    pub avg_duration_ms: f64,
}

/// GET /health/extended - Health check étendu (🔒 requiert clé admin)
pub async fn extended_health_handler(
    req: actix_web::HttpRequest,
    state: web::Data<HttpServerState>,
) -> HttpResponse {
    // 🔒 SÉCURITÉ: Exiger clé admin pour le health check étendu
    let admin_key = req
        .headers()
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !super::middleware::validate_admin_key(admin_key) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Admin key required for extended health endpoint"
        }));
    }
    let m = Metrics::get();
    
    // Vérifier la connexion à la base de données
    let (db_status, pending, keys, size) = match state.db.count_pending_reports() {
        Ok(pending) => {
            let keys = state.db.count_active_api_keys().unwrap_or(0);
            let size = state.db.get_database_size().unwrap_or(0);
            ("ok".to_string(), pending, keys, size)
        }
        Err(e) => (format!("error: {}", e), 0, 0, 0),
    };
    
    let response = ExtendedHealthResponse {
        status: if db_status == "ok" { "healthy".to_string() } else { "degraded".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: m.uptime_seconds(),
        database: DatabaseHealth {
            status: db_status,
            pending_reports: pending,
            api_keys_active: keys,
            size_bytes: size,
        },
        requests: RequestsHealth {
            total: m.requests_total.load(Ordering::Relaxed),
            success: m.requests_success.load(Ordering::Relaxed),
            errors: m.requests_error.load(Ordering::Relaxed),
            avg_duration_ms: m.avg_duration_ms(),
        },
    };
    
    HttpResponse::Ok().json(response)
}
