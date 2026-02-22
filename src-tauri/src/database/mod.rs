// ============================================================================
// AIRADCR Desktop - Module Base de Données SQLite
// ============================================================================
// Ce module gère le stockage local des rapports en attente et des clés API.
// La base est stockée dans le répertoire AppData de l'application.
// ============================================================================

pub mod schema;
pub mod queries;
pub mod backup;
pub mod keychain;

use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;
use std::path::PathBuf;
use log::{info, warn, error};

/// Structure principale de la base de données thread-safe
pub struct Database {
    conn: Mutex<Connection>,
}

/// Applique la clé de chiffrement SQLCipher sur une connexion ouverte
fn apply_sqlcipher_key(conn: &Connection, encryption_key: &str) -> SqlResult<()> {
    // PRAGMA key doit être la PREMIÈRE instruction après l'ouverture
    conn.execute_batch(&format!("PRAGMA key = '{}';", encryption_key))?;
    
    // Vérifier que le chiffrement fonctionne en lisant une table système
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")?;
    
    Ok(())
}

impl Database {
    /// Crée ou ouvre la base de données chiffrée avec SQLCipher
    pub fn new(app_data_dir: PathBuf) -> SqlResult<Self> {
        // Créer le répertoire si nécessaire
        std::fs::create_dir_all(&app_data_dir).ok();
        
        let db_path = app_data_dir.join("pending_reports.db");
        info!("[Database] Chemin: {:?}", db_path);
        
        // Récupérer la clé de chiffrement depuis le keychain OS
        let encryption_key = keychain::get_or_create_db_encryption_key()
            .map_err(|e| {
                error!("[Database] Erreur keychain: {}", e);
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_AUTH),
                    Some(format!("Erreur keychain: {}", e)),
                )
            })?;
        
        let db_exists = db_path.exists();
        let conn = Connection::open(&db_path)?;
        
        if db_exists {
            // Base existante : essayer d'abord avec la clé SQLCipher
            match apply_sqlcipher_key(&conn, &encryption_key) {
                Ok(_) => {
                    info!("[Database] Base chiffrée SQLCipher ouverte avec succès");
                }
                Err(_) => {
                    // La base existante n'est probablement pas chiffrée (migration)
                    warn!("[Database] Base non chiffrée détectée, migration vers SQLCipher...");
                    drop(conn);
                    Self::migrate_to_encrypted(&db_path, &encryption_key)?;
                    let conn = Connection::open(&db_path)?;
                    apply_sqlcipher_key(&conn, &encryption_key)?;
                    schema::initialize(&conn)?;
                    info!("[Database] Migration SQLCipher terminée avec succès");
                    return Ok(Self {
                        conn: Mutex::new(conn),
                    });
                }
            }
        } else {
            // Nouvelle base : appliquer directement la clé
            apply_sqlcipher_key(&conn, &encryption_key)?;
            info!("[Database] Nouvelle base SQLCipher créée");
        }
        
        // Initialiser le schéma
        schema::initialize(&conn)?;
        
        info!("[Database] Base initialisée avec succès (chiffrée AES-256)");
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// Migre une base non chiffrée vers SQLCipher
    fn migrate_to_encrypted(db_path: &std::path::Path, encryption_key: &str) -> SqlResult<()> {
        let backup_path = db_path.with_extension("db.unencrypted.bak");
        let encrypted_path = db_path.with_extension("db.encrypted");
        
        // 1. Ouvrir la base non chiffrée
        let plain_conn = Connection::open(db_path)?;
        
        // 2. Exporter vers une nouvelle base chiffrée via ATTACH + sqlcipher_export
        plain_conn.execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY '{}';
             SELECT sqlcipher_export('encrypted');
             DETACH DATABASE encrypted;",
            encrypted_path.display(),
            encryption_key
        ))?;
        
        drop(plain_conn);
        
        // 3. Renommer : ancien → backup, chiffré → principal
        std::fs::rename(db_path, &backup_path).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(format!("Erreur rename backup: {}", e)),
            )
        })?;
        
        std::fs::rename(&encrypted_path, db_path).map_err(|e| {
            // Restaurer le backup en cas d'erreur
            let _ = std::fs::rename(&backup_path, db_path);
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(format!("Erreur rename encrypted: {}", e)),
            )
        })?;
        
        info!("[Database] Migration SQLCipher: backup conservé à {:?}", backup_path);
        Ok(())
    }
    
    /// Crée une base en mémoire (pour les tests) — non chiffrée
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
    
    // =========================================================================
    // Opérations sur les logs d'accès API (AUDIT)
    // =========================================================================
    
    /// Insère un log d'accès API
    pub fn insert_access_log(
        &self,
        timestamp: &str,
        ip_address: &str,
        method: &str,
        endpoint: &str,
        status_code: i32,
        result: &str,
        api_key_prefix: Option<&str>,
        user_agent: Option<&str>,
        request_id: &str,
        duration_ms: i64,
        error_message: Option<&str>,
    ) -> SqlResult<i64> {
        self.with_connection(|conn| {
            queries::insert_access_log(
                conn,
                timestamp,
                ip_address,
                method,
                endpoint,
                status_code,
                result,
                api_key_prefix,
                user_agent,
                request_id,
                duration_ms,
                error_message,
            )
        })
    }
    
    /// Liste les logs d'accès récents
    pub fn list_access_logs(&self, limit: i64, offset: i64) -> SqlResult<Vec<queries::AccessLogSummary>> {
        self.with_connection(|conn| {
            queries::list_access_logs(conn, limit, offset)
        })
    }
    
    /// Récupère les statistiques des logs d'accès
    pub fn get_access_logs_stats(&self) -> SqlResult<queries::AccessLogsStats> {
        self.with_connection(|conn| {
            queries::get_access_logs_stats(conn)
        })
    }
    
    /// Nettoie les vieux logs d'accès
    pub fn cleanup_old_access_logs(&self, days: i64) -> SqlResult<usize> {
        self.with_connection(|conn| {
            queries::cleanup_old_access_logs(conn, days)
        })
    }
    
    // =========================================================================
    // Méthodes pour métriques Prometheus (Phase 2)
    // =========================================================================
    
    /// Compte les rapports en attente
    pub fn count_pending_reports(&self) -> SqlResult<i64> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM pending_reports WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
        })
    }
    
    /// Compte les clés API actives
    pub fn count_active_api_keys(&self) -> SqlResult<i64> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM api_keys WHERE is_active = 1",
                [],
                |row| row.get(0),
            )
        })
    }
    
    /// Récupère la taille de la base de données
    pub fn get_database_size(&self) -> SqlResult<u64> {
        self.with_connection(|conn| {
            let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
            let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
            Ok((page_count * page_size) as u64)
        })
    }
}
