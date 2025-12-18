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
    // Identifiants patients (LOCAL UNIQUEMENT)
    pub patient_id: Option<String>,
    pub exam_uid: Option<String>,
    pub accession_number: Option<String>,
    pub study_instance_uid: Option<String>,
    // Données structurées
    pub structured_data: String,
    pub source_type: String,
    pub ai_modules: Option<String>,
    pub modality: Option<String>,
    pub metadata: Option<String>,
    // Statut
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub retrieved_at: Option<String>,
}

// ============================================================================
// Opérations CRUD sur les rapports
// ============================================================================

/// Insère un nouveau rapport en attente (avec identifiants patients pour LOCAL)
pub fn insert_pending_report(
    conn: &Connection,
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
    // Utiliser INSERT OR REPLACE pour écraser un rapport existant avec le même technical_id
    conn.execute(
        "INSERT OR REPLACE INTO pending_reports 
         (id, technical_id, patient_id, exam_uid, accession_number, study_instance_uid,
          structured_data, source_type, ai_modules, modality, metadata, status, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'pending', ?12, ?13)",
        params![id, technical_id, patient_id, exam_uid, accession_number, study_instance_uid,
                structured_data, source_type, ai_modules, modality, metadata, created_at, expires_at],
    )?;
    
    Ok(())
}

/// Récupère un rapport par son technical_id (uniquement si non expiré)
pub fn get_pending_report_by_tid(conn: &Connection, technical_id: &str) -> SqlResult<Option<PendingReport>> {
    let mut stmt = conn.prepare(
        "SELECT id, technical_id, patient_id, exam_uid, accession_number, study_instance_uid,
                structured_data, source_type, ai_modules, modality, metadata, status, created_at, expires_at, retrieved_at
         FROM pending_reports
         WHERE technical_id = ?1 AND status != 'expired' AND expires_at > datetime('now')"
    )?;
    
    let result = stmt.query_row([technical_id], |row| {
        Ok(PendingReport {
            id: row.get(0)?,
            technical_id: row.get(1)?,
            patient_id: row.get(2)?,
            exam_uid: row.get(3)?,
            accession_number: row.get(4)?,
            study_instance_uid: row.get(5)?,
            structured_data: row.get(6)?,
            source_type: row.get(7)?,
            ai_modules: row.get(8)?,
            modality: row.get(9)?,
            metadata: row.get(10)?,
            status: row.get(11)?,
            created_at: row.get(12)?,
            expires_at: row.get(13)?,
            retrieved_at: row.get(14)?,
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

/// Recherche un rapport par identifiants RIS (patient_id, accession_number, exam_uid)
/// Retourne le premier rapport correspondant non expiré
pub fn find_pending_report_by_identifiers(
    conn: &Connection,
    patient_id: Option<&str>,
    accession_number: Option<&str>,
    exam_uid: Option<&str>,
) -> SqlResult<Option<PendingReport>> {
    // Construire la requête dynamiquement selon les paramètres fournis
    let mut conditions = Vec::new();
    let mut param_values: Vec<String> = Vec::new();
    
    if let Some(acc) = accession_number {
        if !acc.is_empty() {
            conditions.push("accession_number = ?");
            param_values.push(acc.to_string());
        }
    }
    
    if let Some(exam) = exam_uid {
        if !exam.is_empty() {
            conditions.push("exam_uid = ?");
            param_values.push(exam.to_string());
        }
    }
    
    if let Some(pat) = patient_id {
        if !pat.is_empty() {
            conditions.push("patient_id = ?");
            param_values.push(pat.to_string());
        }
    }
    
    if conditions.is_empty() {
        return Ok(None);
    }
    
    let where_clause = conditions.join(" OR ");
    let sql = format!(
        "SELECT id, technical_id, patient_id, exam_uid, accession_number, study_instance_uid,
                structured_data, source_type, ai_modules, modality, metadata, status, created_at, expires_at, retrieved_at
         FROM pending_reports
         WHERE ({}) AND status != 'expired' AND expires_at > datetime('now')
         ORDER BY created_at DESC
         LIMIT 1",
        where_clause
    );
    
    let mut stmt = conn.prepare(&sql)?;
    
    // Convertir les paramètres en références pour rusqlite
    let params: Vec<&dyn rusqlite::ToSql> = param_values.iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();
    
    let result = stmt.query_row(params.as_slice(), |row| {
        Ok(PendingReport {
            id: row.get(0)?,
            technical_id: row.get(1)?,
            patient_id: row.get(2)?,
            exam_uid: row.get(3)?,
            accession_number: row.get(4)?,
            study_instance_uid: row.get(5)?,
            structured_data: row.get(6)?,
            source_type: row.get(7)?,
            ai_modules: row.get(8)?,
            modality: row.get(9)?,
            metadata: row.get(10)?,
            status: row.get(11)?,
            created_at: row.get(12)?,
            expires_at: row.get(13)?,
            retrieved_at: row.get(14)?,
        })
    });
    
    match result {
        Ok(report) => Ok(Some(report)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
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

/// Liste toutes les clés API
pub fn list_api_keys(conn: &Connection) -> SqlResult<Vec<(String, String, String, bool, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, key_prefix, is_active, created_at FROM api_keys ORDER BY created_at DESC"
    )?;
    
    let keys = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)? == 1,
            row.get::<_, String>(4)?,
        ))
    })?
    .collect::<SqlResult<Vec<_>>>()?;
    
    Ok(keys)
}

/// Révoque une clé API (soft-delete: is_active = 0)
pub fn revoke_api_key(conn: &Connection, key_prefix: &str) -> SqlResult<bool> {
    let rows = conn.execute(
        "UPDATE api_keys SET is_active = 0 WHERE key_prefix = ?1 AND is_active = 1",
        [key_prefix],
    )?;
    
    Ok(rows > 0)
}

// ============================================================================
// Nouvelles fonctions pour le Debug Panel
// ============================================================================

/// Structure pour les statistiques de la base
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub total_reports: i64,
    pub pending_reports: i64,
    pub retrieved_reports: i64,
    pub expired_reports: i64,
    pub active_api_keys: i64,
    pub revoked_api_keys: i64,
}

/// Structure simplifiée pour affichage des rapports
#[derive(Debug, Clone, serde::Serialize)]
pub struct PendingReportSummary {
    pub technical_id: String,
    pub accession_number: Option<String>,
    pub patient_id: Option<String>,
    pub modality: Option<String>,
    pub status: String,
    pub source_type: String,
    pub created_at: String,
    pub expires_at: String,
}

/// Structure simplifiée pour affichage des clés API
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiKeySummary {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub created_at: String,
}

/// Liste tous les rapports en attente (pour le Debug Panel)
pub fn list_all_pending_reports(conn: &Connection) -> SqlResult<Vec<PendingReportSummary>> {
    let mut stmt = conn.prepare(
        "SELECT technical_id, accession_number, patient_id, modality, status, source_type, created_at, expires_at
         FROM pending_reports
         ORDER BY created_at DESC
         LIMIT 100"
    )?;
    
    let reports = stmt.query_map([], |row| {
        Ok(PendingReportSummary {
            technical_id: row.get(0)?,
            accession_number: row.get(1)?,
            patient_id: row.get(2)?,
            modality: row.get(3)?,
            status: row.get(4)?,
            source_type: row.get(5)?,
            created_at: row.get(6)?,
            expires_at: row.get(7)?,
        })
    })?
    .collect::<SqlResult<Vec<_>>>()?;
    
    Ok(reports)
}

/// Récupère les statistiques de la base de données
pub fn get_database_stats(conn: &Connection) -> SqlResult<DatabaseStats> {
    let total_reports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_reports",
        [],
        |row| row.get(0),
    )?;
    
    let pending_reports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_reports WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    
    let retrieved_reports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_reports WHERE status = 'retrieved'",
        [],
        |row| row.get(0),
    )?;
    
    let expired_reports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_reports WHERE status = 'expired' OR expires_at < datetime('now')",
        [],
        |row| row.get(0),
    )?;
    
    let active_api_keys: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys WHERE is_active = 1",
        [],
        |row| row.get(0),
    )?;
    
    let revoked_api_keys: i64 = conn.query_row(
        "SELECT COUNT(*) FROM api_keys WHERE is_active = 0",
        [],
        |row| row.get(0),
    )?;
    
    Ok(DatabaseStats {
        total_reports,
        pending_reports,
        retrieved_reports,
        expired_reports,
        active_api_keys,
        revoked_api_keys,
    })
}

