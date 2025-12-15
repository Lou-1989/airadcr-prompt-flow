// ============================================================================
// AIRADCR Desktop - Requêtes SQLite
// ============================================================================

use rusqlite::{Connection, Result as SqlResult, params};
use chrono::Utc;

// ============================================================================
// Structures de données
// ============================================================================

/// Représente un rapport en attente dans la base
#[derive(Debug, Clone)]
pub struct PendingReport {
    pub id: String,
    pub technical_id: String,
    pub structured_data: String,
    pub source_type: String,
    pub ai_modules: Option<String>,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub retrieved_at: Option<String>,
}

// ============================================================================
// Opérations CRUD sur les rapports
// ============================================================================

/// Insère un nouveau rapport en attente
pub fn insert_pending_report(
    conn: &Connection,
    id: &str,
    technical_id: &str,
    structured_data: &str,
    source_type: &str,
    ai_modules: Option<&str>,
    created_at: &str,
    expires_at: &str,
) -> SqlResult<()> {
    // Utiliser INSERT OR REPLACE pour écraser un rapport existant avec le même technical_id
    conn.execute(
        "INSERT OR REPLACE INTO pending_reports 
         (id, technical_id, structured_data, source_type, ai_modules, status, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
        params![id, technical_id, structured_data, source_type, ai_modules, created_at, expires_at],
    )?;
    
    Ok(())
}

/// Récupère un rapport par son technical_id (uniquement si non expiré)
pub fn get_pending_report_by_tid(conn: &Connection, technical_id: &str) -> SqlResult<Option<PendingReport>> {
    let mut stmt = conn.prepare(
        "SELECT id, technical_id, structured_data, source_type, ai_modules, status, created_at, expires_at, retrieved_at
         FROM pending_reports
         WHERE technical_id = ?1 AND status != 'expired' AND expires_at > datetime('now')"
    )?;
    
    let result = stmt.query_row([technical_id], |row| {
        Ok(PendingReport {
            id: row.get(0)?,
            technical_id: row.get(1)?,
            structured_data: row.get(2)?,
            source_type: row.get(3)?,
            ai_modules: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
            expires_at: row.get(7)?,
            retrieved_at: row.get(8)?,
        })
    });
    
    match result {
        Ok(report) => Ok(Some(report)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Marque un rapport comme récupéré
pub fn mark_as_retrieved(conn: &Connection, technical_id: &str) -> SqlResult<bool> {
    let now = Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE pending_reports SET status = 'retrieved', retrieved_at = ?1 WHERE technical_id = ?2",
        params![now, technical_id],
    )?;
    
    Ok(rows > 0)
}

/// Supprime un rapport
pub fn delete_pending_report(conn: &Connection, technical_id: &str) -> SqlResult<bool> {
    let rows = conn.execute(
        "DELETE FROM pending_reports WHERE technical_id = ?1",
        [technical_id],
    )?;
    
    Ok(rows > 0)
}

/// Nettoie les rapports expirés
pub fn cleanup_expired_reports(conn: &Connection) -> SqlResult<usize> {
    let rows = conn.execute(
        "DELETE FROM pending_reports WHERE expires_at < datetime('now') OR status = 'expired'",
        [],
    )?;
    
    if rows > 0 {
        println!("🧹 [Database] {} rapport(s) expiré(s) supprimé(s)", rows);
    }
    
    Ok(rows)
}

// ============================================================================
// Opérations sur les clés API
// ============================================================================

/// Valide une clé API
pub fn validate_api_key(conn: &Connection, key_prefix: &str, key_hash: &str) -> SqlResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys WHERE key_prefix = ?1 AND key_hash = ?2 AND is_active = 1",
        params![key_prefix, key_hash],
        |row| row.get(0),
    )?;
    
    Ok(count > 0)
}

/// Ajoute une nouvelle clé API
pub fn add_api_key(
    conn: &Connection,
    id: &str,
    key_prefix: &str,
    key_hash: &str,
    name: &str,
) -> SqlResult<()> {
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO api_keys (id, key_prefix, key_hash, name, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        params![id, key_prefix, key_hash, name, now],
    )?;
    
    Ok(())
}

// ============================================================================
// Tests unitaires
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;
    
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        conn
    }
    
    #[test]
    fn test_insert_and_get_report() {
        let conn = setup_test_db();
        
        insert_pending_report(
            &conn,
            "test-id-1",
            "TEST_001",
            r#"{"title": "Test IRM"}"#,
            "test",
            Some(r#"["module1"]"#),
            "2025-12-15T10:00:00Z",
            "2099-12-31T23:59:59Z", // Loin dans le futur pour ne pas expirer
        ).unwrap();
        
        let result = get_pending_report_by_tid(&conn, "TEST_001").unwrap();
        assert!(result.is_some());
        
        let report = result.unwrap();
        assert_eq!(report.technical_id, "TEST_001");
        assert_eq!(report.source_type, "test");
    }
    
    #[test]
    fn test_delete_report() {
        let conn = setup_test_db();
        
        insert_pending_report(
            &conn,
            "test-id-2",
            "TEST_002",
            r#"{"title": "Test"}"#,
            "test",
            None,
            "2025-12-15T10:00:00Z",
            "2099-12-31T23:59:59Z",
        ).unwrap();
        
        let deleted = delete_pending_report(&conn, "TEST_002").unwrap();
        assert!(deleted);
        
        let result = get_pending_report_by_tid(&conn, "TEST_002").unwrap();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_expired_report_not_returned() {
        let conn = setup_test_db();
        
        insert_pending_report(
            &conn,
            "test-id-3",
            "TEST_003",
            r#"{"title": "Expired"}"#,
            "test",
            None,
            "2020-01-01T10:00:00Z",
            "2020-01-02T10:00:00Z", // Expiré dans le passé
        ).unwrap();
        
        let result = get_pending_report_by_tid(&conn, "TEST_003").unwrap();
        assert!(result.is_none()); // Ne devrait pas être retourné car expiré
    }
}
