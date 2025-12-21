// ============================================================================
// AIRADCR Desktop - Routes HTTP
// ============================================================================

use actix_web::web;
use super::handlers;
use super::metrics;

/// Configure toutes les routes du serveur HTTP
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check (sans authentification)
        .route("/health", web::get().to(handlers::health_check))
        
        // 🆕 Health check étendu (Phase 2)
        .route("/health/extended", web::get().to(metrics::extended_health_handler))
        
        // 🆕 Métriques Prometheus (Phase 2)
        .route("/metrics", web::get().to(metrics::metrics_handler))
        
        // Pending reports API
        .route("/pending-report", web::post().to(handlers::store_pending_report))
        .route("/pending-report", web::get().to(handlers::get_pending_report))
        .route("/pending-report", web::delete().to(handlers::delete_pending_report))
        
        // RIS search endpoint (search by patient identifiers)
        .route("/find-report", web::get().to(handlers::find_report))
        
        // RIS navigation endpoint (open report in iframe)
        .route("/open-report", web::post().to(handlers::open_report))
        
        // API Keys management (admin only)
        .route("/api-keys", web::post().to(handlers::create_api_key))
        .route("/api-keys", web::get().to(handlers::list_api_keys))
        .route("/api-keys/{prefix}", web::delete().to(handlers::revoke_api_key));
}
