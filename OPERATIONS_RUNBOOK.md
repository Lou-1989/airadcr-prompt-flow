# 📘 AIRADCR Desktop - Runbook Opérationnel

## Table des matières

1. [Démarrage et Arrêt](#démarrage-et-arrêt)
2. [Configuration](#configuration)
3. [Gestion des Incidents](#gestion-des-incidents)
4. [Backup et Restauration](#backup-et-restauration)
5. [Gestion des Clés API](#gestion-des-clés-api)
6. [Monitoring](#monitoring)
7. [Troubleshooting](#troubleshooting)

---

## Démarrage et Arrêt

### Démarrage Normal

L'application démarre automatiquement avec Windows si configurée. Sinon :

1. **Via le menu Démarrer** : Rechercher "AIRADCR" et lancer
2. **Via le raccourci Bureau** : Double-cliquer sur l'icône AIRADCR
3. **Via ligne de commande** : `"C:\Program Files\AIRADCR\AIRADCR.exe"`

### Vérification du Démarrage

```powershell
# Vérifier que le serveur HTTP répond
curl http://localhost:8741/health

# Réponse attendue
{"status":"ok","timestamp":"...","version":"1.0.0"}
```

### Arrêt

1. **Via System Tray** : Clic droit sur l'icône → "Quitter"
2. **Fermeture fenêtre** : La fenêtre se minimise dans le tray
3. **Forcer l'arrêt** : `taskkill /F /IM AIRADCR.exe`

---

## Configuration

### Fichier de Configuration

Emplacement : `%APPDATA%\airadcr-desktop\config.toml`

```toml
# Port du serveur HTTP
http_port = 8741

# Niveau de log (error, warn, info, debug, trace)
log_level = "info"

# Rétention des logs d'accès (jours)
log_retention_days = 30

# Rétention des rapports expirés (heures)
report_retention_hours = 24

# URL de l'iframe AIRADCR
iframe_url = "https://airadcr.com"

# Backup automatique SQLite
backup_enabled = true
backup_retention_days = 7

# Intervalle de cleanup (secondes)
cleanup_interval_secs = 3600
```

### Variables d'Environnement

| Variable | Description | Obligatoire |
|----------|-------------|-------------|
| `AIRADCR_PROD_API_KEY` | Clé API de production | Oui (prod) |
| `AIRADCR_ADMIN_KEY` | Clé admin pour gestion API keys | Non |
| `AIRADCR_ENV` | Environnement (production/dev) | Non |

### Configurer les Variables (Windows)

```powershell
# Configuration système (redémarrage requis)
[System.Environment]::SetEnvironmentVariable("AIRADCR_PROD_API_KEY", "votre_cle_secrete", "Machine")

# Configuration utilisateur
[System.Environment]::SetEnvironmentVariable("AIRADCR_PROD_API_KEY", "votre_cle_secrete", "User")
```

---

## Gestion des Incidents

### Incident : Port 8741 Occupé

**Symptômes** : L'application ne démarre pas, erreur "Address already in use"

**Diagnostic** :
```powershell
netstat -ano | findstr :8741
```

**Résolution** :
1. Identifier le processus : `tasklist /FI "PID eq <PID>"`
2. Terminer le processus : `taskkill /F /PID <PID>`
3. Ou modifier le port dans `config.toml`

L'application essaie automatiquement les ports 8741, 8742, 8743.

### Incident : Base de Données Corrompue

**Symptômes** : Erreurs SQLite, données manquantes

**Diagnostic** :
```powershell
# Vérifier l'intégrité via SQLite CLI
sqlite3 "%APPDATA%\airadcr-desktop\airadcr.db" "PRAGMA integrity_check;"
```

**Résolution** :
1. Restaurer depuis un backup (voir section Backup)
2. Si aucun backup : supprimer `airadcr.db`, l'app recréera une base vide

### Incident : Injection Non Fonctionnelle

**Symptômes** : Le texte ne s'injecte pas dans le RIS

**Diagnostic** :
1. Vérifier que l'application cible a le focus
2. Vérifier les coordonnées du curseur (Ctrl+Alt+D pour Debug Panel)

**Résolution** :
1. Vérifier les droits administrateur (certains RIS nécessitent élévation)
2. Utiliser le raccourci F9 pour désactiver le mode "click-through"
3. Verrouiller la cible d'injection dans les paramètres

### Incident : Clé API Compromise

**Symptômes** : Accès non autorisés dans les logs

**Résolution IMMÉDIATE** :
1. Révoquer la clé via l'API admin ou le Debug Panel
2. Générer une nouvelle clé
3. Mettre à jour les systèmes clients (RIS, TÉO Hub)
4. Analyser les logs d'accès pour évaluer l'impact

---

## Backup et Restauration

### Backups Automatiques

- **Emplacement** : `%APPDATA%\airadcr-desktop\backups\`
- **Fréquence** : Quotidien (configurable)
- **Rétention** : 7 jours par défaut

### Backup Manuel

Via le Debug Panel (Ctrl+Alt+D) → Onglet "Database" → "Créer Backup"

### Restauration

1. Arrêter l'application
2. Localiser le backup : `%APPDATA%\airadcr-desktop\backups\`
3. Copier le fichier de backup vers `airadcr.db`
4. Redémarrer l'application

```powershell
# Exemple PowerShell
$backupDir = "$env:APPDATA\airadcr-desktop\backups"
$dbPath = "$env:APPDATA\airadcr-desktop\airadcr.db"

# Lister les backups disponibles
Get-ChildItem $backupDir -Filter "*.db" | Sort-Object LastWriteTime -Descending

# Restaurer le plus récent
Copy-Item "$backupDir\airadcr_backup_YYYYMMDD_HHMMSS.db" $dbPath -Force
```

---

## Gestion des Clés API

### Créer une Clé API

**Via API** (nécessite clé admin) :
```bash
curl -X POST http://localhost:8741/api-keys \
  -H "Content-Type: application/json" \
  -H "X-Admin-Key: votre_cle_admin" \
  -d '{"name": "RIS Integration"}'
```

**Via Debug Panel** : Ctrl+Alt+D → Onglet "API Keys" → "Nouvelle Clé"

### Révoquer une Clé API

```bash
curl -X DELETE http://localhost:8741/api-keys/airadcr_ \
  -H "X-Admin-Key: votre_cle_admin"
```

### Rotation des Clés

1. Créer une nouvelle clé
2. Mettre à jour les systèmes clients
3. Tester la nouvelle clé
4. Révoquer l'ancienne clé

---

## Monitoring

### Endpoint Prometheus

```
GET http://localhost:8741/metrics
```

Métriques exposées :
- `airadcr_requests_total` - Nombre total de requêtes
- `airadcr_requests_success_total` - Requêtes réussies
- `airadcr_requests_error_total` - Requêtes en erreur
- `airadcr_uptime_seconds` - Uptime du serveur
- `airadcr_pending_reports_count` - Rapports en attente
- `airadcr_api_keys_active_count` - Clés API actives
- `airadcr_db_size_bytes` - Taille de la base de données

### Health Check Étendu

```
GET http://localhost:8741/health/extended
```

Réponse :
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "database": {
    "status": "ok",
    "pending_reports": 5,
    "api_keys_active": 3,
    "size_bytes": 102400
  },
  "requests": {
    "total": 1500,
    "success": 1480,
    "errors": 20,
    "avg_duration_ms": 12.5
  }
}
```

### Logs d'Accès

Les logs d'accès sont stockés dans la table `access_logs` de SQLite.

Consultation via Debug Panel → Onglet "Access Logs"

---

## Troubleshooting

### Logs de l'Application

**Emplacement** : `%APPDATA%\airadcr-desktop\logs\`

**Consulter les derniers logs** :
```powershell
Get-Content "$env:APPDATA\airadcr-desktop\logs\airadcr.log" -Tail 100
```

**Activer le mode debug** :
1. Modifier `config.toml` : `log_level = "debug"`
2. Ou définir `localStorage.setItem('airadcr_debug', 'true')` dans la console

### Raccourcis de Debug

| Raccourci | Action |
|-----------|--------|
| Ctrl+Alt+D | Ouvrir Debug Panel |
| Ctrl+Alt+L | Ouvrir fenêtre de logs |
| Ctrl+Alt+I | Test d'injection |
| F9 | Désactiver click-through (anti-fantôme) |

### Vérifications Courantes

```powershell
# 1. Vérifier que l'application tourne
Get-Process -Name "AIRADCR" -ErrorAction SilentlyContinue

# 2. Vérifier le port HTTP
Test-NetConnection -ComputerName localhost -Port 8741

# 3. Vérifier la santé de l'API
Invoke-RestMethod -Uri "http://localhost:8741/health"

# 4. Vérifier la taille de la base de données
(Get-Item "$env:APPDATA\airadcr-desktop\airadcr.db").Length / 1MB
```

### Contact Support

En cas de problème non résolu :
1. Exporter les logs depuis Debug Panel
2. Capturer les métriques (`/health/extended`)
3. Contacter support@airadcr.com avec ces informations

---

## Changelog

| Version | Date | Changements |
|---------|------|-------------|
| 1.0.0 | 2025-01 | Version initiale |
