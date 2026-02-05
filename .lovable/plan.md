
# Plan d'Implementation: Mode Client HTTP vers TÉO Hub

## 1. Contexte et Objectif

### 1.1 Situation Actuelle
L'application AIRADCR Desktop fonctionne actuellement en mode **SERVEUR HTTP** (écoute sur `localhost:8741`). Les systèmes externes envoient des données vers AIRADCR.

### 1.2 Nouveau Besoin
Le CTO de TÉO Hub a implementé une architecture où **TÉO Hub est le serveur** et **AIRADCR doit être le client** qui :
- **Récupère** les rapports IA via `GET /th_get_ai_report`
- **Renvoie** les rapports validés via `POST /th_post_approved_report`

### 1.3 Configuration TÉO Hub
```text
┌─────────────────────────────────────────────────────────────┐
│  Paramètres TÉO Hub (fournis par le CTO)                    │
├─────────────────────────────────────────────────────────────┤
│  Host       : 192.168.1.36 (variable)                       │
│  Port       : 54489                                         │
│  Endpoints  : th_health, th_get_ai_report,                  │
│               th_post_approved_report                       │
│  SSL        : Prévu (certificats CA, cert, key)             │
│  Body limit : 20 Mo                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Architecture Proposée

### 2.1 Diagramme du Flux Client

```text
┌───────────────┐                    ┌─────────────────┐
│  TÉO Hub      │                    │  AIRADCR Tauri  │
│  Serveur HTTP │                    │  Client HTTP    │
│  :54489       │                    │                 │
└───────┬───────┘                    └────────┬────────┘
        │                                     │
        │  1. GET /th_health                  │
        │<────────────────────────────────────│
        │                                     │
        │  2. 200 OK {"status": "ok"}         │
        │────────────────────────────────────>│
        │                                     │
        │  3. GET /th_get_ai_report           │
        │     ?accession_number=ACC001        │
        │<────────────────────────────────────│
        │                                     │
        │  4. 200 + Rapport IA structuré      │
        │────────────────────────────────────>│
        │                                     │
        │         [ Radiologue valide ]       │
        │                                     │
        │  5. POST /th_post_approved_report   │
        │     (rapport final + metadata)      │
        │<────────────────────────────────────│
        │                                     │
        │  6. 201 Created                     │
        │────────────────────────────────────>│
```

### 2.2 Coexistence des Deux Modes

L'application supportera les deux modes simultanément :

```text
┌─────────────────────────────────────────────────────────────┐
│                    AIRADCR Desktop                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   MODE SERVEUR (existant)         MODE CLIENT (nouveau)     │
│   localhost:8741                  -> TÉO Hub:54489          │
│   ─────────────────               ────────────────────      │
│   POST /pending-report            GET /th_get_ai_report     │
│   GET  /pending-report            POST /th_post_approved    │
│   POST /open-report                                         │
│                                                             │
│   Reçoit depuis: RIS/PACS         Envoie vers: TÉO Hub      │
│   Stocke dans: SQLite local       Récupère depuis: MySQL    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Modifications Techniques

### 3.1 Nouveau Module Rust: `teo_client`

**Fichier**: `src-tauri/src/teo_client/mod.rs`

Responsabilités :
- Client HTTP asynchrone (reqwest)
- Gestion des certificats TLS
- Retry logic avec backoff exponentiel
- Health check automatique

### 3.2 Configuration Étendue

**Fichier**: `src-tauri/src/config.rs` (extension)

Nouveaux paramètres dans `config.toml` :
```toml
# Configuration TÉO Hub Client
[teo_hub]
enabled = true
host = "192.168.1.36"
port = 54489
health_endpoint = "th_health"
get_report_endpoint = "th_get_ai_report"
post_report_endpoint = "th_post_approved_report"
timeout_secs = 30
retry_count = 3
retry_delay_ms = 1000

# SSL/TLS (optionnel)
tls_enabled = false
ca_file = ""
cert_file = ""
key_file = ""
```

### 3.3 Structures de Données TÉO Hub

**Fichier**: `src-tauri/src/teo_client/models.rs`

```text
TeoHealthResponse
├── status: String
└── version: Option<String>

TeoAiReport (réponse GET)
├── report_id: String
├── accession_number: String
├── patient_id: String
├── study_instance_uid: String
├── modality: String
├── ai_analysis: TeoAiAnalysis
│   ├── findings: Vec<String>
│   ├── measurements: Option<Value>
│   ├── confidence_score: f64
│   └── ai_modules: Vec<String>
├── template_id: Option<String>
├── status: String
└── created_at: String

TeoApprovedReport (payload POST)
├── report_id: String
├── accession_number: String
├── approved_text: String
├── radiologist_id: Option<String>
├── approval_timestamp: String
└── modifications_made: bool
```

### 3.4 Nouvelles Commandes Tauri

**Fichier**: `src-tauri/src/main.rs` (extension)

| Commande | Description |
|----------|-------------|
| `teo_check_health` | Vérifie la disponibilité du serveur TÉO Hub |
| `teo_fetch_report` | Récupère un rapport IA depuis TÉO Hub |
| `teo_submit_approved` | Envoie le rapport validé à TÉO Hub |
| `teo_get_config` | Récupère la configuration client active |
| `teo_update_config` | Met à jour les paramètres de connexion |

### 3.5 Dépendances Cargo

