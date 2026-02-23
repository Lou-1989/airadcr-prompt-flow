# 📋 Documentation API Locale AIRADCR Desktop

**Version** : 2.0.0  
**Dernière mise à jour** : Février 2026  
**Base URL** : `http://127.0.0.1:8741`

---

## Vue d'ensemble

Le serveur HTTP local Tauri (`127.0.0.1:8741`) permet aux systèmes RIS/PACS et à TÉO Hub d'envoyer des rapports radiologiques pré-structurés **avec identifiants patients** car les données ne quittent jamais la machine.

```
┌──────────────┐                                  ┌──────────────────┐
│   TÉO Hub    │  1. POST /pending-report          │  AIRADCR Desktop │
│   (IA)       │ ────────────────────────────────▶ │  127.0.0.1:8741  │
└──────────────┘  (patient_id, structured, IA)     └────────┬─────────┘
                                                            │ SQLite
┌──────────────┐                                            │
│     RIS      │  2. POST /open-report                      │
│  (Xplore…)   │ ──────────────────────────────────────────▶│
└──────────────┘  ?accession_number=XXX                     │
                                                            │
                  3. Événement Tauri → iframe navigue        │
                     https://airadcr.com/app?tori=true&tid=…│
                                                            │
┌──────────────┐  4. GET /pending-report?tid=XXX            │
│ airadcr.com  │ ◀─────────────────────────────────────────│
│  (iframe)    │  → Formulaire pré-rempli                   │
└──────────────┘                                            │
                  5. Radiologue dicte → Injection RIS       │
```

---

## 🔑 Différence Cloud vs Local

| Champ | Cloud | Local (Tauri) |
|-------|-------|---------------|
| `patient_id` | ❌ Interdit | ✅ **Accepté** |
| `exam_uid` | ❌ Interdit | ✅ **Accepté** |
| `accession_number` | ❌ Interdit | ✅ **Accepté** |
| `study_instance_uid` | ❌ Interdit | ✅ **Accepté** |
| **Transit** | Internet (HTTPS) | localhost uniquement |
| **Stockage** | Cloud | SQLite chiffré local |

> ⚠️ **Important** : Les identifiants patients sont acceptés car les données ne quittent jamais la machine locale (serveur sur `127.0.0.1` uniquement).

---

## 🔐 Authentification

### Clés API

| Header | Usage | Endpoints protégés |
|--------|-------|--------------------|
| `X-API-Key` | Opérations de données | `POST /pending-report`, `DELETE /pending-report`, `POST /open-report` |
| `X-Admin-Key` | Administration | `POST /api-keys`, `GET /api-keys`, `DELETE /api-keys/{prefix}` |

La clé API est hashée en **SHA-256** côté serveur avec comparaison en temps constant. Aucune clé n'est stockée en clair.

### Endpoints sans authentification

| Endpoint | Raison |
|----------|--------|
| `GET /health` | Vérification de disponibilité |
| `GET /pending-report?tid=XXX` | Lecture par airadcr.com (configurable) |
| `GET /find-report` | Recherche RIS (configurable) |

> 💡 Pour exiger une API key sur les endpoints GET, définissez `require_auth_for_reads = true` dans le fichier de configuration (`%APPDATA%/airadcr-desktop/config.toml`).

### Créer une clé API

```bash
curl -X POST http://127.0.0.1:8741/api-keys \
  -H "Content-Type: application/json" \
  -H "X-Admin-Key: VOTRE_CLE_ADMIN" \
  -d '{"name": "RIS Production"}'
```

Réponse :
```json
{
  "success": true,
  "id": "uuid",
  "key": "airadcr_xxxxxxxxxxxxxxxxxxxxxxxxx",
  "name": "RIS Production",
  "message": "API key created successfully. Store this key securely - it won't be shown again."
}
```

> ⚠️ **La clé complète n'est affichée qu'une seule fois.** Sauvegardez-la immédiatement.

---

## 📡 Endpoints API

### 1. `GET /health` — Vérification de disponibilité

