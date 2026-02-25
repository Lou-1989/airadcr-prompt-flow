# Guide Développeur — Intégration AIRADCR Desktop

> **Version** : 3.0 — Février 2026  
> **Audience** : Développeur RIS / PACS / TÉO Hub  
> **Prérequis** : AIRADCR Desktop installé et démarré sur le poste radiologique

---

## ⚡ Appel rapide — Ce que vous cherchez probablement

```bash
# Récupérer un rapport IA depuis TÉO Hub et rafraîchir l'interface AIRADCR
curl -X POST "http://localhost:8741/refresh_gui?patient_id=PAT001&study_uid=1.2.840.113619.2.55.3" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Ce que fait cet appel :**
1. Cherche le rapport en base locale (SQLite)
2. Si absent → appelle TÉO Hub automatiquement (`GET /th_get_ai_report`)
3. Stocke le rapport localement
4. Rafraîchit l'iframe AIRADCR avec le rapport IA pré-rempli

**Réponse attendue :**
```json
{
  "success": true,
  "message": "Navigation triggered successfully",
  "technical_id": "teo_a1b2c3d4",
  "navigated_to": "https://airadcr.com/app?tori=true&tid=teo_a1b2c3d4",
  "source": "local"
}
```

> `/refresh_gui` est un **alias** de `/open-report` — les deux sont 100 % identiques.

---

## Table des matières

1. [Architecture du pipeline](#1-architecture-du-pipeline)
2. [Authentification](#2-authentification)
3. [Référence des endpoints](#3-référence-des-endpoints)
4. [Mode Push — Envoi manuel du rapport](#4-mode-push--envoi-manuel-du-rapport)
5. [Mode Pull — Récupération automatique TÉO Hub](#5-mode-pull--récupération-automatique-téo-hub)
6. [Exemples curl complets](#6-exemples-curl-complets)
7. [Codes HTTP et erreurs](#7-codes-http-et-erreurs)
8. [Configuration TÉO Hub](#8-configuration-téo-hub)
9. [Troubleshooting](#9-troubleshooting)

---

## 1. Architecture du pipeline

### Mode Push (2 appels HTTP)

Le système externe (TÉO Hub) envoie le rapport complet, puis déclenche l'ouverture dans l'iframe.

```
┌──────────┐   POST /pending-report      ┌──────────────┐
│ TÉO Hub  │ ──────────────────────────▶ │  SQLite      │
│ ou RIS   │    (rapport structuré)      │  locale      │
└──────────┘                             └──────┬───────┘
      │                                         │
      │   POST /open-report?tid=XXX             │
      │   (ou POST /refresh_gui?tid=XXX)        │
      └────────────────────────────────▶ Événement Tauri
                                                │
                                        ┌───────▼───────┐
                                        │ Iframe        │
                                        │ airadcr.com   │
                                        │ ?tori=true    │
                                        │ &tid=XXX      │
                                        └───────────────┘
```

### Mode Pull (1 seul appel HTTP) ← **Recommandé**

AIRADCR récupère automatiquement le rapport depuis TÉO Hub.

```
┌──────────┐   POST /refresh_gui                ┌──────────────┐
│   RIS    │   ?patient_id=X&study_uid=Y        │  SQLite      │
└────┬─────┘ ──────────────────────────────────▶│  locale      │
     │                                          └──────┬───────┘
     │         Pas trouvé en local ?                   │
     │         ┌───────────────────────────┐           │
     │         │ Fallback automatique      │           │
     │         │ GET TÉO Hub              │───────────▶│ Stockage
     │         │ /th_get_ai_report         │           │
     │         │ ?patient_id=X&study_uid=Y │           │
     │         └───────────────────────────┘           │
     │                                         Événement Tauri
     │                                         "airadcr:navigate_to_report"
     │                                                 │
     │                                         ┌───────▼───────┐
     │                                         │ Iframe        │
     │                                         │ airadcr.com   │
     │                                         │ ?tori=true    │
     │                                         │ &tid=teo_xxx  │
     │                                         └───────────────┘
