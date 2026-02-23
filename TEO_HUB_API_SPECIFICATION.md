# 📋 Spécification API AIRADCR Desktop pour TÉO Hub

**Version** : 2.0.0  
**Date** : Février 2026  
**Contact** : contact@airadcr.com  
**Base URL** : `http://127.0.0.1:8741`

---

## 📑 Table des Matières

1. [Vue d'Ensemble](#1-vue-densemble)
2. [Architecture et Flux de Données](#2-architecture-et-flux-de-données)
3. [Authentification](#3-authentification)
4. [Endpoints API](#4-endpoints-api)
5. [Exemples de Code Python](#5-exemples-de-code-python)
6. [Exemples de Code C#](#6-exemples-de-code-c)
7. [Script Orthanc (Lua)](#7-script-orthanc-lua)
8. [Gestion des Erreurs](#8-gestion-des-erreurs)
9. [Bonnes Pratiques](#9-bonnes-pratiques)
10. [Tests et Validation](#10-tests-et-validation)
11. [Configuration](#11-configuration)
12. [Annexes](#12-annexes)

---

## 1. Vue d'Ensemble

### 1.1 Objectif

Cette API permet à TÉO Hub et aux systèmes RIS/PACS d'envoyer des rapports radiologiques pré-traités par IA au desktop AIRADCR. Les rapports sont stockés localement en SQLite chiffré et automatiquement chargés dans l'interface de dictée airadcr.com.

### 1.2 Avantages du Mode Local

| Aspect | Cloud | Local (Tauri Desktop) |
|--------|-------|----------------------|
| **patient_id** | ❌ Interdit | ✅ **Accepté** |
| **exam_uid** | ❌ Interdit | ✅ **Accepté** |
| **accession_number** | ❌ Interdit | ✅ **Accepté** |
| **study_instance_uid** | ❌ Interdit | ✅ **Accepté** |
| **Stockage** | Cloud | SQLite chiffré local |
| **Transit** | Internet (HTTPS) | localhost uniquement |
| **Sécurité** | RLS + API Key | API Key + SHA-256 |

> ⚠️ **Important** : Les identifiants patients sont acceptés car les données ne quittent jamais la machine locale.

---

## 2. Architecture et Flux de Données

### 2.1 Diagramme de Séquence

```
┌──────────┐     ┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│ TÉO Hub  │     │ AIRADCR Desktop │     │ airadcr.com  │     │     RIS     │
│   (IA)   │     │ 127.0.0.1:8741  │     │   (iframe)   │     │  (cible)    │
└────┬─────┘     └────────┬────────┘     └──────┬───────┘     └──────┬──────┘
     │                    │                     │                    │
     │ 1. POST /pending-report                  │                    │
     │ X-API-Key: airadcr_xxx                   │                    │
     │ (patient_id, structured, ai_modules)     │                    │
     │──────────────────>│                      │                    │
     │                   │ SQLite               │                    │
     │ 2. 200 OK         │                      │                    │
     │ (technical_id, retrieval_url)            │                    │
     │<──────────────────│                      │                    │
     │                   │                      │                    │
     │ 3. Notifier RIS avec accession_number    │                    │
     │─────────────────────────────────────────────────────────────>│
     │                   │                      │                    │
     │                   │                      │ 4. POST /open-report│
     │                   │                      │ X-API-Key: xxx      │
     │                   │                      │ ?accession=ACC001   │
     │                   │<─────────────────────────────────────────│
     │                   │                      │                    │
     │                   │ 5. Événement Tauri   │                    │
     │                   │    navigate_to_report│                    │
     │                   │─────────────────────>│                    │
     │                   │                      │                    │
     │                   │ 6. GET /pending-report?tid=XXX            │
     │                   │<─────────────────────│                    │
     │                   │ 7. Données complètes │                    │
     │                   │─────────────────────>│                    │
     │                   │                      │                    │
     │                   │                      │ 8. Formulaire      │
     │                   │                      │    pré-rempli      │
     │                   │                      │                    │
     │                   │                      │ 9. Dictée + Valid. │
     │                   │                      │                    │
     │                   │ 10. postMessage      │                    │
     │                   │     airadcr:inject   │                    │
     │                   │<─────────────────────│                    │
     │                   │                      │                    │
     │                   │ 11. Injection clavier─────────────────────>│
     │                   │     (Ctrl+V → RIS)   │                    │
```

### 2.2 Stockage Local

- **Base de données** : SQLite chiffré (SQLCipher AES-256)
- **Emplacement** : `%APPDATA%/airadcr-desktop/pending_reports.db`
- **Expiration** : 24 heures par défaut (configurable)
- **Nettoyage** : Automatique toutes les heures (configurable via `cleanup_interval_secs`)
- **Backup** : Automatique quotidien (configurable)

---

## 3. Authentification

### 3.1 Types de clés

| Type | Header | Usage |
|------|--------|-------|
| **API Key** | `X-API-Key` | Opérations de données (POST, DELETE) |
| **Admin Key** | `X-Admin-Key` | Gestion des clés API |

### 3.2 Matrice d'authentification

| Endpoint | Méthode | Auth | Header |
|----------|---------|------|--------|
| `/health` | GET | ❌ | — |
| `/pending-report` | POST | ✅ | `X-API-Key` |
| `/pending-report` | GET | ⚙️ | `X-API-Key` si `require_auth_for_reads` |
| `/pending-report` | DELETE | ✅ | `X-API-Key` |
| `/find-report` | GET | ⚙️ | `X-API-Key` si `require_auth_for_reads` |
| `/open-report` | POST | ✅ | `X-API-Key` |
| `/api-keys` | POST/GET/DELETE | ✅ | `X-Admin-Key` |

### 3.3 Sécurité des clés

- Clé hashée en **SHA-256** côté serveur
- Comparaison en **temps constant** (protection timing attacks)
- Seul le **préfixe** (8 chars) est stocké en clair pour recherche rapide
- Le hash complet est comparé pour validation finale

### 3.4 Créer une clé API

```bash
curl -X POST http://127.0.0.1:8741/api-keys \
  -H "Content-Type: application/json" \
  -H "X-Admin-Key: VOTRE_CLE_ADMIN" \
  -d '{"name": "TÉO Hub Production"}'
```

Réponse `201` :
```json
{
  "success": true,
  "id": "uuid-auto-genere",
  "key": "airadcr_xxxxxxxxxxxxxxxxxxxxxxxxx",
  "name": "TÉO Hub Production",
  "message": "API key created successfully. Store this key securely - it won't be shown again."
}
```

> ⚠️ **La clé complète n'est retournée qu'une seule fois.** Sauvegardez-la immédiatement.

---

## 4. Endpoints API

### 4.1 GET /health

Vérifie la disponibilité du desktop AIRADCR.

```http
GET http://127.0.0.1:8741/health
```

Réponse `200` :
```json
{
  "status": "ok",
  "version": "",
  "timestamp": "2026-02-23T10:30:00Z"
}
```

> ℹ️ La version est masquée sans authentification (sécurité).

**Usage recommandé :** Toujours appeler avant `POST /pending-report` pour vérifier que le desktop est lancé.

---

### 4.2 POST /pending-report ⭐ (Endpoint Principal)

Stocke un rapport pré-traité par TÉO Hub.

**Authentification** : `X-API-Key` obligatoire.

```http
POST http://127.0.0.1:8741/pending-report
Content-Type: application/json
X-API-Key: airadcr_xxxxxxxxx
```

**Corps de la requête :**

```json
{
  "technical_id": "TEO_2024_12345",
  
  "patient_id": "PAT123456",
  "exam_uid": "1.2.840.113619.2.XXX.YYY.ZZZ",
  "accession_number": "ACC2024001",
  "study_instance_uid": "1.2.840.10008.5.1.4.1.1.2.XXX",
  
  "structured": {
    "title": "IRM Cérébrale",
    "indication": "Céphalées chroniques depuis 3 mois, recherche de lésion",
    "technique": "IRM 3T avec injection de gadolinium. Séquences T1, T2, FLAIR, diffusion",
    "results": "Analyse IA TÉO Hub :\n- Volumétrie cérébrale : normale pour l'âge\n- Aucune lésion suspecte détectée\n- Ventricules de taille normale",
    "conclusion": ""
  },
  
  "source_type": "teo_hub",
  "ai_modules": ["brain_volumetry", "lesion_detection", "white_matter_analysis"],
  "modality": "MR",
  "metadata": {
    "teo_version": "2.1.0",
    "processing_time_ms": 1523,
    "confidence_score": 0.94,
    "site_id": "SITE_001"
  },
  "expires_in_hours": 24
}
```

#### Champs Obligatoires

| Champ | Type | Contraintes | Description |
|-------|------|-------------|-------------|
| `technical_id` | string | **Max 64 chars**, regex `[a-zA-Z0-9_-]` | Identifiant unique du rapport |
| `structured` | object | Requis, JSON libre | Contenu structuré du rapport |

#### Champs Identifiants Patients (✅ Acceptés en Local)

| Champ | Type | Description |
|-------|------|-------------|
| `patient_id` | string | ID patient local/RIS |
| `exam_uid` | string | UID DICOM de l'examen |
| `accession_number` | string | Numéro d'accession DICOM |
| `study_instance_uid` | string | Study Instance UID DICOM |

#### Champs Optionnels

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `source_type` | string | `"tauri_local"` | Source du rapport (recommandé : `"teo_hub"`) |
| `ai_modules` | string[] | `null` | Modules IA utilisés |
| `modality` | string | `null` | Modalité DICOM (MR, CT, US, CR, etc.) |
| `metadata` | object | `null` | Métadonnées libres (JSON) |
| `expires_in_hours` | int | `24` | Durée de vie en heures |

#### Structure `structured`

| Champ | Type | Description |
|-------|------|-------------|
| `title` | string | Titre du rapport (ex: "IRM Cérébrale") |
| `indication` | string | Indication clinique |
| `technique` | string | Protocole technique utilisé |
| `results` | string | Résultats de l'analyse IA (pré-rempli par TÉO Hub) |
| `conclusion` | string | Conclusion (vide = à compléter par le radiologue) |

**Réponse `200` (Succès) :**
```json
{
  "success": true,
  "technical_id": "TEO_2024_12345",
  "retrieval_url": "https://airadcr.com/app?tori=true&tid=TEO_2024_12345",
  "expires_at": "2026-02-24T10:30:00Z"
}
```

**Réponses d'erreur :**

| Code | Description | Exemple |
|------|-------------|---------|
| `400` | Validation échouée | `{"error": "technical_id must be 64 characters or less", "field": "technical_id"}` |
| `401` | Clé API invalide | `{"error": "Invalid API key"}` |
| `500` | Erreur serveur | `{"error": "Database error: ..."}` |

---

### 4.3 GET /pending-report?tid=XXX

Récupère un rapport par son `technical_id`.

> ℹ️ Cet endpoint est principalement utilisé par airadcr.com (iframe), pas directement par TÉO Hub.

**Authentification** : Aucune par défaut (configurable via `require_auth_for_reads`).

```http
GET http://127.0.0.1:8741/pending-report?tid=TEO_2024_12345
```

**Réponse `200` :**
```json
{
  "success": true,
  "data": {
    "technical_id": "TEO_2024_12345",
    "patient_id": "PAT123456",
    "exam_uid": "1.2.840.113619.2.XXX.YYY.ZZZ",
    "accession_number": "ACC2024001",
    "study_instance_uid": "1.2.840.10008.5.1.4.1.1.2.XXX",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées chroniques",
      "technique": "IRM 3T avec injection",
      "results": "Analyse IA TÉO Hub...",
      "conclusion": ""
    },
    "source_type": "teo_hub",
    "ai_modules": ["brain_volumetry", "lesion_detection"],
    "modality": "MR",
    "metadata": { "teo_version": "2.1.0", "confidence_score": 0.94 },
    "status": "retrieved",
    "created_at": "2026-02-23T10:30:00Z"
  }
}
```

> Le statut passe automatiquement de `"pending"` à `"retrieved"` après le premier GET.

---

### 4.4 DELETE /pending-report?tid=XXX

**Authentification** : `X-API-Key` obligatoire.

```http
DELETE http://127.0.0.1:8741/pending-report?tid=TEO_2024_12345
X-API-Key: airadcr_xxxxxxxxx
```

Réponse `200` : `{"success": true, "deleted": true}`

---

### 4.5 GET /find-report 🔍 (Recherche RIS)

Recherche un rapport par identifiants RIS sans connaître le `technical_id`.

**Authentification** : Aucune par défaut (configurable).

```http
GET http://127.0.0.1:8741/find-report?accession_number=ACC2024001
GET http://127.0.0.1:8741/find-report?patient_id=PAT123456
GET http://127.0.0.1:8741/find-report?patient_id=PAT123&accession_number=ACC2024001
```

**Paramètres** (au moins un requis) :

| Paramètre | Type | Description |
|-----------|------|-------------|
| `accession_number` | string | Numéro d'accession DICOM |
| `patient_id` | string | ID patient local/RIS |
| `exam_uid` | string | UID DICOM de l'examen |

**Réponse `200` :**
```json
{
  "success": true,
  "data": { "technical_id": "TEO_2024_12345", "...": "..." },
  "retrieval_url": "http://127.0.0.1:8741/pending-report?tid=TEO_2024_12345"
}
```

---

### 4.6 POST /open-report 🚀 (Ouverture Contextuelle)

Ouvre AIRADCR et navigue automatiquement vers un rapport.

**Authentification** : `X-API-Key` obligatoire.

```http
POST http://127.0.0.1:8741/open-report?accession_number=ACC2024001
X-API-Key: airadcr_xxxxxxxxx
```

**Paramètres** (au moins un requis, `tid` prioritaire) :

| Paramètre | Priorité | Description |
|-----------|----------|-------------|
| `tid` | 1 (direct) | `technical_id` du rapport |
| `accession_number` | 2 (recherche) | Numéro d'accession |
| `patient_id` | 2 | ID patient |
| `exam_uid` | 2 | UID examen |

**Comportement :**
1. Si `tid` → utilisation directe
2. Sinon → recherche SQLite par identifiants
3. Validation du TID (max 64 chars, `[a-zA-Z0-9_-]`)
4. Émission événement Tauri `airadcr:navigate_to_report`
5. Navigation iframe → `https://airadcr.com/app?tori=true&tid=XXX`
6. Fenêtre AIRADCR → premier plan (show + focus)

**Réponse `200` :**
```json
{
  "success": true,
  "message": "Navigation triggered successfully",
  "technical_id": "TEO_2024_12345",
  "navigated_to": "https://airadcr.com/app?tori=true&tid=TEO_2024_12345"
}
```

**Erreurs** :

| Code | Cause |
|------|-------|
| `400` | Aucun identifiant / TID invalide |
| `401` | API key manquante ou invalide |
| `404` | Rapport non trouvé |
| `503` | Application pas encore prête (`Retry-After: 2`) |

---

## 5. Exemples de Code Python

### 5.1 Client Python Complet

```python
"""
AIRADCR Desktop Client pour TÉO Hub
===================================
pip install requests
"""

import requests
import logging
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, asdict
from datetime import datetime

logger = logging.getLogger("airadcr_client")


@dataclass
class StructuredReport:
    """Structure du rapport radiologique."""
    title: str
    indication: str = ""
    technique: str = ""
    results: str = ""
    conclusion: str = ""


class AiradcrDesktopClient:
    """Client Python pour l'API AIRADCR Desktop."""
    
    def __init__(self, api_key: str, base_url: str = "http://127.0.0.1:8741", timeout: int = 10):
        self.api_key = api_key
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self._session = requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "X-API-Key": api_key
        })
    
    def is_desktop_available(self) -> bool:
        """Vérifie si le desktop AIRADCR est lancé."""
        try:
            r = self._session.get(f"{self.base_url}/health", timeout=2)
            return r.status_code == 200
        except requests.exceptions.RequestException:
            return False
    
    def store_report(
        self,
        technical_id: str,
        structured: StructuredReport,
        patient_id: Optional[str] = None,
        exam_uid: Optional[str] = None,
        accession_number: Optional[str] = None,
        study_instance_uid: Optional[str] = None,
        source_type: str = "teo_hub",
        ai_modules: Optional[List[str]] = None,
        modality: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None,
        expires_in_hours: int = 24
    ) -> Dict[str, Any]:
        """
        Stocke un rapport pré-rempli sur le desktop.
        
        Returns:
            {"success": True, "technical_id": "...", "retrieval_url": "...", "expires_at": "..."}
            ou {"success": False, "error": "..."}
        """
        payload = {
            "technical_id": technical_id,
            "structured": asdict(structured),
            "source_type": source_type,
            "expires_in_hours": expires_in_hours
        }
        for key, val in [("patient_id", patient_id), ("exam_uid", exam_uid),
                         ("accession_number", accession_number), 
                         ("study_instance_uid", study_instance_uid),
                         ("ai_modules", ai_modules), ("modality", modality),
                         ("metadata", metadata)]:
            if val is not None:
                payload[key] = val
        
        try:
            r = self._session.post(f"{self.base_url}/pending-report", json=payload, timeout=self.timeout)
            data = r.json()
            if r.status_code == 200:
                logger.info(f"✅ Rapport stocké: {data.get('retrieval_url')}")
                return {**data, "success": True}
            else:
                logger.error(f"❌ Erreur {r.status_code}: {data.get('error')}")
                return {"success": False, "error": data.get("error", f"HTTP {r.status_code}")}
        except requests.exceptions.RequestException as e:
            return {"success": False, "error": str(e)}
    
    def find_report(self, accession_number: str = None, patient_id: str = None, exam_uid: str = None) -> Optional[Dict]:
        """Recherche un rapport par identifiants RIS."""
        params = {k: v for k, v in [("accession_number", accession_number),
                                     ("patient_id", patient_id), ("exam_uid", exam_uid)] if v}
        if not params:
            return None
        try:
            r = self._session.get(f"{self.base_url}/find-report", params=params, timeout=self.timeout)
            return r.json() if r.status_code == 200 else None
        except requests.exceptions.RequestException:
            return None
    
    def open_report(self, tid: str = None, accession_number: str = None, 
                    patient_id: str = None, exam_uid: str = None) -> Dict[str, Any]:
        """Ouvre AIRADCR et navigue vers un rapport. La fenêtre passe au premier plan."""
        params = {k: v for k, v in [("tid", tid), ("accession_number", accession_number),
                                     ("patient_id", patient_id), ("exam_uid", exam_uid)] if v}
        if not params:
            return {"success": False, "error": "At least one identifier required"}
        try:
            r = self._session.post(f"{self.base_url}/open-report", params=params, timeout=self.timeout)
            return r.json()
        except requests.exceptions.RequestException as e:
            return {"success": False, "error": str(e)}


# =============================================================================
# EXEMPLE D'UTILISATION
# =============================================================================

if __name__ == "__main__":
    client = AiradcrDesktopClient(api_key="VOTRE_CLE_API")
    
    if not client.is_desktop_available():
        print("❌ Desktop AIRADCR non disponible")
        exit(1)
    
    # 1. TÉO Hub stocke le rapport IA
    result = client.store_report(
        technical_id=f"TEO_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
        structured=StructuredReport(
            title="IRM Cérébrale",
            indication="Céphalées chroniques",
            technique="IRM 3T séquences T1, T2, FLAIR, diffusion",
            results="Volumétrie normale. Aucune lésion détectée.",
            conclusion=""
        ),
        patient_id="PAT123456",
        accession_number="ACC2024001",
        ai_modules=["brain_volumetry", "lesion_detection"],
        modality="MR",
        metadata={"teo_version": "2.1.0", "confidence_score": 0.94}
    )
    
    if result["success"]:
        print(f"✅ Rapport stocké: {result['retrieval_url']}")
        
        # 2. RIS ouvre le rapport (par accession_number)
        nav = client.open_report(accession_number="ACC2024001")
        if nav["success"]:
            print(f"✅ AIRADCR ouvert: {nav['navigated_to']}")
```

---

## 6. Exemples de Code C#

```csharp
using System.Net.Http;
using System.Text.Json;

public class AiradcrClient
{
    private readonly HttpClient _http;
    private readonly string _baseUrl;

    public AiradcrClient(string apiKey, string baseUrl = "http://127.0.0.1:8741")
    {
        _baseUrl = baseUrl;
        _http = new HttpClient();
        _http.DefaultRequestHeaders.Add("X-API-Key", apiKey);
        _http.Timeout = TimeSpan.FromSeconds(10);
    }

    public async Task<bool> IsAvailableAsync()
    {
        try
        {
            var r = await _http.GetAsync($"{_baseUrl}/health");
            return r.IsSuccessStatusCode;
        }
        catch { return false; }
    }

    public async Task<JsonElement?> StoreReportAsync(object report)
    {
        var content = new StringContent(
            JsonSerializer.Serialize(report), 
            System.Text.Encoding.UTF8, 
            "application/json"
        );
        var r = await _http.PostAsync($"{_baseUrl}/pending-report", content);
        var json = await r.Content.ReadAsStringAsync();
        return JsonSerializer.Deserialize<JsonElement>(json);
    }

    public async Task<JsonElement?> OpenReportAsync(string accessionNumber)
    {
        var r = await _http.PostAsync(
            $"{_baseUrl}/open-report?accession_number={Uri.EscapeDataString(accessionNumber)}", 
            null
        );
        var json = await r.Content.ReadAsStringAsync();
        return JsonSerializer.Deserialize<JsonElement>(json);
    }
}
```

---

## 7. Script Orthanc (Lua)

Pour intégration directe Orthanc (PACS) → AIRADCR :

```lua
-- Hook déclenché quand une étude est stable dans Orthanc
function OnStableStudy(studyId, tags, metadata)
    local study = ParseJson(RestApiGet('/studies/' .. studyId))
    local mainDicomTags = study['MainDicomTags']
    
    local accessionNumber = mainDicomTags['AccessionNumber'] or ''
    local patientId = mainDicomTags['PatientID'] or ''
    local studyDescription = mainDicomTags['StudyDescription'] or ''
    local modality = mainDicomTags['ModalitiesInStudy'] or ''
    local studyInstanceUID = mainDicomTags['StudyInstanceUID'] or ''
    
    -- 1. POST /pending-report
    local report = {
        technical_id = 'ORTHANC_' .. accessionNumber,
        patient_id = patientId,
        accession_number = accessionNumber,
        study_instance_uid = studyInstanceUID,
        structured = {
            title = modality .. ' - ' .. studyDescription,
            indication = studyDescription,
            technique = '',
            results = '',
            conclusion = ''
        },
        source_type = 'orthanc',
        modality = modality
    }
    
    local headers = {
        ['Content-Type'] = 'application/json',
        ['X-API-Key'] = 'VOTRE_CLE_API'
    }
    
    HttpPost('http://127.0.0.1:8741/pending-report', DumpJson(report), headers)
    
    -- 2. POST /open-report
    HttpPost('http://127.0.0.1:8741/open-report?accession_number=' .. accessionNumber, '', headers)
    
    print('AIRADCR: rapport envoyé pour ' .. accessionNumber)
end
```

---

## 8. Gestion des Erreurs

### Codes HTTP

| Code | Signification | Action recommandée |
|------|---------------|-------------------|
| `200` | Succès | — |
| `400` | Paramètres invalides | Vérifier le payload |
| `401` | Clé API invalide | Vérifier `X-API-Key` |
| `404` | Rapport non trouvé | Vérifier le `tid` / identifiants |
| `429` | Rate limit atteint | Attendre 1 seconde et réessayer |
| `500` | Erreur serveur interne | Réessayer après délai |
| `503` | Application pas prête | Réessayer après `Retry-After` (2s) |

### Retry recommandé

```python
import time

def post_with_retry(url, data, headers, max_retries=3):
    for attempt in range(max_retries):
        try:
            r = requests.post(url, json=data, headers=headers, timeout=10)
            if r.status_code == 503:
                retry_after = int(r.headers.get('Retry-After', 2))
                time.sleep(retry_after)
                continue
            if r.status_code == 429:
                time.sleep(1)
                continue
            return r
        except requests.exceptions.ConnectionError:
            time.sleep(2 ** attempt)  # Backoff exponentiel
    return None
```

---

## 9. Bonnes Pratiques

### Pour TÉO Hub

1. **Toujours vérifier `/health`** avant d'envoyer un rapport
2. **Utiliser `source_type: "teo_hub"`** pour traçabilité
3. **Renseigner `ai_modules`** pour que le radiologue sache quelles IA ont analysé
4. **Renseigner `accession_number`** systématiquement pour permettre la recherche RIS
5. **Ne pas réutiliser un `technical_id`** — il doit être unique par examen

### Pour le RIS

1. **Utiliser `POST /open-report`** avec `accession_number` — pas besoin de connaître le `technical_id`
2. **Inclure `X-API-Key`** dans les appels POST (obligatoire depuis v2.0)
3. **Gérer le `503`** (application pas encore prête) avec retry
4. **Un seul appel suffit** : `/open-report` fait recherche + navigation + focus

### Sécurité

1. **Ne jamais stocker la clé API en clair** dans le code source du RIS — utiliser un fichier de config protégé
2. **Tourner les clés** régulièrement : créer une nouvelle clé → mettre à jour le RIS → révoquer l'ancienne
3. **Activer `require_auth_for_reads`** si le poste est partagé

---

## 10. Tests et Validation

### Script de test complet

```bash
#!/bin/bash
API_KEY="VOTRE_CLE_API"
BASE="http://127.0.0.1:8741"

echo "=== 1. Health Check ==="
curl -s "$BASE/health" | python3 -m json.tool

echo -e "\n=== 2. Store Report ==="
curl -s -X POST "$BASE/pending-report" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d '{
    "technical_id": "TEST_001",
    "patient_id": "PAT123",
    "accession_number": "ACC001",
    "structured": {"title": "Radio Thorax", "indication": "Toux", "technique": "", "results": "", "conclusion": ""},
    "modality": "CR"
  }' | python3 -m json.tool

echo -e "\n=== 3. Find Report ==="
curl -s "$BASE/find-report?accession_number=ACC001" | python3 -m json.tool

echo -e "\n=== 4. Open Report ==="
curl -s -X POST "$BASE/open-report?accession_number=ACC001" \
  -H "X-API-Key: $API_KEY" | python3 -m json.tool

echo -e "\n=== 5. Get Report ==="
curl -s "$BASE/pending-report?tid=TEST_001" | python3 -m json.tool

echo -e "\n=== 6. Delete Report ==="
curl -s -X DELETE "$BASE/pending-report?tid=TEST_001" \
  -H "X-API-Key: $API_KEY" | python3 -m json.tool

echo -e "\n=== Done ==="
```

---

## 11. Configuration

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

# require_auth_for_reads = true  # Décommenter pour exiger X-API-Key sur GET

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
# api_token est stocké dans le keychain OS, pas dans ce fichier
```

### Variables d'environnement

| Variable | Usage | Obligatoire |
|----------|-------|-------------|
| `AIRADCR_ADMIN_KEY` | Clé d'administration (min 32 chars) | ✅ En production |
| `AIRADCR_PROD_API_KEY` | Clé API pré-configurée au démarrage | ❌ Optionnel |
| `AIRADCR_ENV` | `production` pour mode prod | ❌ Optionnel |

---

## 12. Annexes

### A. Format `technical_id`

- **Longueur** : 1 à 64 caractères
- **Caractères autorisés** : `a-z`, `A-Z`, `0-9`, `-`, `_`
- **Exemples valides** : `TEO_ACC2024001_MR`, `EXAM-2024-001`, `patient_12345`
- **Exemples rejetés** : URLs, espaces, caractères spéciaux

### B. Rate limiting

60 requêtes/minute par IP avec burst autorisé de 60.

### C. Ports alternatifs

Si le port `8741` est occupé, le serveur tente automatiquement `8742` puis `8743`.

### D. Deep Links

```
airadcr://open?tid=TEO_ACC2024001_MR
airadcr://open/TEO_ACC2024001_MR
airadcr://TEO_ACC2024001_MR
```

---

*Document mis à jour le 2026-02-23 — Version 2.0.0*
