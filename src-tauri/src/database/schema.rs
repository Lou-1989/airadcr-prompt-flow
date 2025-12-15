// ============================================================================
// AIRADCR Desktop - Schéma SQLite
// ============================================================================

use rusqlite::{Connection, Result as SqlResult};

/// Initialise le schéma de la base de données
pub fn initialize(conn: &Connection) -> SqlResult<()> {
    // Table des rapports en attente
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pending_reports (
            id TEXT PRIMARY KEY,
            technical_id TEXT UNIQUE NOT NULL,
            structured_data TEXT NOT NULL,
            source_type TEXT DEFAULT 'tauri_local',
            ai_modules TEXT,
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
        "CREATE INDEX IF NOT EXISTS idx_pending_expires ON pending_reports(expires_at)",
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
    
    // Insérer une clé API de test si aucune n'existe (pour le développement)
    // Clé: "airadcr_dev_key_2024" → Hash SHA-256
    // IMPORTANT: En production, remplacer par des vraies clés
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys",
        [],
        |row| row.get(0),
    )?;
    
    if count == 0 {
        // Clé de développement par défaut
        // airadcr_dev_key_2024 → SHA-256 = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        // Note: Vous devez générer le vrai hash de votre clé de dev
        conn.execute(
            "INSERT INTO api_keys (id, key_prefix, key_hash, name, is_active, created_at)
             VALUES ('dev-key-1', 'airadcr_', 'a5e744d0164540d33b1d7ea616c28f2fa97b94c8e895a05e7ad9e66a3e16d3eb', 'Development Key', 1, datetime('now'))",
            [],
        )?;
        println!("🔑 [Database] Clé API de développement créée (prefix: airadcr_)");
    }
    
    Ok(())
}