```

---

## 2. Authentification

### Header `X-API-Key`

Tous les endpoints d'écriture nécessitent le header `X-API-Key`.

| Méthode | Endpoint | Auth requise |
|---------|----------|:------------:|
| `GET`   | `/health` | ❌ Non |
| `GET`   | `/health/extended` | ❌ Non |
| `GET`   | `/metrics` | ❌ Non |
| `GET`   | `/pending-report?tid=` | ❌ Non |
| `GET`   | `/find-report` | ❌ Non |
| `POST`  | `/pending-report` | ✅ `X-API-Key` |
| `POST`  | `/open-report` | ✅ `X-API-Key` |
| `POST`  | `/refresh_gui` | ✅ `X-API-Key` |
| `DELETE` | `/pending-report` | ✅ `X-API-Key` |
| `GET`   | `/teo-hub/fetch` | ✅ `X-API-Key` |
| `POST`  | `/api-keys` | ✅ `X-Admin-Key` |
| `GET`   | `/api-keys` | ✅ `X-Admin-Key` |
| `DELETE` | `/api-keys/{prefix}` | ✅ `X-Admin-Key` |

### Obtenir une clé API

**Option 1 — Clé par défaut (développement uniquement) :**
```
airadcr_prod_7f3k9m2x5p8w1q4v6n0z
```
> ⚠️ Ne pas utiliser en production !

**Option 2 — Variable d'environnement (production) :**
```bash
set AIRADCR_PROD_API_KEY=votre_cle_secrete_production
```

**Option 3 — Créer une clé via l'API admin :**
```bash
curl -X POST http://localhost:8741/api-keys \
  -H "X-Admin-Key: VOTRE_CLE_ADMIN" \
  -H "Content-Type: application/json" \
  -d '{"name": "RIS Integration"}'
```

---

## 3. Référence des endpoints

| Méthode | URL | Params query | Description |
|---------|-----|-------------|-------------|
| `GET` | `/health` | — | Vérification serveur actif |
| `GET` | `/health/extended` | — | Health check détaillé (DB, TÉO Hub) |
| `GET` | `/metrics` | — | Métriques Prometheus |
| `POST` | `/pending-report` | — | Stocker un rapport structuré (body JSON) |
| `GET` | `/pending-report` | `tid` | Récupérer un rapport par technical_id |
| `DELETE` | `/pending-report` | `tid` | Supprimer un rapport |
| `GET` | `/find-report` | `accession_number`, `patient_id`, `exam_uid` | Chercher par identifiants RIS |
| `POST` | `/open-report` | `tid`, `patient_id`, `accession_number`, `exam_uid`, `study_uid` | Ouvrir un rapport dans l'iframe |
| `POST` | `/refresh_gui` | *(identique à `/open-report`)* | **Alias** — même handler, même comportement |
| `GET` | `/teo-hub/fetch` | `patient_id`, `study_uid`, `exam_uid` | Importer depuis TÉO Hub sans ouvrir l'iframe |

### Paramètres de `/open-report` et `/refresh_gui`

| Param | Type | Description |
|-------|------|-------------|
| `tid` | `string` | Technical ID direct (prioritaire) |
| `patient_id` | `string` | Identifiant patient |
| `accession_number` | `string` | Numéro d'accession RIS |
| `exam_uid` | `string` | UID d'examen DICOM |
| `study_uid` | `string` | **Alias** de `exam_uid` — UID d'étude DICOM (accepté partout) |

> Si `exam_uid` ET `study_uid` sont fournis, `exam_uid` a la priorité.

---

## 4. Mode Push — Envoi manuel du rapport

### Étape 1 : Envoyer le rapport (`POST /pending-report`)

```bash
curl -X POST http://localhost:8741/pending-report \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -H "Content-Type: application/json" \
  -d '{
    "technical_id": "rapport_12345",
    "patient_id": "PAT001",
    "exam_uid": "1.2.840.113619.2.55.3",
    "accession_number": "ACC2026001",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées chroniques",
      "technique": "Séquences T1, T2, FLAIR, diffusion",
      "results": "Pas de lésion intra-crânienne décelée...",
      "conclusion": "Examen normal."
    },
    "source_type": "teo_hub",
    "modality": "MR",
    "expires_in_hours": 24
  }'
