// ============================================================================
// AIRADCR Desktop - Système de Backup SQLite
// ============================================================================

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use chrono::{Utc, Duration};

/// Gère les backups automatiques de la base de données SQLite
pub struct BackupManager {
    db_path: PathBuf,
    backup_dir: PathBuf,
    retention_days: u32,
}

impl BackupManager {
    /// Crée un nouveau gestionnaire de backup
    pub fn new(db_path: PathBuf, retention_days: u32) -> Self {
        let backup_dir = db_path
            .parent()
            .unwrap_or(&db_path)
            .join("backups");
        
        Self {
            db_path,
            backup_dir,
            retention_days,
        }
    }
    
    /// Crée un backup de la base de données
    pub fn create_backup(&self) -> Result<PathBuf, String> {
        // Créer le répertoire de backup s'il n'existe pas
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| format!("Erreur création répertoire backup: {}", e))?;
        
        // Générer le nom du fichier avec timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("airadcr_backup_{}.db", timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);
        
        // Copier le fichier de base de données
        let mut source = File::open(&self.db_path)
            .map_err(|e| format!("Erreur ouverture source: {}", e))?;
        
        let mut content = Vec::new();
        source.read_to_end(&mut content)
            .map_err(|e| format!("Erreur lecture source: {}", e))?;
        
        let mut dest = File::create(&backup_path)
            .map_err(|e| format!("Erreur création backup: {}", e))?;
        
        dest.write_all(&content)
            .map_err(|e| format!("Erreur écriture backup: {}", e))?;
        
        println!("✅ [Backup] Créé: {:?} ({} bytes)", backup_path, content.len());
        
        // Vérifier l'intégrité du backup
        self.verify_backup(&backup_path)?;
        
        Ok(backup_path)
    }
    
    /// Vérifie l'intégrité d'un fichier backup
    fn verify_backup(&self, backup_path: &PathBuf) -> Result<(), String> {
        // Ouvrir le backup avec SQLite et vérifier l'intégrité
        let conn = rusqlite::Connection::open(backup_path)
            .map_err(|e| format!("Erreur ouverture backup pour vérification: {}", e))?;
        
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| format!("Erreur vérification intégrité: {}", e))?;
        
        if integrity != "ok" {
            return Err(format!("Backup corrompu: {}", integrity));
        }
        
        println!("✅ [Backup] Intégrité vérifiée: OK");
        Ok(())
    }
    
    /// Nettoie les backups anciens selon la rétention configurée
    pub fn cleanup_old_backups(&self) -> Result<u32, String> {
        if !self.backup_dir.exists() {
            return Ok(0);
        }
        
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        let mut deleted_count = 0u32;
        
        let entries = fs::read_dir(&self.backup_dir)
            .map_err(|e| format!("Erreur lecture répertoire backup: {}", e))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }
            
            // Vérifier si c'est un fichier de backup
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !name.starts_with("airadcr_backup_") || !name.ends_with(".db") {
                    continue;
                }
            }
            
            // Vérifier la date de modification
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_time = chrono::DateTime::<Utc>::from(modified);
                    
                    if modified_time < cutoff {
                        if fs::remove_file(&path).is_ok() {
                            println!("🗑️ [Backup] Supprimé ancien backup: {:?}", path);
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
        
        if deleted_count > 0 {
            println!("🧹 [Backup] {} backup(s) ancien(s) supprimé(s)", deleted_count);
        }
        
        Ok(deleted_count)
    }
    
    /// Liste tous les backups disponibles
    pub fn list_backups(&self) -> Vec<BackupInfo> {
        let mut backups = Vec::new();
        
        if !self.backup_dir.exists() {
            return backups;
        }
        
        if let Ok(entries) = fs::read_dir(&self.backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if !path.is_file() {
                    continue;
                }
                
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("airadcr_backup_") && name.ends_with(".db") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            let size = metadata.len();
                            let created = metadata.modified()
                                .ok()
                                .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339());
                            
                            backups.push(BackupInfo {
                                filename: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                                size_bytes: size,
                                created_at: created,
                            });
                        }
                    }
                }
            }
        }
        
        // Trier par date (plus récent en premier)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        backups
    }
    
    /// Restaure un backup spécifique
    pub fn restore_backup(&self, backup_filename: &str) -> Result<(), String> {
        let backup_path = self.backup_dir.join(backup_filename);
        
        if !backup_path.exists() {
            return Err(format!("Backup non trouvé: {}", backup_filename));
        }
        
        // Vérifier l'intégrité avant restauration
        self.verify_backup(&backup_path)?;
        
        // Créer un backup de sécurité avant restauration
        let safety_backup = self.db_path.with_extension("db.before_restore");
        fs::copy(&self.db_path, &safety_backup)
            .map_err(|e| format!("Erreur création backup de sécurité: {}", e))?;
        
        // Copier le backup vers la base principale
        fs::copy(&backup_path, &self.db_path)
            .map_err(|e| format!("Erreur restauration: {}", e))?;
        
        println!("✅ [Backup] Restauré depuis: {}", backup_filename);
        Ok(())
    }
}

/// Informations sur un backup
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: Option<String>,
}
