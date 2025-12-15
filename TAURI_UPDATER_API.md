# 🔄 API de Mise à Jour Tauri - Documentation Serveur

**Endpoint de mise à jour automatique pour AIRADCR Desktop**

---

## 📋 Vue d'ensemble

L'application AIRADCR Desktop utilise le système de mise à jour intégré de Tauri. Le serveur airadcr.com doit exposer un endpoint qui retourne les informations de mise à jour au format JSON.

### Configuration actuelle (tauri.conf.json)

```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://airadcr.com/api/tauri-updates/{{target}}/{{arch}}/{{current_version}}"
    ],
    "dialog": true,
    "pubkey": "VOTRE_CLE_PUBLIQUE"
  }
}
```

---

## 🌐 Endpoint API

### URL Pattern

```
GET https://airadcr.com/api/tauri-updates/{target}/{arch}/{current_version}
```

### Paramètres d'URL

| Paramètre | Description | Exemples |
|-----------|-------------|----------|
| `target` | Système d'exploitation | `windows-x86_64`, `darwin-x86_64`, `darwin-aarch64`, `linux-x86_64` |
| `arch` | Architecture CPU | `x86_64`, `aarch64`, `i686` |
| `current_version` | Version actuelle de l'app | `1.0.0`, `1.0.1` |

### Exemples de requêtes

```bash
# Windows 64-bit
GET /api/tauri-updates/windows-x86_64/x86_64/1.0.0

# macOS Intel
GET /api/tauri-updates/darwin-x86_64/x86_64/1.0.0

# macOS Apple Silicon
GET /api/tauri-updates/darwin-aarch64/aarch64/1.0.0

# Linux 64-bit
GET /api/tauri-updates/linux-x86_64/x86_64/1.0.0
```

---

## 📤 Réponses

### Cas 1 : Mise à jour disponible (HTTP 200)

```json
{
  "version": "1.1.0",
  "notes": "## Nouveautés v1.1.0\n\n- Amélioration des performances\n- Correction de bugs\n- Nouvelle fonctionnalité X",
  "pub_date": "2025-01-15T10:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRQUFBQUFBQUFBQUF...",
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64-setup.nsis.zip"
    },
    "darwin-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRQUFBQUFBQUFBQUF...",
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRQUFBQUFBQUFBQUF...",
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_aarch64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRQUFBQUFBQUFBQUF...",
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_amd64.AppImage.tar.gz"
    }
  }
}
```

### Cas 2 : Pas de mise à jour (HTTP 204)

```
HTTP/1.1 204 No Content
```

Retourner 204 si `current_version` >= dernière version disponible.

### Cas 3 : Plateforme non supportée (HTTP 404)

```json
{
  "error": "Platform not supported",
  "target": "unknown-platform",
  "supported": ["windows-x86_64", "darwin-x86_64", "darwin-aarch64", "linux-x86_64"]
}
```

---

## 🔐 Signature des mises à jour

### Génération de la paire de clés

```bash
# Générer la paire de clés (à faire une seule fois)
cargo tauri signer generate -w ~/.tauri/airadcr.key

# Output:
# Your public key was generated successfully:
# dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFJUkFEQ1IKUldRQUFBQUFBQUFBQUF...
#
# Your secret key was generated successfully:
# Saved to: ~/.tauri/airadcr.key
```

### Configuration

1. **Clé publique** → dans `tauri.conf.json` (champ `pubkey`)
2. **Clé privée** → variable d'environnement `TAURI_PRIVATE_KEY` lors du build

### Signature des artifacts

Lors du build avec `cargo tauri build`, si `TAURI_PRIVATE_KEY` est défini, Tauri génère automatiquement un fichier `.sig` pour chaque artifact :

```
AIRADCR_1.1.0_x64-setup.nsis.zip
AIRADCR_1.1.0_x64-setup.nsis.zip.sig  ← Signature
```

Le contenu du fichier `.sig` est la valeur à mettre dans le champ `signature` de la réponse JSON.

---

## 🗂️ Structure des fichiers de release

```
/downloads/releases/
├── v1.0.0/
│   ├── AIRADCR_1.0.0_x64-setup.nsis.zip
│   ├── AIRADCR_1.0.0_x64-setup.nsis.zip.sig
│   ├── AIRADCR_1.0.0_x64.app.tar.gz
│   ├── AIRADCR_1.0.0_x64.app.tar.gz.sig
│   └── release-notes.md
├── v1.1.0/
│   ├── AIRADCR_1.1.0_x64-setup.nsis.zip
│   ├── AIRADCR_1.1.0_x64-setup.nsis.zip.sig
│   └── ...
└── latest.json  ← Métadonnées de la dernière version
```

### Fichier latest.json

```json
{
  "version": "1.1.0",
  "pub_date": "2025-01-15T10:00:00Z",
  "notes": "## Nouveautés v1.1.0\n\n- Amélioration des performances",
  "platforms": {
    "windows-x86_64": {
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64-setup.nsis.zip",
      "signature": "..."
    }
  }
}
```

---

## 🛠️ Implémentation serveur

### Exemple Node.js/Express

```javascript
const express = require('express');
const semver = require('semver');
const fs = require('fs');
const path = require('path');

const app = express();

// Charger les métadonnées de la dernière version
function getLatestRelease() {
  const latestPath = path.join(__dirname, 'releases', 'latest.json');
  return JSON.parse(fs.readFileSync(latestPath, 'utf8'));
}

app.get('/api/tauri-updates/:target/:arch/:currentVersion', (req, res) => {
  const { target, arch, currentVersion } = req.params;
  
  try {
    const latest = getLatestRelease();
    
    // Vérifier si une mise à jour est nécessaire
    if (semver.gte(currentVersion, latest.version)) {
      return res.status(204).send();
    }
    
    // Vérifier si la plateforme est supportée
    const platformKey = target; // ex: "windows-x86_64"
    if (!latest.platforms[platformKey]) {
      return res.status(404).json({
        error: 'Platform not supported',
        target: platformKey,
        supported: Object.keys(latest.platforms)
      });
    }
    
    // Retourner les informations de mise à jour
    res.json({
      version: latest.version,
      notes: latest.notes,
      pub_date: latest.pub_date,
      platforms: {
        [platformKey]: latest.platforms[platformKey]
      }
    });
    
  } catch (error) {
    console.error('Update check error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.listen(3000);
```