```

**Réponse `200` :**
```json
{
  "success": true,
  "technical_id": "rapport_12345",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=rapport_12345",
  "expires_at": "2026-02-26T15:30:00Z"
}
```

### Étape 2 : Ouvrir dans l'iframe (`POST /open-report` ou `/refresh_gui`)

```bash
curl -X POST "http://localhost:8741/refresh_gui?tid=rapport_12345" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse `200` :**
```json
{
  "success": true,
  "message": "Navigation triggered successfully",
  "technical_id": "rapport_12345",
  "navigated_to": "https://airadcr.com/app?tori=true&tid=rapport_12345",
  "source": "local"
}
```

---

## 5. Mode Pull — Récupération automatique TÉO Hub

### Un seul appel suffit

Si le rapport n'existe pas localement, AIRADCR le récupère **automatiquement** depuis TÉO Hub.

```bash
curl -X POST "http://localhost:8741/refresh_gui?patient_id=PAT001&study_uid=1.2.840.113619.2.55.3" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse `200` :**
```json
{
  "success": true,
  "message": "Navigation triggered successfully",
  "technical_id": "teo_a1b2c3d4",
  "navigated_to": "https://airadcr.com/app?tori=true&tid=teo_a1b2c3d4",
  "source": "local"
}
```

> Le `technical_id` est auto-généré avec le préfixe `teo_`.

### Pipeline détaillé déclenché par cet appel

```
POST /refresh_gui?patient_id=PAT001&study_uid=1.2.840...
  │
  ▼
1. Recherche SQLite locale (par patient_id + study_uid)
  │
  ▼ (pas trouvé)
2. Appel TÉO Hub : GET /th_get_ai_report?patient_id=PAT001&study_uid=1.2.840...
  │
  ▼ (rapport IA reçu)
3. Stockage SQLite local (technical_id auto-généré : teo_XXXXXXXX)
  │
  ▼
4. Émission événement Tauri "airadcr:navigate_to_report"
  │
  ▼
5. WebViewContainer écoute l'événement
  │
  ▼
6. iframe.src = "https://airadcr.com/app?tori=true&tid=teo_XXXXXXXX"
  │
  ▼
7. GUI rafraîchie avec le rapport IA pré-rempli
```

### Prérequis pour le Mode Pull

| Élément | Vérification |
|---------|-------------|
| TÉO Hub activé | `config.toml` → `teo_hub.enabled = true` |
| Token API | Keychain OS ou `config.toml` → `teo_hub.api_token` |
| Réseau | TÉO Hub accessible (défaut : `192.168.1.253:54489`) |
| Clé API locale | Header `X-API-Key` dans la requête |

### Option B : Pré-charger sans ouvrir l'iframe (`GET /teo-hub/fetch`)

```bash
curl "http://localhost:8741/teo-hub/fetch?patient_id=PAT001&study_uid=1.2.840.113619.2.55.3" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse `200` :**
```json
{
  "success": true,
  "technical_id": "teo_e5f6g7h8",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=teo_e5f6g7h8",
  "source": "teo_hub"
}
```

Ouvrir plus tard :
```bash
curl -X POST "http://localhost:8741/refresh_gui?tid=teo_e5f6g7h8" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

---

## 6. Exemples curl complets

```bash
# Health check
curl http://localhost:8741/health

# Health check étendu
curl http://localhost:8741/health/extended

# Rechercher un rapport par accession number
curl "http://localhost:8741/find-report?accession_number=ACC2026001"

# Récupérer un rapport par TID
curl "http://localhost:8741/pending-report?tid=rapport_12345"

# Mode Pull : récupérer + afficher en 1 appel
curl -X POST "http://localhost:8741/refresh_gui?patient_id=PAT001&study_uid=1.2.840.113619.2.55.3" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"

# Supprimer un rapport
curl -X DELETE "http://localhost:8741/pending-report?tid=rapport_12345" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"

