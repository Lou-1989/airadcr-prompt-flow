// ============================================================================
// AIRADCR Desktop - Configuration Externalisée
// ============================================================================
// Gère les paramètres configurables via fichier TOML et variables d'environnement
// ============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Configuration de l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Port du serveur HTTP local (défaut: 8741)
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    
    /// Niveau de log (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Rétention des logs d'accès en jours (défaut: 30)
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u32,
    
    /// Rétention des rapports expirés en heures (défaut: 24)
    #[serde(default = "default_report_retention_hours")]
    pub report_retention_hours: u32,
    
    /// URL de l'iframe AIRADCR (défaut: https://airadcr.com)
    #[serde(default = "default_iframe_url")]
    pub iframe_url: String,
    
    /// Activer les backups automatiques SQLite
    #[serde(default = "default_backup_enabled")]
    pub backup_enabled: bool,
    
    /// Nombre de jours de rétention des backups
    #[serde(default = "default_backup_retention_days")]
    pub backup_retention_days: u32,
    
    /// Intervalle de cleanup en secondes (défaut: 3600 = 1h)
    #[serde(default = "default_cleanup_interval_secs")]
    pub cleanup_interval_secs: u64,
}

fn default_http_port() -> u16 { 8741 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_retention_days() -> u32 { 30 }
fn default_report_retention_hours() -> u32 { 24 }
fn default_iframe_url() -> String { "https://airadcr.com".to_string() }
fn default_backup_enabled() -> bool { true }
fn default_backup_retention_days() -> u32 { 7 }
fn default_cleanup_interval_secs() -> u64 { 3600 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http_port: default_http_port(),
            log_level: default_log_level(),
            log_retention_days: default_log_retention_days(),
            report_retention_hours: default_report_retention_hours(),
            iframe_url: default_iframe_url(),
            backup_enabled: default_backup_enabled(),
            backup_retention_days: default_backup_retention_days(),
            cleanup_interval_secs: default_cleanup_interval_secs(),
        }
    }
}

impl AppConfig {
    /// Chemin du fichier de configuration
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("airadcr-desktop").join("config.toml"))
    }
    
    /// Charge la configuration depuis le fichier TOML ou utilise les valeurs par défaut
    pub fn load() -> Self {
        // 1. Essayer de charger depuis le fichier
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                        println!("📁 [Config] Chargé depuis {:?}", path);
                        return config;
                    } else {
                        eprintln!("⚠️ [Config] Erreur parsing {:?}, utilisation des valeurs par défaut", path);
                    }
                }
            }
        }
        
        // 2. Utiliser les valeurs par défaut
        println!("📁 [Config] Utilisation de la configuration par défaut");
        Self::default()
    }
    
    /// Sauvegarde la configuration dans le fichier TOML
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()
            .ok_or_else(|| "Impossible de déterminer le chemin de configuration".to_string())?;
        
        // Créer le répertoire parent si nécessaire
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Erreur création répertoire: {}", e))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Erreur sérialisation: {}", e))?;
        
        fs::write(&path, content)
            .map_err(|e| format!("Erreur écriture: {}", e))?;
        
        println!("💾 [Config] Sauvegardé dans {:?}", path);
        Ok(())
    }
    
    /// Crée un fichier de configuration par défaut s'il n'existe pas
    pub fn ensure_config_file() {
        if let Some(path) = Self::config_path() {
            if !path.exists() {
                let default_config = Self::default();
                if default_config.save().is_ok() {
                    println!("✅ [Config] Fichier de configuration créé: {:?}", path);
                }
            }
        }
    }
}

/// Obtient la configuration globale (thread-safe)
pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(AppConfig::load)
}

/// Génère la clé API de production depuis l'environnement
pub fn get_production_api_key() -> Option<String> {
    std::env::var("AIRADCR_PROD_API_KEY").ok()
}

/// Génère la clé admin depuis l'environnement
pub fn get_admin_key() -> Option<String> {
    std::env::var("AIRADCR_ADMIN_KEY").ok()
}

/// Vérifie si on est en mode production
pub fn is_production() -> bool {
    std::env::var("AIRADCR_ENV")
        .map(|v| v.to_lowercase() == "production" || v.to_lowercase() == "prod")
        .unwrap_or(false)
}

// Dépendance TOML pour parsing config
// Note: Ajouté dans Cargo.toml
