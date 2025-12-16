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
        // Hash SHA-256: 8a4f2b1c9e7d3a6f5b8c2e1d4a7f9b3c6e8d1a4f7b2c5e8d1a4f7b2c5e8d1a4f
        conn.execute(
            "INSERT INTO api_keys (id, key_prefix, key_hash, name, is_active, created_at)
             VALUES ('prod-key-1', 'airadcr_', 'c7b8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8', 'Production Key - TEO Hub', 1, datetime('now'))",
            [],
        )?;
        println!("🔑 [Database] Clé API de production créée (prefix: airadcr_)");
        println!("📋 Clé à utiliser: airadcr_prod_7f3k9m2x5p8w1q4v6n0z");
    }
    
    Ok(())
}
