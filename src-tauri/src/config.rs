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

/// Configuration TÉO Hub Client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeoHubConfig {
    /// Activer le client TÉO Hub
    #[serde(default = "default_teo_enabled")]
    pub enabled: bool,
    
    /// Adresse du serveur TÉO Hub
    #[serde(default = "default_teo_host")]
    pub host: String,
    
    /// Port du serveur TÉO Hub
    #[serde(default = "default_teo_port")]
    pub port: u16,
    
    /// Endpoint health check
    #[serde(default = "default_teo_health_endpoint")]
    pub health_endpoint: String,
    
    /// Endpoint GET rapport IA
    #[serde(default = "default_teo_get_report_endpoint")]
    pub get_report_endpoint: String,
    
    /// Endpoint POST rapport approuvé
    #[serde(default = "default_teo_post_report_endpoint")]
    pub post_report_endpoint: String,
    
    /// Timeout en secondes
    #[serde(default = "default_teo_timeout")]
    pub timeout_secs: u64,
    
    /// Nombre de retries
    #[serde(default = "default_teo_retry_count")]
    pub retry_count: u32,
    
    /// Délai entre retries (ms)
    #[serde(default = "default_teo_retry_delay")]
    pub retry_delay_ms: u64,
    
    /// Activer TLS
    #[serde(default)]
    pub tls_enabled: bool,
    
    /// Fichier certificat CA
    #[serde(default)]
    pub ca_file: String,
    
    /// Fichier certificat client
    #[serde(default)]
    pub cert_file: String,
    
    /// Fichier clé privée client
    #[serde(default)]
    pub key_file: String,
    
    /// API Key pour authentification
    #[serde(default)]
    pub api_key: String,
    
    /// Bearer token pour authentification
    #[serde(default)]
    pub bearer_token: String,
}

fn default_teo_enabled() -> bool { false }
fn default_teo_host() -> String { "192.168.1.36".to_string() }
fn default_teo_port() -> u16 { 54489 }
fn default_teo_health_endpoint() -> String { "th_health".to_string() }
fn default_teo_get_report_endpoint() -> String { "th_get_ai_report".to_string() }
fn default_teo_post_report_endpoint() -> String { "th_post_approved_report".to_string() }
fn default_teo_timeout() -> u64 { 30 }
fn default_teo_retry_count() -> u32 { 3 }
fn default_teo_retry_delay() -> u64 { 1000 }

impl Default for TeoHubConfig {
    fn default() -> Self {
        Self {
            enabled: default_teo_enabled(),
            host: default_teo_host(),
            port: default_teo_port(),
            health_endpoint: default_teo_health_endpoint(),
            get_report_endpoint: default_teo_get_report_endpoint(),
            post_report_endpoint: default_teo_post_report_endpoint(),
            timeout_secs: default_teo_timeout(),
            retry_count: default_teo_retry_count(),
            retry_delay_ms: default_teo_retry_delay(),
            tls_enabled: false,
            ca_file: String::new(),
            cert_file: String::new(),
            key_file: String::new(),
            api_key: String::new(),
            bearer_token: String::new(),
        }
    }
}

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
    
    /// Configuration TÉO Hub Client
    #[serde(default)]
    pub teo_hub: TeoHubConfig,
}

fn default_http_port() -> u16 { 8741 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_retention_days() -> u32 { 30 }
fn default_report_retention_hours() -> u32 { 24 }
fn default_iframe_url() -> String { "https://airadcr.com/app?tori=true".to_string() }
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
            teo_hub: TeoHubConfig::default(),
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
                if let Ok(mut config) = toml::from_str::<AppConfig>(&content) {
                        // Migration automatique : corriger l'ancienne URL sans ?tori=true
                        if config.iframe_url == "https://airadcr.com/app" || config.iframe_url == "https://airadcr.com" {
                            println!("🔄 [Config] Migration: ancienne iframe_url détectée, mise à jour vers ?tori=true");
                            config.iframe_url = "https://airadcr.com/app?tori=true".to_string();
                            let _ = config.save();
                        }
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
