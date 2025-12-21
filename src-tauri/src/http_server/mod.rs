// ============================================================================
// AIRADCR Desktop - Module Serveur HTTP Local (Port 8741)
// ============================================================================
// Ce module expose un serveur HTTP local pour recevoir des rapports
// radiologiques pré-remplis depuis des systèmes RIS/PACS externes.
// ============================================================================

pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod metrics;

use actix_web::{App, HttpServer, web, middleware::Logger};
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use std::sync::Arc;
use crate::database::Database;

/// État partagé du serveur HTTP
pub struct HttpServerState {
    pub db: Arc<Database>,
}

/// Démarre le serveur HTTP sur le port spécifié avec retry sur ports alternatifs
pub async fn start_server(port: u16, db: Arc<Database>) -> std::io::Result<()> {
    let state = web::Data::new(HttpServerState { db });
    
    // 🔄 Tentative de binding avec ports alternatifs
    let ports_to_try = [port, port + 1, port + 2]; // 8741, 8742, 8743
    
    for &try_port in &ports_to_try {
        println!("🌐 [HTTP Server] Tentative de démarrage sur http://127.0.0.1:{}", try_port);
        
        // Configuration du rate limiting : 60 requêtes par minute par IP
        let governor_conf = GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(60)
            .finish()
            .unwrap();
        
        let state_clone = state.clone();
        
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_origin("http://localhost:5173")
                .allowed_origin("http://localhost:8080")
                .allowed_origin("https://airadcr.com")
                .allowed_origin("https://www.airadcr.com")
                .allowed_origin_fn(|origin, _req_head| {
                    origin.as_bytes().starts_with(b"http://localhost:")
                })
                .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::CONTENT_TYPE,
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::HeaderName::from_static("x-api-key"),
                    actix_web::http::header::HeaderName::from_static("x-admin-key"),
                ])
                .max_age(3600);
            
            App::new()
                .app_data(state_clone.clone())
                .wrap(Governor::new(&governor_conf))
                .wrap(cors)
                .wrap(Logger::new("%a \"%r\" %s %b %Dms"))
                .configure(routes::configure)
        })
        .client_request_timeout(std::time::Duration::from_secs(30))
        .keep_alive(std::time::Duration::from_secs(75));
        
        match server.bind(("127.0.0.1", try_port)) {
            Ok(bound_server) => {
                if try_port != port {
                    println!("⚠️  [HTTP Server] Port {} occupé, utilisation du port alternatif {}", port, try_port);
                }
                println!("✅ [HTTP Server] Démarré avec succès sur http://127.0.0.1:{}", try_port);
                return bound_server.run().await;
            }
            Err(e) => {
                println!("⚠️  [HTTP Server] Échec binding port {}: {}", try_port, e);
                if try_port == *ports_to_try.last().unwrap() {
                    // Dernier port testé, retourner l'erreur
                    println!("❌ [HTTP Server] Tous les ports ({:?}) sont occupés", ports_to_try);
                    println!("💡 [HTTP Server] Une autre instance AIRADCR est peut-être déjà en cours d'exécution");
                    return Err(e);
                }
                // Continuer avec le port suivant
            }
        }
    }
    
    // Cas improbable mais nécessaire pour le compilateur
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "Tous les ports alternatifs sont occupés"
    ))
}