```http
GET http://127.0.0.1:8741/health
```

Réponse `200` :
```json
{
  "status": "ok",
  "version": "",
  "timestamp": "2026-02-23T10:00:00Z"
}
```

> ℹ️ Le champ `version` est volontairement masqué sans authentification pour raisons de sécurité.

---

### 2. `POST /pending-report` — Stocker un rapport ⭐

**Authentification** : `X-API-Key` obligatoire.

```http
POST http://127.0.0.1:8741/pending-report
Content-Type: application/json
X-API-Key: airadcr_xxxxxxxxx
```

**Corps de la requête :**

```json
{
  "technical_id": "TEO_ACC2024001_MR",

  "patient_id": "PAT123456",
  "exam_uid": "1.2.840.113619.2.XXX.YYY.ZZZ",
  "accession_number": "ACC2024001",
  "study_instance_uid": "1.2.840.10008.5.1.4.1.1.2.XXX",

  "structured": {
    "title": "IRM Cérébrale",
    "indication": "Céphalées chroniques depuis 3 mois",
    "technique": "IRM 3T avec injection gadolinium",
    "results": "Analyse IA TÉO Hub :\n- Volumétrie : normale\n- Aucune lésion détectée",
    "conclusion": ""
  },

  "source_type": "teo_hub",
  "ai_modules": ["brain_volumetry", "lesion_detection"],
  "modality": "MR",
  "metadata": {
    "teo_version": "2.1.0",
    "confidence_score": 0.94,
    "site_id": "SITE_001"
  },
  "expires_in_hours": 24
}
```

#### Champs obligatoires

| Champ | Type | Contraintes | Description |
|-------|------|-------------|-------------|
| `technical_id` | string | **Max 64 chars**, `[a-zA-Z0-9_-]` uniquement | Identifiant unique du rapport |
| `structured` | object | Requis | Contenu structuré du rapport |

#### Champs identifiants patients (✅ acceptés en local)

| Champ | Type | Description |
|-------|------|-------------|
| `patient_id` | string | ID patient RIS |
| `exam_uid` | string | UID DICOM de l'examen |
| `accession_number` | string | Numéro d'accession DICOM |
| `study_instance_uid` | string | Study Instance UID DICOM |

#### Champs optionnels

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `source_type` | string | `"tauri_local"` | Source (recommandé : `"teo_hub"`) |
| `ai_modules` | string[] | `null` | Modules IA utilisés |
| `modality` | string | `null` | Modalité DICOM (MR, CT, US…) |
| `metadata` | object | `null` | Métadonnées libres (JSON) |
| `expires_in_hours` | int | `24` | Durée de vie en heures |

#### Structure `structured`

| Champ | Type | Description |
|-------|------|-------------|
| `title` | string | Titre du rapport (ex : "IRM Cérébrale") |
| `indication` | string | Indication clinique |
| `technique` | string | Protocole technique |
| `results` | string | Résultats IA pré-remplis |
| `conclusion` | string | Conclusion (vide, à compléter par radiologue) |

**Réponse succès `200` :**
```json
{
  "success": true,
  "technical_id": "TEO_ACC2024001_MR",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=TEO_ACC2024001_MR",
  "expires_at": "2026-02-24T10:00:00Z"
}
```

**Erreurs :**

| Code | Cause | Exemple |
|------|-------|---------|
| `400` | Validation échouée | `{"error": "technical_id must contain only alphanumeric characters, hyphens, and underscores", "field": "technical_id"}` |
| `401` | Clé API invalide | `{"error": "Invalid API key"}` |
| `500` | Erreur serveur | `{"error": "Database error: ..."}` |

---

### 3. `GET /pending-report?tid=XXX` — Récupérer un rapport

**Authentification** : Aucune par défaut (configurable via `require_auth_for_reads`).

```http
GET http://127.0.0.1:8741/pending-report?tid=TEO_ACC2024001_MR
```

