# Guide Développeur — Intégration AIRADCR Desktop

> **Version** : 2.0 — Février 2026  
> **Audience** : Développeur RIS / PACS / TÉO Hub  
> **Prérequis** : AIRADCR Desktop installé et démarré sur le poste radiologique

---

## Table des matières

1. [Vue d'ensemble](#1-vue-densemble)
2. [Architecture du pipeline](#2-architecture-du-pipeline)
3. [Authentification](#3-authentification)
4. [Endpoints de référence](#4-endpoints-de-référence)
5. [Mode Push (manuel)](#5-mode-push-manuel)
6. [Mode Pull (automatique TÉO Hub)](#6-mode-pull-automatique-téo-hub)
7. [Exemples curl complets](#7-exemples-curl-complets)
8. [Codes HTTP et erreurs](#8-codes-http-et-erreurs)
9. [Configuration TÉO Hub](#9-configuration-téo-hub)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Vue d'ensemble

AIRADCR Desktop expose un **serveur HTTP local** sur le port **8741** (configurable) qui permet aux systèmes externes (RIS, PACS, TÉO Hub) d'interagir avec l'application de dictée radiologique.

**Deux modes d'intégration** sont supportés :

| Mode | Description | Appels nécessaires |
|------|-------------|-------------------|
| **Push** | Le système externe envoie le rapport complet | 2 appels : `POST /pending-report` puis `POST /open-report` |
| **Pull** | AIRADCR récupère automatiquement depuis TÉO Hub | 1 seul appel : `POST /open-report?patient_id=X&exam_uid=Y` |

---

## 2. Architecture du pipeline

### Mode Push (2 appels)

```
┌──────────┐   POST /pending-report    ┌──────────────┐
│ TÉO Hub  │ ─────────────────────────▶│  SQLite      │
│ ou RIS   │                           │  locale      │
└──────────┘                           └──────┬───────┘
      │                                       │
      │   POST /open-report?tid=XXX           │
      └──────────────────────────────▶ Événement Tauri
                                              │
                                      ┌───────▼───────┐
                                      │ Iframe        │
                                      │ airadcr.com   │
                                      │ ?tori=true    │
                                      │ &tid=XXX      │
                                      └───────────────┘
```

### Mode Pull (1 appel)

```
┌──────────┐   POST /open-report           ┌──────────────┐
│   RIS    │   ?patient_id=X&exam_uid=Y    │  SQLite      │
└────┬─────┘ ─────────────────────────────▶│  locale      │
     │                                      └──────┬───────┘
     │         Pas trouvé en local                 │
     │         ┌──────────────────────┐            │
     │         │ Fallback automatique │            │
     │         │ GET TÉO Hub API     │────────────▶│ Stockage
     │         │ /th_get_ai_report   │            │
     │         └──────────────────────┘            │
     │                                     Événement Tauri
     │                                             │
     │                                     ┌───────▼───────┐
     │                                     │ Iframe        │
     │                                     │ airadcr.com   │
     │                                     │ ?tori=true    │
     │                                     │ &tid=teo_xxx  │
     │                                     └───────────────┘
```

---

## 3. Authentification

### Clé API (X-API-Key)

Tous les endpoints d'écriture nécessitent un header `X-API-Key`.

**3 façons d'obtenir une clé API :**

#### a) Clé de production (variable d'environnement)

```bash
# Sur le poste AIRADCR, définir :
set AIRADCR_PROD_API_KEY=votre_cle_secrete
```

#### b) Clé par défaut (développement)

```
airadcr_prod_7f3k9m2x5p8w1q4v6n0z
```

> ⚠️ **Ne pas utiliser en production !**

#### c) Créer une clé via l'API admin

```bash
curl -X POST http://localhost:8741/api-keys \
  -H "X-Admin-Key: VOTRE_CLE_ADMIN" \
  -H "Content-Type: application/json" \
  -d '{"name": "RIS Integration"}'
```

La clé admin est définie via `AIRADCR_ADMIN_KEY`.

### Endpoints sans authentification

| Endpoint | Auth requise |
|----------|-------------|
| `GET /health` | ❌ Non |
| `GET /pending-report?tid=` | ❌ Non (configurable) |
| `GET /find-report` | ❌ Non (configurable) |
| `POST /pending-report` | ✅ X-API-Key |
| `POST /open-report` | ✅ X-API-Key |
| `DELETE /pending-report` | ✅ X-API-Key |
| `GET /teo-hub/fetch` | ✅ X-API-Key |
| `POST /api-keys` | ✅ X-Admin-Key |

---

## 4. Endpoints de référence

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/health` | Vérification serveur actif |
| `GET` | `/health/extended` | Health check détaillé |
| `GET` | `/metrics` | Métriques Prometheus |
| `POST` | `/pending-report` | Stocker un rapport structuré |
| `GET` | `/pending-report?tid=XXX` | Récupérer un rapport par TID |
| `DELETE` | `/pending-report?tid=XXX` | Supprimer un rapport |
| `GET` | `/find-report?accession_number=XXX` | Chercher par identifiants RIS |
| `POST` | `/open-report?tid=XXX` | Ouvrir un rapport dans l'iframe |
| `GET` | `/teo-hub/fetch?patient_id=X&study_uid=Y` | 🆕 Importer depuis TÉO Hub |

---

## 5. Mode Push (manuel)

### Étape 1 : Envoyer le rapport

```bash
curl -X POST http://localhost:8741/pending-report \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -H "Content-Type: application/json" \
  -d '{
    "technical_id": "rapport_12345",
    "patient_id": "PAT001",
    "exam_uid": "1.2.3.4.5.6.7.8.9",
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

**Réponse 200 :**
```json
{
  "success": true,
  "technical_id": "rapport_12345",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=rapport_12345",
  "expires_at": "2026-02-26T15:30:00Z"
}
```

### Étape 2 : Ouvrir dans l'iframe

```bash
curl -X POST "http://localhost:8741/open-report?tid=rapport_12345" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse 200 :**
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

## 6. Mode Pull (automatique TÉO Hub)

### Option A : open-report avec fallback automatique

Un **seul appel** suffit. Si le rapport n'existe pas localement, AIRADCR le récupère automatiquement depuis TÉO Hub.

```bash
curl -X POST "http://localhost:8741/open-report?patient_id=PAT001&exam_uid=1.2.3.4.5.6.7.8.9" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse 200 :**
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

**Prérequis :**
- TÉO Hub activé dans `config.toml` (`teo_hub.enabled = true`)
- API_TOKEN TÉO Hub configuré (keychain OS ou config.toml)
- TÉO Hub accessible sur le réseau

### Option B : fetch dédié (sans navigation)

Pour pré-charger un rapport depuis TÉO Hub **sans ouvrir l'iframe** :

```bash
curl "http://localhost:8741/teo-hub/fetch?patient_id=PAT001&study_uid=1.2.3.4.5.6.7.8.9" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

**Réponse 200 :**
```json
{
  "success": true,
  "technical_id": "teo_e5f6g7h8",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=teo_e5f6g7h8",
  "source": "teo_hub"
}
```

Vous pouvez ensuite appeler `POST /open-report?tid=teo_e5f6g7h8` pour ouvrir le rapport quand vous le souhaitez.

---

## 7. Exemples curl complets

### Health check

```bash
curl http://localhost:8741/health
# {"status":"ok","timestamp":"2026-02-25T10:00:00Z","version":""}
```

### Rechercher un rapport par accession number

```bash
curl "http://localhost:8741/find-report?accession_number=ACC2026001"
```

### Supprimer un rapport

```bash
curl -X DELETE "http://localhost:8741/pending-report?tid=rapport_12345" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

### Workflow complet Mode Pull

```bash
# 1. Vérifier que le serveur est actif
curl http://localhost:8741/health

# 2. Récupérer et ouvrir en un seul appel
curl -X POST "http://localhost:8741/open-report?patient_id=PAT001&exam_uid=1.2.3.4.5" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

---

## 8. Codes HTTP et erreurs

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

## 9. Configuration TÉO Hub

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

Le token est stocké dans le **keychain OS** (Windows Credential Manager). Pour le configurer initialement, ajoutez-le dans `config.toml` :

```toml
[teo_hub]
api_token = "Dz1RyxZu8noENuX9Vno9URcBlsP0UXA1UgUDX0Fd7gJQL2tY4zvlIRDsxIISkrk7sJ8PR2vfC6mGOvQK"
```

Au prochain démarrage, le token sera automatiquement migré vers le keychain OS et supprimé du fichier TOML.

---

## 10. Troubleshooting

### Le serveur ne répond pas

```bash
curl http://localhost:8741/health
# Si erreur connexion : l'application AIRADCR Desktop n'est pas démarrée
```

**Solutions :**
- Vérifier que AIRADCR Desktop est lancé
- Vérifier le port dans `config.toml` (`http_port`)
- Vérifier le pare-feu Windows (port 8741)

### Erreur 401 Unauthorized

La clé API est invalide ou manquante.

```bash
# Vérifier avec la clé de dev :
curl -X POST http://localhost:8741/open-report?tid=test \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
```

### Mode Pull : erreur 502 Bad Gateway

TÉO Hub n'est pas accessible ou a retourné une erreur.

**Vérifications :**
1. TÉO Hub est-il activé ? → `config.toml` → `teo_hub.enabled = true`
2. TÉO Hub est-il joignable ? → `curl http://192.168.1.253:54489/th_health`
3. Le token API est-il correct ? → Vérifier dans le keychain OS

### Rapport non trouvé (404)

- En mode Push : le rapport a peut-être expiré (24h par défaut)
- En mode Pull : TÉO Hub n'a pas de rapport pour ce patient/examen
- Vérifier les identifiants (patient_id, exam_uid, accession_number)

### Le rapport s'affiche mais l'iframe ne charge pas

- Vérifier que `https://airadcr.com` est accessible depuis le poste
- Vérifier la configuration de l'iframe URL dans `config.toml` → `iframe_url`

---

## Annexe : Résumé rapide

| Je veux... | Commande |
|-----------|----------|
| Vérifier le serveur | `GET /health` |
| Envoyer un rapport complet | `POST /pending-report` + `POST /open-report?tid=X` |
| Ouvrir avec fetch auto | `POST /open-report?patient_id=X&exam_uid=Y` |
| Pré-charger depuis TÉO Hub | `GET /teo-hub/fetch?patient_id=X&study_uid=Y` |
| Chercher par accession | `GET /find-report?accession_number=X` |
| Supprimer un rapport | `DELETE /pending-report?tid=X` |
