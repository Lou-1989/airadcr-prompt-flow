// ============================================================================
// AIRADCR Desktop - Module Serveur HTTP Local (Port 8741)
// ============================================================================
// Ce module expose un serveur HTTP local pour recevoir des rapports
// radiologiques pré-remplis depuis des systèmes RIS/PACS externes.
// ============================================================================

pub mod routes;
pub mod handlers;
pub mod middleware;

use actix_web::{App, HttpServer, web, middleware::Logger};
use actix_cors::Cors;
use std::sync::Arc;
use crate::database::Database;

/// État partagé du serveur HTTP
pub struct HttpServerState {
    pub db: Arc<Database>,
}

/// Démarre le serveur HTTP sur le port spécifié
pub async fn start_server(port: u16, db: Arc<Database>) -> std::io::Result<()> {
    let state = web::Data::new(HttpServerState { db });
    
    println!("🌐 [HTTP Server] Démarrage sur http://127.0.0.1:{}", port);
    
    HttpServer::new(move || {
        // Configuration CORS permissive pour localhost et airadcr.com
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://airadcr.com")
            .allowed_origin("https://www.airadcr.com")
            .allowed_origin_fn(|origin, _req_head| {
                // Autoriser tous les ports localhost
                origin.as_bytes().starts_with(b"http://localhost:")
            })
            .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::HeaderName::from_static("x-api-key"),
            ])
            .max_age(3600);
        
        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(Logger::new("%a \"%r\" %s %b %Dms"))
            .configure(routes::configure)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