**Réponse `200` :**
```json
{
  "success": true,
  "data": {
    "technical_id": "TEO_ACC2024001_MR",
    "patient_id": "PAT123456",
    "exam_uid": "1.2.840.113619.2.XXX.YYY.ZZZ",
    "accession_number": "ACC2024001",
    "study_instance_uid": "1.2.840.10008.5.1.4.1.1.2.XXX",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées chroniques",
      "technique": "IRM 3T avec injection",
      "results": "Analyse IA...",
      "conclusion": ""
    },
    "source_type": "teo_hub",
    "ai_modules": ["brain_volumetry", "lesion_detection"],
    "modality": "MR",
    "metadata": { "teo_version": "2.1.0" },
    "status": "retrieved",
    "created_at": "2026-02-23T10:00:00Z"
  }
}
```

> ℹ️ Le statut passe automatiquement à `"retrieved"` après le premier GET.

**Erreurs** : `400` (tid manquant), `404` (rapport non trouvé ou expiré).

---

### 4. `DELETE /pending-report?tid=XXX` — Supprimer un rapport

**Authentification** : `X-API-Key` obligatoire.

```http
DELETE http://127.0.0.1:8741/pending-report?tid=TEO_ACC2024001_MR
X-API-Key: airadcr_xxxxxxxxx
```

Réponse `200` : `{"success": true, "deleted": true}`

---

### 5. `GET /find-report` — Rechercher par identifiants RIS 🔍

**Authentification** : Aucune par défaut (configurable via `require_auth_for_reads`).

Permet au RIS de chercher un rapport **sans connaître le `technical_id`**.

```http
GET http://127.0.0.1:8741/find-report?accession_number=ACC2024001
GET http://127.0.0.1:8741/find-report?patient_id=PAT123&accession_number=ACC2024001
GET http://127.0.0.1:8741/find-report?exam_uid=1.2.3.4.5.6.7.8.9
```

**Paramètres** (au moins un requis) :

| Paramètre | Type | Description |
|-----------|------|-------------|
| `accession_number` | string | Numéro d'accession DICOM |
| `patient_id` | string | ID patient RIS |
| `exam_uid` | string | UID DICOM de l'examen |

**Réponse `200` :**
```json
{
  "success": true,
  "data": { "technical_id": "...", "...": "..." },
  "retrieval_url": "http://127.0.0.1:8741/pending-report?tid=TEO_ACC2024001_MR"
}
```

**Erreurs** : `400` (aucun identifiant fourni), `404` (aucun rapport trouvé).

---

### 6. `POST /open-report` — Ouvrir un rapport dans AIRADCR 🚀

**Authentification** : `X-API-Key` obligatoire.

Déclenche automatiquement la navigation de l'iframe AIRADCR vers le rapport ET met la fenêtre au premier plan.

```http
POST http://127.0.0.1:8741/open-report?accession_number=ACC2024001
X-API-Key: airadcr_xxxxxxxxx

# Ou directement par technical_id
POST http://127.0.0.1:8741/open-report?tid=TEO_ACC2024001_MR
X-API-Key: airadcr_xxxxxxxxx
```

**Paramètres** (au moins un requis, `tid` prioritaire) :

| Paramètre | Type | Priorité | Description |
|-----------|------|----------|-------------|
| `tid` | string | 1 (direct) | `technical_id` du rapport |
| `accession_number` | string | 2 (recherche) | Numéro d'accession |
| `patient_id` | string | 2 | ID patient |
| `exam_uid` | string | 2 | UID examen |

**Comportement interne :**

1. Si `tid` fourni → utilisation directe
2. Sinon → recherche SQLite par identifiants RIS
3. Validation du TID (max 64 chars, `[a-zA-Z0-9_-]`)
4. Émission événement Tauri `airadcr:navigate_to_report`
5. L'iframe navigue vers `https://airadcr.com/app?tori=true&tid=XXX`
6. La fenêtre AIRADCR passe au premier plan (show + focus)

