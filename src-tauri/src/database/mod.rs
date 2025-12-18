// ============================================================================
// AIRADCR Desktop - Module Base de Données SQLite
// ============================================================================
// Ce module gère le stockage local des rapports en attente et des clés API.
// La base est stockée dans le répertoire AppData de l'application.
// ============================================================================

pub mod schema;
pub mod queries;

use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;
use std::path::PathBuf;

/// Structure principale de la base de données thread-safe
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Crée ou ouvre la base de données
    pub fn new(app_data_dir: PathBuf) -> SqlResult<Self> {
        // Créer le répertoire si nécessaire
        std::fs::create_dir_all(&app_data_dir).ok();
        
        let db_path = app_data_dir.join("pending_reports.db");
        println!("📂 [Database] Chemin: {:?}", db_path);
        
        let conn = Connection::open(&db_path)?;
        
        // Initialiser le schéma
        schema::initialize(&conn)?;
        
        println!("✅ [Database] Base initialisée avec succès");
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// Crée une base en mémoire (pour les tests)
    #[allow(dead_code)]
    pub fn new_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::initialize(&conn)?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// Exécute une opération avec la connexion
    pub fn with_connection<F, T>(&self, f: F) -> SqlResult<T>
    where
        F: FnOnce(&Connection) -> SqlResult<T>,
    {
        let conn = self.conn.lock().map_err(|_| {
            rusqlite::Error::ExecuteReturnedResults
        })?;
        f(&conn)
    }
}

// ============================================================================
// Implémentation des méthodes de requête (délégation au module queries)
// ============================================================================

impl Database {
    /// Insère un nouveau rapport en attente (avec identifiants patients pour LOCAL)
    pub fn insert_pending_report(
        &self,
        id: &str,
        technical_id: &str,
        patient_id: Option<&str>,
        exam_uid: Option<&str>,
        accession_number: Option<&str>,
        study_instance_uid: Option<&str>,
        structured_data: &str,
        source_type: &str,
        ai_modules: Option<&str>,
        modality: Option<&str>,
        metadata: Option<&str>,
        created_at: &str,
        expires_at: &str,
    ) -> SqlResult<()> {
        self.with_connection(|conn| {
            queries::insert_pending_report(
                conn,
                id,
                technical_id,
                patient_id,
                exam_uid,
                accession_number,
                study_instance_uid,
                structured_data,
                source_type,
                ai_modules,
                modality,
                metadata,
                created_at,
                expires_at,
            )
        })
    }
    
    /// Récupère un rapport par son technical_id
    pub fn get_pending_report(&self, technical_id: &str) -> SqlResult<Option<queries::PendingReport>> {
        self.with_connection(|conn| {
            queries::get_pending_report_by_tid(conn, technical_id)
        })
    }
    
    /// Marque un rapport comme récupéré
    pub fn mark_as_retrieved(&self, technical_id: &str) -> SqlResult<bool> {
        self.with_connection(|conn| {
            queries::mark_as_retrieved(conn, technical_id)
        })
    }
    
    /// Supprime un rapport
    pub fn delete_pending_report(&self, technical_id: &str) -> SqlResult<bool> {
        self.with_connection(|conn| {
            queries::delete_pending_report(conn, technical_id)
        })
    }
    
    /// Nettoie les rapports expirés
    pub fn cleanup_expired_reports(&self) -> SqlResult<usize> {
        self.with_connection(|conn| {
            queries::cleanup_expired_reports(conn)
        })
    }
    
    /// Valide une clé API
    pub fn validate_api_key(&self, key_prefix: &str, key_hash: &str) -> SqlResult<bool> {
        self.with_connection(|conn| {
            queries::validate_api_key(conn, key_prefix, key_hash)
        })
    }
    
    /// Ajoute une clé API (pour l'administration)
    pub fn add_api_key(&self, id: &str, key_prefix: &str, key_hash: &str, name: &str) -> SqlResult<()> {
        self.with_connection(|conn| {
            queries::add_api_key(conn, id, key_prefix, key_hash, name)
        })
    }
    
    /// Liste toutes les clés API
    pub fn list_api_keys(&self) -> SqlResult<Vec<(String, String, String, bool, String)>> {
        self.with_connection(|conn| {
            queries::list_api_keys(conn)
        })
    }
    
    /// Révoque une clé API (soft-delete)
    pub fn revoke_api_key(&self, key_prefix: &str) -> SqlResult<bool> {
        self.with_connection(|conn| {
            queries::revoke_api_key(conn, key_prefix)
        })
    }
    
    /// Recherche un rapport par identifiants RIS
    pub fn find_pending_report_by_identifiers(
        &self,
        patient_id: Option<&str>,
        accession_number: Option<&str>,
        exam_uid: Option<&str>,
    ) -> SqlResult<Option<queries::PendingReport>> {
        self.with_connection(|conn| {
            queries::find_pending_report_by_identifiers(conn, patient_id, accession_number, exam_uid)
        })
    }
    
    /// Liste tous les rapports (pour Debug Panel)
    pub fn list_all_pending_reports(&self) -> SqlResult<Vec<queries::PendingReportSummary>> {
        self.with_connection(|conn| {
            queries::list_all_pending_reports(conn)
        })
    }
    
    /// Récupère les statistiques de la base
    pub fn get_database_stats(&self) -> SqlResult<queries::DatabaseStats> {
        self.with_connection(|conn| {
            queries::get_database_stats(conn)
        })
    }
    
    /// Liste les clés API (version simplifiée pour Debug Panel)
    pub fn list_api_keys_summary(&self) -> SqlResult<Vec<queries::ApiKeySummary>> {
        self.with_connection(|conn| {
            queries::list_api_keys_summary(conn)
        })
    }
}
