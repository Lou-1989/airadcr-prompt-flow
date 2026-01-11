# 🔐 Configuration Sécurité AIRADCR Desktop

## 📋 Vue d'ensemble

Ce document décrit les configurations de sécurité requises pour déployer AIRADCR Desktop en production.

---

## 🔑 Clé Admin (OBLIGATOIRE en production)

La clé admin permet de gérer les clés API via l'endpoint `/api-keys`.

### Option 1 : Variable d'environnement (Recommandé)

```bash
# Windows PowerShell
$env:AIRADCR_ADMIN_KEY = "votre_cle_admin_secrete_min_32_chars"

# Windows CMD
set AIRADCR_ADMIN_KEY=votre_cle_admin_secrete_min_32_chars

# Linux/macOS
export AIRADCR_ADMIN_KEY="votre_cle_admin_secrete_min_32_chars"
```

### Option 2 : Fichier de configuration

Créez le fichier `~/.airadcr/admin.key` :

```bash
# Windows
mkdir %USERPROFILE%\.airadcr
echo votre_cle_admin_secrete_min_32_chars > %USERPROFILE%\.airadcr\admin.key

# Linux/macOS
mkdir -p ~/.airadcr
echo "votre_cle_admin_secrete_min_32_chars" > ~/.airadcr/admin.key
chmod 600 ~/.airadcr/admin.key
```

### ⚠️ Important

- **En mode Release (production)** : Sans clé configurée, toutes les fonctions d'administration sont **désactivées**
- **En mode Debug** : Une clé de développement temporaire est utilisée (avec avertissement)
- **Recommandation** : Utilisez une clé d'au moins 32 caractères alphanumériques

---

## 🔐 Clé API Production

La clé API authentifie les requêtes POST vers `/pending-report`.

### Création via HTTP (avec clé admin)

```bash
curl -X POST http://localhost:8741/api-keys \
  -H "Content-Type: application/json" \
  -H "X-Admin-Key: votre_cle_admin" \
  -d '{"name": "RIS Production"}'
```

### Création via Debug Panel

1. Ouvrir l'application AIRADCR Desktop
2. Appuyer sur `Ctrl+Alt+D` pour ouvrir le panneau de debug
3. Onglet "Base de données" → "Créer une clé API"
4. **Sauvegarder immédiatement** la clé affichée

### Utilisation

```bash
curl -X POST http://localhost:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: airadcr_xxxxxxxx_yyyyyyyyyyyyyyyy" \
  -d '{"technical_id": "test123", "structured": {...}}'
```

---

## 🛡️ Recommandations de Déploiement

### Réseau

- ✅ Le serveur HTTP écoute **uniquement sur 127.0.0.1** (localhost)
- ✅ Aucun port exposé sur le réseau externe
- ⚠️ Si un accès réseau est nécessaire, utilisez un reverse proxy avec TLS

### Permissions fichiers

```bash
# Linux/macOS - Sécuriser le fichier de clé admin
chmod 600 ~/.airadcr/admin.key
chown $USER:$USER ~/.airadcr/admin.key
```

### Rotation des clés

1. Créer une nouvelle clé API via `/api-keys`
2. Mettre à jour la configuration RIS/PACS
3. Révoquer l'ancienne clé via `DELETE /api-keys/{prefix}`

---

## 🔍 Audit et Monitoring

### Logs d'accès

Les logs d'accès API sont stockés dans SQLite et accessibles via :
- Debug Panel → Onglet "Base de données" → "Logs d'accès"
- Commande Tauri : `get_access_logs`

### Données loggées

- ✅ Timestamp, IP, méthode HTTP, endpoint, code statut
- ✅ Préfixe de clé API (pas la clé complète)
- ✅ User-Agent, durée de requête
- ✅ Messages d'erreur (masqués pour les données sensibles)
- ❌ Contenu des rapports
- ❌ Identifiants patients complets (masqués : `1234****`)

---

## 📊 Checklist Pré-Production

- [ ] Variable `AIRADCR_ADMIN_KEY` configurée
- [ ] Au moins une clé API créée pour le RIS/PACS
- [ ] Clé API distribuée de manière sécurisée à l'équipe RIS
- [ ] Logs d'accès activés et consultables
- [ ] Backup régulier du fichier `pending_reports.db`
- [ ] Protocole `airadcr://` enregistré (deep links)

---

## 🚨 En cas d'incident

### Clé API compromise

1. Révoquer immédiatement : `DELETE /api-keys/{prefix}`
2. Créer une nouvelle clé
3. Mettre à jour la configuration RIS/PACS
4. Consulter les logs d'accès pour identifier les requêtes suspectes

### Clé Admin compromise

1. Stopper l'application
2. Supprimer `~/.airadcr/admin.key` ou désactiver la variable d'environnement
3. Redémarrer l'application (fonctions admin désactivées)
4. Configurer une nouvelle clé admin
5. Révoquer toutes les clés API existantes et en créer de nouvelles

---

*Document généré le 2026-01-11 - Version 1.0*
