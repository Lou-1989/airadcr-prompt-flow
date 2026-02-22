// ============================================================================
// AIRADCR Desktop - Schéma SQLite
// ============================================================================

use rusqlite::{Connection, Result as SqlResult};
use sha2::{Sha256, Digest};
use rand::Rng;

/// Génère une clé API aléatoire sécurisée
fn generate_secure_api_key() -> String {
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    format!("airadcr_{}", suffix.to_lowercase())
}

/// Calcule le hash SHA-256 d'une clé API
fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Initialise le schéma de la base de données
pub fn initialize(conn: &Connection) -> SqlResult<()> {
    // Table des rapports en attente - AVEC identifiants patients (LOCAL UNIQUEMENT)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pending_reports (
            id TEXT PRIMARY KEY,
            technical_id TEXT UNIQUE NOT NULL,
            
            -- Identifiants patients (ACCEPTÉS EN LOCAL car données ne quittent pas la machine)
            patient_id TEXT,
            exam_uid TEXT,
            accession_number TEXT,
            study_instance_uid TEXT,
            
            -- Données structurées du rapport
            structured_data TEXT NOT NULL,
            source_type TEXT DEFAULT 'tauri_local',
            ai_modules TEXT,
            modality TEXT,
            metadata TEXT,
            
            -- Statut et timing
            status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'retrieved', 'expired')),
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            retrieved_at TEXT
        )",
        [],
    )?;
    
    // Table des clés API
    conn.execute(
        "CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            key_prefix TEXT NOT NULL,
            key_hash TEXT NOT NULL,
            name TEXT,
            is_active INTEGER DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // =========================================================================
    // INDEX DE PERFORMANCE (Phase 3)
    // =========================================================================
    
    // Index existants
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_technical_id ON pending_reports(technical_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_patient_id ON pending_reports(patient_id)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_expires ON pending_reports(expires_at)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_accession ON pending_reports(accession_number)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_exam_uid ON pending_reports(exam_uid)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_status ON pending_reports(status)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(key_prefix)",
        [],
    )?;
    
    // 🆕 Nouveaux index de performance (Phase 3)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_pending_created_at ON pending_reports(created_at)",
        [],
    )?;
    
    // =========================================================================
    // Table des logs d'accès API (AUDIT)
    // =========================================================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS access_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            method TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            result TEXT NOT NULL CHECK (result IN ('success', 'unauthorized', 'not_found', 'error', 'bad_request')),
            api_key_prefix TEXT,
            user_agent TEXT,
            request_id TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            error_message TEXT
        )",
        [],
    )?;
    
    // Index pour les requêtes de recherche sur access_logs
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_access_logs_timestamp ON access_logs(timestamp)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_access_logs_endpoint ON access_logs(endpoint)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_access_logs_result ON access_logs(result)",
        [],
    )?;
    
    // 🆕 Nouveaux index de performance pour access_logs (Phase 3)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_access_logs_ip ON access_logs(ip_address)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_access_logs_timestamp_result ON access_logs(timestamp, result)",
        [],
    )?;
    
    // =========================================================================
    // Clé API de production - EXTERNALISÉE (Phase 1)
    // =========================================================================
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys",
        [],
        |row| row.get(0),
    )?;
    
    if count == 0 {
        // 🔐 SÉCURITÉ: Lire la clé depuis la variable d'environnement
        let (api_key, key_source) = match std::env::var("AIRADCR_PROD_API_KEY") {
            Ok(key) if !key.is_empty() => {
                println!("🔐 [Database] Clé API de production chargée depuis AIRADCR_PROD_API_KEY");
                (key, "env")
            }
            _ => {
                // Mode développement: générer une clé aléatoire
                let generated_key = generate_secure_api_key();
                println!("⚠️  [Database] ATTENTION: Aucune clé de production configurée!");
                println!("⚠️  [Database] Variable AIRADCR_PROD_API_KEY non définie");
                println!("🔑 [Database] Clé de développement générée: {}...", &generated_key[..16]);
                println!("💡 [Database] En production, définissez AIRADCR_PROD_API_KEY");
                (generated_key, "generated")
            }
        };
        
        let key_hash = hash_api_key(&api_key);
        let key_prefix = if api_key.len() >= 8 { 
            api_key[..8].to_string() 
        } else { 
            "airadcr_".to_string() 
        };
        
        let key_name = if key_source == "env" {
            "Production Key (from ENV)"
        } else {
            "Development Key (auto-generated)"
        };
        
        conn.execute(
            "INSERT INTO api_keys (id, key_prefix, key_hash, name, is_active, created_at)
             VALUES ('prod-key-1', ?1, ?2, ?3, 1, datetime('now'))",
            [&key_prefix, &key_hash, &key_name.to_string()],
        )?;
        
        println!("🔑 [Database] Clé API {} créée (prefix: {})", key_source, key_prefix);
    }
    
    Ok(())
}
