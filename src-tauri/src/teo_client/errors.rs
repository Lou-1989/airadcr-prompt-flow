 // ============================================================================
 // AIRADCR Desktop - Erreurs TÉO Client
 // ============================================================================
 
 use std::fmt;
 
 /// Erreurs possibles du client TÉO Hub
 #[derive(Debug, Clone)]
 pub enum TeoClientError {
     /// Le client TÉO Hub est désactivé dans la configuration
     Disabled,
     
     /// Erreur réseau (timeout, connexion refusée, etc.)
     NetworkError(String),
     
     /// Erreur HTTP avec code de statut
     HttpError(u16, String),
     
     /// Non autorisé (401/403)
     Unauthorized(String),
     
     /// Rapport non trouvé (404)
     NotFound(String),
     
     /// Erreur de parsing JSON
     ParseError(String),
     
     /// Erreur de configuration TLS
     TlsError(String),
     
     /// Erreur de création du client HTTP
     ClientError(String),
 }
 
 impl fmt::Display for TeoClientError {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
             TeoClientError::Disabled => write!(f, "TÉO Hub client désactivé"),
             TeoClientError::NetworkError(msg) => write!(f, "Erreur réseau: {}", msg),
             TeoClientError::HttpError(code, msg) => write!(f, "Erreur HTTP {}: {}", code, msg),
             TeoClientError::Unauthorized(msg) => write!(f, "Non autorisé: {}", msg),
             TeoClientError::NotFound(id) => write!(f, "Rapport non trouvé: {}", id),
             TeoClientError::ParseError(msg) => write!(f, "Erreur parsing: {}", msg),
             TeoClientError::TlsError(msg) => write!(f, "Erreur TLS: {}", msg),
             TeoClientError::ClientError(msg) => write!(f, "Erreur client: {}", msg),
         }
     }
 }
 
 impl std::error::Error for TeoClientError {}
 
 // Conversion en String pour les commandes Tauri
 impl From<TeoClientError> for String {
     fn from(err: TeoClientError) -> String {
         err.to_string()
     }
 }
 
 /// Résultat typé pour les opérations TÉO Client
 pub type TeoResult<T> = Result<T, TeoClientError>;