**Fichier**: `src-tauri/Cargo.toml` (ajout)

```toml
# Client HTTP async
reqwest = { version = "0.12", features = ["json", "rustls-tls", "cookies"] }

# Gestion certificats TLS
rustls = "0.23"
rustls-pemfile = "2"
```

---

## 4. Sécurité

### 4.1 Authentification TÉO Hub

Le CTO n'a pas précisé le mécanisme d'authentification. Options supportées :
- **API Key** via header `X-API-Key`
- **Bearer Token** via header `Authorization`
- **Certificat client TLS** (mutualTLS)

La configuration permettra les trois modes.

### 4.2 Validation des Réponses

- Vérification du Content-Type (application/json)
- Validation du schéma JSON avec serde
- Limite de taille (20 Mo max)
- Timeout configurable

### 4.3 Logging Sécurisé

- PII masqué dans les logs (patient_id: `1234****`)
- Credentials jamais loggés
- Logs d'accès dans SQLite (audit)

---

## 5. Interface Utilisateur

### 5.1 Panneau de Configuration TÉO Hub

Nouveau panneau dans l'interface (React) permettant de :
- Configurer l'adresse du serveur TÉO Hub
- Tester la connexion (health check)
- Voir le statut de la connexion
- Uploader les certificats TLS

### 5.2 Indicateur de Connexion

Badge dans la barre d'état montrant :
- Vert : TÉO Hub connecté
- Orange : En cours de connexion
- Rouge : TÉO Hub inaccessible
- Gris : Mode TÉO Hub désactivé

---

## 6. Plan de Fichiers

| Action | Fichier | Description |
|--------|---------|-------------|
| CRÉER | `src-tauri/src/teo_client/mod.rs` | Module principal client |
| CRÉER | `src-tauri/src/teo_client/models.rs` | Structures de données |
| CRÉER | `src-tauri/src/teo_client/errors.rs` | Gestion des erreurs |
| MODIFIER | `src-tauri/src/config.rs` | Ajout config TÉO Hub |
| MODIFIER | `src-tauri/src/main.rs` | Nouvelles commandes Tauri |
| MODIFIER | `src-tauri/Cargo.toml` | Dépendance reqwest |
| CRÉER | `src/components/TeoHubConfig.tsx` | UI configuration |
| MODIFIER | `src/components/DebugPanel.tsx` | Ajout onglet TÉO Hub |

---

## 7. Workflow Complet Intégré

```text
┌────────────────────────────────────────────────────────────────────┐
│                    WORKFLOW COMPLET                                │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  1. PACS → TÉO Hub                                                 │
│     [Images DICOM envoyées pour analyse IA]                        │
│                                                                    │
│  2. TÉO Hub → Prestataire IA (Gleamer, etc.)                       │
│     [Analyse volumétrie, détection lésions, etc.]                  │
│                                                                    │
│  3. Prestataire IA → TÉO Hub                                       │
│     [Résultats IA + DICOM SR]                                      │
│                                                                    │
│  4. TÉO Hub stocke dans MySQL (table th_ai_reports)                │
│                                                                    │
│  5. RIS/PACS → AIRADCR (appel contextuel)                          │
│     POST localhost:8741/open-report?accession_number=XXX           │
│                                                                    │
│  6. AIRADCR → TÉO Hub [NOUVEAU CLIENT]                             │
│     GET http://192.168.1.36:54489/th_get_ai_report                 │
│         ?accession_number=XXX                                      │
│                                                                    │
│  7. AIRADCR affiche le rapport pré-rempli                          │
│     [Radiologue valide/modifie]                                    │
│                                                                    │
│  8. AIRADCR → TÉO Hub [NOUVEAU CLIENT]                             │
│     POST http://192.168.1.36:54489/th_post_approved_report         │
│                                                                    │
│  9. AIRADCR → RIS                                                  │
│     [Injection clavier du rapport final]                           │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

---

## 8. Estimation et Phases

| Phase | Tâche | Durée estimée |
|-------|-------|---------------|
| 1 | Configuration + structures de données | 2-3 heures |
| 2 | Client HTTP avec reqwest + TLS | 3-4 heures |
| 3 | Commandes Tauri + intégration | 2-3 heures |
| 4 | Interface utilisateur React | 2-3 heures |
| 5 | Tests et documentation | 2 heures |
| **Total** | | **11-15 heures** |

---

## 9. Points à Clarifier avec le CTO

Avant l'implementation complète, il serait utile de confirmer :

1. **Authentification** : Quel mécanisme (API Key, Bearer, mTLS) ?
2. **Format exact** des réponses `th_get_ai_report`
3. **Format exact** attendu pour `th_post_approved_report`
4. **Gestion des erreurs** côté TÉO Hub (codes HTTP, messages)
5. **Polling vs Push** : AIRADCR doit-il interroger périodiquement ou attendre un signal ?

---

## 10. Résumé Exécutif

Cette implementation ajoute un **mode client HTTP** à AIRADCR Desktop pour communiquer avec TÉO Hub en tant que serveur central. Les deux modes (serveur local et client TÉO Hub) coexistent et permettent une intégration flexible selon le contexte de déploiement.

L'architecture est conçue pour être **robuste** (retry logic, timeouts), **sécurisée** (TLS, authentification, PII masking) et **configurable** (paramètres externalisés dans config.toml).
