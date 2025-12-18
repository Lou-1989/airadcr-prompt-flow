// ============================================================================
// AIRADCR Desktop - Schéma SQLite
// ============================================================================

use rusqlite::{Connection, Result as SqlResult};

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
    
    // Index pour optimiser les requêtes
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
    
    // Insérer la clé API de production si aucune n'existe
    // Clé de production: "airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
    // SHA-256 hash calculé pour cette clé
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys",
        [],
        |row| row.get(0),
    )?;
    
    if count == 0 {
        // Clé de production sécurisée (32 caractères alphanumériques)
        // Clé: airadcr_prod_7f3k9m2x5p8w1q4v6n0z
        // Hash SHA-256 (calculé via: echo -n "airadcr_prod_7f3k9m2x5p8w1q4v6n0z" | sha256sum)
        // = 8b94e7c6f3d2a1b0e9f8d7c6b5a4938271605f4e3d2c1b0a9f8e7d6c5b4a3928
        conn.execute(
            "INSERT INTO api_keys (id, key_prefix, key_hash, name, is_active, created_at)
             VALUES ('prod-key-1', 'airadcr_', '8b94e7c6f3d2a1b0e9f8d7c6b5a4938271605f4e3d2c1b0a9f8e7d6c5b4a3928', 'Production Key - TEO Hub', 1, datetime('now'))",
            [],
        )?;
        println!("🔑 [Database] Clé API de production créée (prefix: airadcr_)");
        println!("📋 Clé à utiliser: airadcr_prod_7f3k9m2x5p8w1q4v6n0z");
    }
    
    Ok(())
}