**Réponse `200` :**
```json
{
  "success": true,
  "message": "Navigation triggered successfully",
  "technical_id": "TEO_ACC2024001_MR",
  "navigated_to": "https://airadcr.com/app?tori=true&tid=TEO_ACC2024001_MR"
}
```

**Erreurs** : `400` (aucun identifiant / TID invalide), `401` (API key manquante), `404` (rapport non trouvé), `503` (application pas encore prête, `Retry-After: 2`).

---

## 🔒 Sécurité

### Rate Limiting
- **60 requêtes/minute** par IP (burst de 60 autorisé)

### CORS
Origines autorisées : `http://localhost:*`, `https://airadcr.com`, `https://www.airadcr.com`

### Payload maximum
- **1 MB** maximum par requête JSON

### Masquage des PII dans les logs
Les identifiants patients sont masqués : `PAT123456` → `PAT1****`

### Expiration et nettoyage
- Rapports expirés après **24 heures** (configurable)
- Nettoyage automatique toutes les **heures** (configurable via `cleanup_interval_secs`)

---

## 🗄️ Schéma SQLite

```sql
CREATE TABLE pending_reports (
    id TEXT PRIMARY KEY,
    technical_id TEXT UNIQUE NOT NULL,
    
    -- Identifiants patients (LOCAL UNIQUEMENT)
    patient_id TEXT,
    exam_uid TEXT,
    accession_number TEXT,
    study_instance_uid TEXT,
    
    -- Données structurées
    structured_data TEXT NOT NULL,
    source_type TEXT DEFAULT 'tauri_local',
    ai_modules TEXT,
    modality TEXT,
    metadata TEXT,
    
    -- Statut et timing
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'retrieved', 'expired')),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    retrieved_at TEXT
);

CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    key_prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    name TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE access_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    method TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('success', 'unauthorized', 'not_found', 'error', 'bad_request')),
    api_key_prefix TEXT,
    user_agent TEXT,
    request_id TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    error_message TEXT
);

-- Index de performance
CREATE INDEX idx_pending_technical_id ON pending_reports(technical_id);
CREATE INDEX idx_pending_patient_id ON pending_reports(patient_id);
CREATE INDEX idx_pending_accession ON pending_reports(accession_number);
CREATE INDEX idx_pending_exam_uid ON pending_reports(exam_uid);
CREATE INDEX idx_pending_status ON pending_reports(status);
CREATE INDEX idx_pending_expires ON pending_reports(expires_at);
CREATE INDEX idx_pending_created_at ON pending_reports(created_at);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
CREATE INDEX idx_access_logs_timestamp ON access_logs(timestamp);
CREATE INDEX idx_access_logs_endpoint ON access_logs(endpoint);
CREATE INDEX idx_access_logs_result ON access_logs(result);
CREATE INDEX idx_access_logs_ip ON access_logs(ip_address);
CREATE INDEX idx_access_logs_timestamp_result ON access_logs(timestamp, result);
```

---

## ⚙️ Configuration

Fichier : `%APPDATA%/airadcr-desktop/config.toml`

```toml
http_port = 8741
log_level = "info"
log_retention_days = 30
report_retention_hours = 24
iframe_url = "https://airadcr.com/app?tori=true"
backup_enabled = true
backup_retention_days = 7
cleanup_interval_secs = 3600

[teo_hub]
enabled = false
host = "192.168.1.253"
port = 54489
health_endpoint = "th_health"
get_report_endpoint = "th_get_ai_report"
post_report_endpoint = "th_post_approved_report"
timeout_secs = 30
retry_count = 3
retry_delay_ms = 1000
tls_enabled = false
```

> ⚠️ Le token TÉO Hub (`api_token`) est stocké dans le **keychain OS** (Windows Credential Manager / macOS Keychain), pas dans le fichier TOML. Si un token est trouvé dans le fichier, il est automatiquement migré vers le keychain et supprimé du fichier.

---

## 🔄 Workflow Complet : TÉO Hub → RIS → AIRADCR