### Exemple avec Edge Function (Supabase)

```typescript
// supabase/functions/tauri-updates/index.ts
import { serve } from "https://deno.land/std@0.168.0/http/server.ts";
import { gt } from "https://deno.land/x/semver@v1.4.1/mod.ts";

const LATEST_VERSION = "1.1.0";
const RELEASES = {
  "1.1.0": {
    notes: "## Nouveautés v1.1.0\n\n- Améliorations diverses",
    pub_date: "2025-01-15T10:00:00Z",
    platforms: {
      "windows-x86_64": {
        url: "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64-setup.nsis.zip",
        signature: "dW50cnVzdGVkIGNvbW1lbnQ6..."
      },
      "darwin-x86_64": {
        url: "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64.app.tar.gz",
        signature: "dW50cnVzdGVkIGNvbW1lbnQ6..."
      }
    }
  }
};

serve(async (req) => {
  const url = new URL(req.url);
  const pathParts = url.pathname.split('/').filter(Boolean);
  
  // /tauri-updates/{target}/{arch}/{current_version}
  const [, target, arch, currentVersion] = pathParts;
  
  // Pas de mise à jour nécessaire
  if (!gt(LATEST_VERSION, currentVersion)) {
    return new Response(null, { status: 204 });
  }
  
  const release = RELEASES[LATEST_VERSION];
  const platform = release.platforms[target];
  
  if (!platform) {
    return new Response(
      JSON.stringify({ error: "Platform not supported" }),
      { status: 404, headers: { "Content-Type": "application/json" } }
    );
  }
  
  return new Response(
    JSON.stringify({
      version: LATEST_VERSION,
      notes: release.notes,
      pub_date: release.pub_date,
      platforms: { [target]: platform }
    }),
    { headers: { "Content-Type": "application/json" } }
  );
});
```

---

## 🔄 Workflow de publication

### 1. Build et signature

```bash
# Variables d'environnement requises
export TAURI_PRIVATE_KEY=$(cat ~/.tauri/airadcr.key)
export TAURI_KEY_PASSWORD=""  # Si la clé a un mot de passe

# Build pour toutes les plateformes
cargo tauri build
```

### 2. Upload des artifacts

```bash
# Créer le dossier de release
mkdir -p releases/v1.1.0

# Copier les artifacts
cp target/release/bundle/nsis/*.zip releases/v1.1.0/
cp target/release/bundle/nsis/*.zip.sig releases/v1.1.0/

# Pour macOS
cp target/release/bundle/macos/*.tar.gz releases/v1.1.0/
cp target/release/bundle/macos/*.tar.gz.sig releases/v1.1.0/
```

### 3. Mettre à jour latest.json

```bash
# Générer latest.json avec les nouvelles signatures
cat > releases/latest.json << EOF
{
  "version": "1.1.0",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "notes": "## Nouveautés v1.1.0\n\n- Liste des changements",
  "platforms": {
    "windows-x86_64": {
      "url": "https://airadcr.com/downloads/releases/v1.1.0/AIRADCR_1.1.0_x64-setup.nsis.zip",
      "signature": "$(cat releases/v1.1.0/AIRADCR_1.1.0_x64-setup.nsis.zip.sig)"
    }
  }
}
EOF
```

### 4. Déployer

```bash
# Upload vers le serveur
rsync -avz releases/ user@airadcr.com:/var/www/downloads/releases/
```

---

## 🧪 Test de l'endpoint

### Curl

```bash
# Test : version actuelle = 1.0.0 (devrait retourner la mise à jour)
curl -i "https://airadcr.com/api/tauri-updates/windows-x86_64/x86_64/1.0.0"

# Test : version actuelle = 1.1.0 (devrait retourner 204)
curl -i "https://airadcr.com/api/tauri-updates/windows-x86_64/x86_64/1.1.0"

# Test : plateforme inconnue (devrait retourner 404)
curl -i "https://airadcr.com/api/tauri-updates/unknown/x86_64/1.0.0"
```

### PowerShell

```powershell
# Test Windows
Invoke-RestMethod -Uri "https://airadcr.com/api/tauri-updates/windows-x86_64/x86_64/1.0.0"
```

---

## 📊 Monitoring recommandé

### Métriques à suivre

- Nombre de requêtes de mise à jour par version
- Distribution des plateformes (Windows vs macOS vs Linux)
- Taux de téléchargement des mises à jour
- Erreurs (404, 500)

### Headers recommandés

```
Cache-Control: no-cache, no-store, must-revalidate
X-Update-Version: 1.1.0
X-Request-Platform: windows-x86_64
```

---

## ⚠️ Points d'attention

1. **HTTPS obligatoire** : Tauri refuse les endpoints HTTP non sécurisés
2. **Signature valide** : Sans signature correcte, la mise à jour échoue silencieusement
3. **Format ZIP** : Les artifacts doivent être compressés en `.zip` ou `.tar.gz`
4. **CORS** : Si l'endpoint est appelé depuis le frontend, configurer les headers CORS

---

*Documentation pour AIRADCR Desktop v1.0.0*
*Dernière mise à jour : Décembre 2024*