# Créer une clé API
curl -X POST http://localhost:8741/api-keys \
  -H "X-Admin-Key: VOTRE_CLE_ADMIN" \
  -H "Content-Type: application/json" \
  -d '{"name": "RIS Integration"}'
```

---

## 7. Codes HTTP et erreurs

| Code | Signification | Quand |
|------|--------------|-------|
| `200` | Succès | Opération réussie |
| `201` | Créé | Clé API créée |
| `400` | Bad Request | Paramètre manquant ou invalide |
| `401` | Unauthorized | Clé API manquante ou invalide |
| `404` | Not Found | Rapport non trouvé (et TÉO Hub désactivé ou échoué) |
| `500` | Internal Error | Erreur base de données ou événement Tauri |
| `502` | Bad Gateway | TÉO Hub a répondu avec une erreur |
| `503` | Service Unavailable | App pas encore prête ou TÉO Hub désactivé |

### Format d'erreur standard

```json
{
  "error": "Description lisible de l'erreur",
  "field": "nom_du_champ_invalide"
}
```

---

## 8. Configuration TÉO Hub

Fichier : `%APPDATA%/airadcr-desktop/config.toml`

```toml
[teo_hub]
enabled = true
host = "192.168.1.253"
port = 54489
health_endpoint = "th_health"
get_report_endpoint = "th_get_ai_report"
post_report_endpoint = "th_post_approved_report"
timeout_secs = 30
retry_count = 3
retry_delay_ms = 1000

# TLS (optionnel)
tls_enabled = false
ca_file = ""
cert_file = ""
key_file = ""
```

### Token API TÉO Hub

Le token est stocké dans le **keychain OS** (Windows Credential Manager). Configuration initiale via `config.toml` :

```toml
[teo_hub]
api_token = "Dz1RyxZu8noENuX9Vno9URcBlsP0UXA1UgUDX0Fd7gJQL2tY4zvlIRDsxIISkrk7sJ8PR2vfC6mGOvQK"
```

> Au prochain démarrage, le token sera migré automatiquement vers le keychain OS et supprimé du fichier TOML.

---

## 9. Troubleshooting

### Le serveur ne répond pas

```bash
curl http://localhost:8741/health
# Si erreur connexion → AIRADCR Desktop n'est pas démarré
```

**Solutions :**
- Vérifier que AIRADCR Desktop est lancé
- Vérifier le port dans `config.toml` (`http_port`)
- Vérifier le pare-feu Windows (port 8741)

### Erreur 401 Unauthorized

Clé API invalide ou manquante.

```bash
# Test avec la clé de dev :
curl -X POST "http://localhost:8741/refresh_gui?tid=test" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

### Erreur 502 Bad Gateway (Mode Pull)

TÉO Hub n'est pas accessible ou a retourné une erreur.

**Vérifications :**
1. TÉO Hub activé ? → `config.toml` → `teo_hub.enabled = true`
2. TÉO Hub joignable ? → `curl http://192.168.1.253:54489/th_health`
3. Token API correct ? → Vérifier dans le keychain OS

### Rapport non trouvé (404)

- Mode Push : le rapport a peut-être expiré (24h par défaut)
- Mode Pull : TÉO Hub n'a pas de rapport pour ce patient/examen
- Vérifier les identifiants (`patient_id`, `study_uid`, `accession_number`)

### `study_uid` vs `exam_uid`

Les deux sont acceptés partout. `study_uid` est l'alias DICOM standard. Si les deux sont fournis, `exam_uid` a la priorité.

---

## Résumé rapide

| Je veux... | Commande |
|-----------|----------|
| Vérifier le serveur | `GET /health` |
| Envoyer un rapport complet | `POST /pending-report` puis `POST /refresh_gui?tid=X` |
| Récupérer + afficher auto | `POST /refresh_gui?patient_id=X&study_uid=Y` |
| Pré-charger depuis TÉO Hub | `GET /teo-hub/fetch?patient_id=X&study_uid=Y` |
| Chercher par accession | `GET /find-report?accession_number=X` |
| Supprimer un rapport | `DELETE /pending-report?tid=X` |