```
┌──────────┐     ┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│ TÉO Hub  │     │ AIRADCR Desktop │     │ airadcr.com  │     │     RIS     │
│   (IA)   │     │ 127.0.0.1:8741  │     │   (iframe)   │     │  (cible)    │
└────┬─────┘     └────────┬────────┘     └──────┬───────┘     └──────┬──────┘
     │                    │                     │                    │
     │ 1. POST /pending-report                  │                    │
     │ X-API-Key: airadcr_xxx                   │                    │
     │ {technical_id, patient_id, structured}   │                    │
     │──────────────────>│                      │                    │
     │                   │ SQLite               │                    │
     │ 2. 200 OK         │                      │                    │
     │ {technical_id, retrieval_url}            │                    │
     │<──────────────────│                      │                    │
     │                   │                      │                    │
     │ 3. Notifier RIS (accession + tid)        │                    │
     │─────────────────────────────────────────────────────────────>│
     │                   │                      │                    │
     │                   │                      │ 4. POST /open-report│
     │                   │                      │ X-API-Key: xxx      │
     │                   │                      │ ?accession=ACC001   │
     │                   │◀─────────────────────────────────────────│
     │                   │                      │                    │
     │                   │ 5. Événement Tauri   │                    │
     │                   │ airadcr:navigate     │                    │
     │                   │─────────────────────>│                    │
     │                   │                      │                    │
     │                   │ 6. GET /pending-report?tid=XXX            │
     │                   │◀─────────────────────│                    │
     │                   │                      │                    │
     │                   │ 7. Données rapport   │                    │
     │                   │─────────────────────>│                    │
     │                   │                      │                    │
     │                   │                      │ 8. Formulaire      │
     │                   │                      │    pré-rempli      │
     │                   │                      │    IA + patient     │
     │                   │                      │                    │
     │                   │                      │ 9. Dictée →        │
     │                   │                      │    Validation      │
     │                   │                      │                    │
     │                   │ 10. postMessage      │                    │
     │                   │     airadcr:inject   │                    │
     │                   │◀─────────────────────│                    │
     │                   │                      │                    │
     │                   │ 11. Injection clavier─────────────────────>│
     │                   │     (Ctrl+V dans RIS)│                    │
```

### Étapes en détail

#### Étape 1-2 : TÉO Hub stocke le rapport IA

TÉO Hub analyse les images DICOM et envoie le rapport structuré :

```bash
curl -X POST http://127.0.0.1:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: VOTRE_CLE_API" \
  -d '{
    "technical_id": "TEO_ACC2024001_MR",
    "patient_id": "PAT123456",
    "accession_number": "ACC2024001",
    "exam_uid": "1.2.3.4.5.6.7.8.9",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées chroniques",
      "technique": "IRM 3T séquences T1, T2, FLAIR, diffusion",
      "results": "VOLUMÉTRIE HIPPOCAMPIQUE:\n- Droit: 3.2 cm³ (normal)\n- Gauche: 3.1 cm³ (normal)\n\nANALYSE LÉSIONNELLE:\n- Aucune lésion focale détectée",
      "conclusion": ""
    },
    "source_type": "teo_hub",
    "ai_modules": ["hippocampal_volumetry", "lesion_detection"],
    "modality": "MR"
  }'
```

#### Étape 3 : TÉO Hub notifie le RIS

TÉO Hub informe le RIS que le rapport est prêt (via HL7, API, ou webhook selon intégration).

#### Étape 4-5 : Le RIS ouvre le rapport

Quand le radiologue clique "Ouvrir dans AIRADCR" dans le RIS :

```bash
curl -X POST "http://127.0.0.1:8741/open-report?accession_number=ACC2024001" \
  -H "X-API-Key: VOTRE_CLE_API"
```

→ AIRADCR passe automatiquement au premier plan et l'iframe navigue vers l'examen.

#### Étape 6-8 : airadcr.com récupère et pré-remplit

L'iframe navigue vers `https://airadcr.com/app?tori=true&tid=TEO_ACC2024001_MR` qui appelle automatiquement `GET /pending-report?tid=...` et pré-remplit le formulaire.