/// Liste toutes les clés API avec infos simplifiées (sans hash)
pub fn list_api_keys_summary(conn: &Connection) -> SqlResult<Vec<ApiKeySummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, key_prefix, is_active, created_at FROM api_keys ORDER BY created_at DESC"
    )?;
    
    let keys = stmt.query_map([], |row| {
        Ok(ApiKeySummary {
            id: row.get(0)?,
            name: row.get(1)?,
            key_prefix: row.get(2)?,
            is_active: row.get::<_, i32>(3)? == 1,
            created_at: row.get(4)?,
        })
    })?
    .collect::<SqlResult<Vec<_>>>()?;
    
    Ok(keys)
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
    fn test_insert_and_get_report_with_patient_ids() {
        let conn = setup_test_db();
        
        insert_pending_report(
            &conn,
            "test-id-1",
            "TEST_001",
            Some("PAT123456"),           // patient_id - ACCEPTÉ EN LOCAL
            Some("1.2.3.4.5.6.7.8.9"),    // exam_uid
            Some("ACC2024001"),           // accession_number
            Some("1.2.840.xxx"),          // study_instance_uid
            r#"{"title": "Test IRM"}"#,
            "ris_local",
            Some(r#"["module1"]"#),
            Some("MR"),
            Some(r#"{"priority": "routine"}"#),
            "2025-12-15T10:00:00Z",
            "2099-12-31T23:59:59Z",
        ).unwrap();
        
        let result = get_pending_report_by_tid(&conn, "TEST_001").unwrap();
        assert!(result.is_some());
        
        let report = result.unwrap();
        assert_eq!(report.technical_id, "TEST_001");
        assert_eq!(report.patient_id, Some("PAT123456".to_string()));
        assert_eq!(report.exam_uid, Some("1.2.3.4.5.6.7.8.9".to_string()));
        assert_eq!(report.accession_number, Some("ACC2024001".to_string()));
        assert_eq!(report.source_type, "ris_local");
        assert_eq!(report.modality, Some("MR".to_string()));
    }
    
    #[test]
    fn test_delete_report() {
        let conn = setup_test_db();
        
        insert_pending_report(
            &conn,
            "test-id-2",
            "TEST_002",
            None, None, None, None,  // Pas d'identifiants patients
            r#"{"title": "Test"}"#,
            "test",
            None, None, None,
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
            Some("PAT999"),
            None, None, None,
            r#"{"title": "Expired"}"#,
            "test",
            None, None, None,
            "2020-01-01T10:00:00Z",
            "2020-01-02T10:00:00Z",
        ).unwrap();
        
        let result = get_pending_report_by_tid(&conn, "TEST_003").unwrap();
        assert!(result.is_none()); // Ne devrait pas être retourné car expiré
    }
}