#### Étape 9-11 : Dictée et injection

Le radiologue dicte, valide, puis le rapport est injecté dans le RIS via le système d'injection clavier.

---

## 🧪 Tests cURL complets

```bash
# 1. Vérifier le desktop
curl http://127.0.0.1:8741/health

# 2. TÉO Hub stocke un rapport
curl -X POST http://127.0.0.1:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: VOTRE_CLE_API" \
  -d '{
    "technical_id": "TEST_001",
    "patient_id": "PAT123456",
    "accession_number": "ACC001",
    "structured": {"title": "Radio Thorax", "indication": "Toux"},
    "modality": "CR"
  }'

# 3. RIS recherche par accession_number
curl "http://127.0.0.1:8741/find-report?accession_number=ACC001"

# 4. RIS ouvre le rapport dans AIRADCR
curl -X POST "http://127.0.0.1:8741/open-report?accession_number=ACC001" \
  -H "X-API-Key: VOTRE_CLE_API"

# 5. Récupérer le rapport (fait automatiquement par airadcr.com)
curl "http://127.0.0.1:8741/pending-report?tid=TEST_001"

# 6. Supprimer
curl -X DELETE "http://127.0.0.1:8741/pending-report?tid=TEST_001" \
  -H "X-API-Key: VOTRE_CLE_API"
```

---

## 🔗 Deep Links (protocole `airadcr://`)

L'application supporte aussi le lancement via protocole URL enregistré dans Windows :

```
airadcr://open?tid=TEO_ACC2024001_MR
airadcr://open/TEO_ACC2024001_MR
airadcr://TEO_ACC2024001_MR
```

Le TID est validé : max 64 caractères, `[a-zA-Z0-9_-]` uniquement.

---

## 📋 Résumé des authentifications par endpoint

| Endpoint | Méthode | Auth requise | Header |
|----------|---------|--------------|--------|
| `/health` | GET | ❌ Non | — |
| `/health/extended` | GET | ❌ Non | — |
| `/metrics` | GET | ❌ Non | — |
| `/pending-report` | POST | ✅ Oui | `X-API-Key` |
| `/pending-report` | GET | ⚙️ Configurable | `X-API-Key` (si `require_auth_for_reads`) |
| `/pending-report` | DELETE | ✅ Oui | `X-API-Key` |
| `/find-report` | GET | ⚙️ Configurable | `X-API-Key` (si `require_auth_for_reads`) |
| `/open-report` | POST | ✅ Oui | `X-API-Key` |
| `/api-keys` | POST | ✅ Admin | `X-Admin-Key` |
| `/api-keys` | GET | ✅ Admin | `X-Admin-Key` |
| `/api-keys/{prefix}` | DELETE | ✅ Admin | `X-Admin-Key` |

---

## ❓ FAQ

### Q: Les identifiants patients sont-ils sécurisés ?

**Oui** : le serveur écoute uniquement sur `127.0.0.1`, les données sont en SQLite chiffré local, et les identifiants sont masqués dans les logs (`PAT1****`).

### Q: Le RIS doit-il connaître le `technical_id` de TÉO Hub ?

**Non.** Le RIS peut utiliser `accession_number`, `patient_id` ou `exam_uid` pour rechercher (`/find-report`) et ouvrir (`/open-report`).

### Q: Quelle différence entre `/find-report` et `/open-report` ?

- **`/find-report`** (GET) : recherche et retourne les données (lecture seule)
- **`/open-report`** (POST) : recherche ET déclenche la navigation + focus fenêtre

### Q: Que se passe-t-il si le port 8741 est occupé ?

Le serveur tente automatiquement les ports `8742` et `8743` en fallback.

### Q: Plusieurs rapports pour le même patient ?

La recherche retourne le rapport le plus récent. Utilisez des identifiants plus spécifiques pour cibler un examen précis.

---

*Document mis à jour le 2026-02-23 — Version 2.0.0*
