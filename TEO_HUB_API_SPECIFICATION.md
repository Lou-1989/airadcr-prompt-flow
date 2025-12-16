# 📋 Spécification API AIRADCR Desktop pour TÉO Hub

**Version** : 1.0.0  
**Date** : Décembre 2024  
**Contact** : contact@airadcr.com  
**Base URL** : `http://localhost:8741`

---

## 📑 Table des Matières

1. [Vue d'Ensemble](#1-vue-densemble)
2. [Architecture et Flux de Données](#2-architecture-et-flux-de-données)
3. [Authentification](#3-authentification)
4. [Endpoints API](#4-endpoints-api)
5. [Exemples de Code Python](#5-exemples-de-code-python)
6. [Exemples de Code C#](#6-exemples-de-code-c)
7. [Gestion des Erreurs](#7-gestion-des-erreurs)
8. [Bonnes Pratiques](#8-bonnes-pratiques)
9. [Tests et Validation](#9-tests-et-validation)
10. [Annexes](#10-annexes)

---

## 1. Vue d'Ensemble

### 1.1 Objectif

Cette API permet à TÉO Hub d'envoyer des rapports radiologiques pré-traités par IA au desktop AIRADCR. Les rapports sont stockés localement en SQLite et automatiquement chargés dans l'interface de dictée airadcr.com.

### 1.2 Avantages du Mode Local

| Aspect | Cloud (Supabase) | Local (Tauri Desktop) |
|--------|------------------|----------------------|
| **patient_id** | ❌ Interdit | ✅ **Accepté** |
| **exam_uid** | ❌ Interdit | ✅ **Accepté** |
| **accession_number** | ❌ Interdit | ✅ **Accepté** |
| **study_instance_uid** | ❌ Interdit | ✅ **Accepté** |
| **Stockage** | Cloud AWS | SQLite local |
| **Transit** | Internet (HTTPS) | localhost uniquement |
| **Sécurité** | RLS + API Key | API Key locale |

> ⚠️ **Important** : Les identifiants patients sont acceptés car les données ne quittent jamais la machine locale.

---

## 2. Architecture et Flux de Données

### 2.1 Diagramme de Séquence

```
┌──────────┐     ┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│ TÉO Hub  │     │ Tauri Desktop   │     │ airadcr.com  │     │     RIS     │
│   (IA)   │     │ localhost:8741  │     │   (iframe)   │     │  (cible)    │
└────┬─────┘     └────────┬────────┘     └──────┬───────┘     └──────┬──────┘
     │                    │                     │                    │
     │ 1. POST /pending-report                  │                    │
     │ (patient_id, structured, ai_modules)     │                    │
     │──────────────────>│                      │                    │
     │                   │                      │                    │
     │ 2. 201 Created    │                      │                    │
     │ (technical_id, retrieval_url)            │                    │
     │<──────────────────│                      │                    │
     │                   │                      │                    │
     │ 3. Notifier RIS avec technical_id        │                    │
     │─────────────────────────────────────────────────────────────>│
     │                   │                      │                    │
     │                   │                      │ 4. Ouvrir URL      │
     │                   │                      │ ?tid=XXX           │
     │                   │                      │<───────────────────│
     │                   │                      │                    │
     │                   │ 5. GET /pending-report?tid=XXX            │
     │                   │<─────────────────────│                    │
     │                   │                      │                    │
     │                   │ 6. 200 + données     │                    │
     │                   │─────────────────────>│                    │
     │                   │                      │                    │
     │                   │                      │ 7. Pré-remplir     │
     │                   │                      │    formulaire      │
     │                   │                      │                    │
     │                   │                      │ 8. Radiologue dicte│
     │                   │                      │    et valide       │
     │                   │                      │                    │
     │                   │ 9. postMessage       │                    │
     │                   │    airadcr:inject    │                    │
     │                   │<─────────────────────│                    │
     │                   │                      │                    │
     │                   │ 10. Injection clavier ──────────────────>│
     │                   │                      │                    │
```

### 2.2 Stockage Local

- **Base de données** : SQLite chiffré
- **Emplacement** : `%APPDATA%/airadcr-desktop/pending_reports.db`
- **Expiration** : 24 heures par défaut (configurable)
- **Nettoyage** : Automatique toutes les 10 minutes

---

## 3. Authentification

### 3.1 Clé API de Production

```
X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z
```

### 3.2 Format de Clé

```
airadcr_prod_[32 caractères alphanumériques]
```

### 3.3 Utilisation

Toute requête `POST` doit inclure le header :

```http
X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z
```

### 3.4 Sécurité

- Clé hashée en **SHA-256** côté serveur
- Validation contre base SQLite locale
- Aucune clé stockée en clair

---

## 4. Endpoints API

### 4.1 GET /health

Vérifie la disponibilité du desktop AIRADCR.

**Requête :**
```http
GET http://localhost:8741/health
```

**Réponse 200 :**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2024-12-16T10:30:00Z"
}
```

**Usage recommandé :** Toujours appeler avant `POST /pending-report`.

---

### 4.2 POST /pending-report ⭐ (Endpoint Principal)

Stocke un rapport pré-traité par TÉO Hub.

**Requête :**
```http
POST http://localhost:8741/pending-report
Content-Type: application/json
X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z
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

| Champ | Type | Description |
|-------|------|-------------|
| `technical_id` | string | Identifiant unique du rapport (max 100 chars) |
| `structured` | object | Contenu structuré du rapport |

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
| `source_type` | string | `"ris_local"` | Source du rapport (recommandé: `"teo_hub"`) |
| `ai_modules` | string[] | `[]` | Modules IA utilisés |
| `modality` | string | `null` | Modalité DICOM (MR, CT, US, etc.) |
| `metadata` | object | `{}` | Métadonnées libres |
| `expires_in_hours` | int | `24` | Durée de vie en heures |

#### Structure `structured`

| Champ | Type | Description |
|-------|------|-------------|
| `title` | string | Titre du rapport (ex: "IRM Cérébrale") |
| `indication` | string | Indication clinique |
| `technique` | string | Protocole technique utilisé |
| `results` | string | Résultats de l'analyse IA (pré-rempli) |
| `conclusion` | string | Conclusion (souvent vide, à compléter par radiologue) |

**Réponse 201 (Succès) :**
```json
{
  "success": true,
  "technical_id": "TEO_2024_12345",
  "retrieval_url": "http://localhost:8741/pending-report?tid=TEO_2024_12345",
  "expires_at": "2024-12-17T10:30:00Z",
  "message": "Report stored successfully"
}
```

**Réponses d'erreur :**

| Code | Description | Corps |
|------|-------------|-------|
| 400 | Validation échouée | `{"success": false, "error": "Missing required field: technical_id"}` |
| 401 | Clé API invalide | `{"success": false, "error": "Invalid or missing API key"}` |
| 409 | ID déjà existant | `{"success": false, "error": "Report with this technical_id already exists"}` |
| 500 | Erreur serveur | `{"success": false, "error": "Internal server error"}` |

---

### 4.3 GET /pending-report?tid=XXX

Récupère un rapport par son `technical_id`.

> ℹ️ **Note** : Cet endpoint est principalement utilisé par airadcr.com, pas par TÉO Hub.

**Requête :**
```http
GET http://localhost:8741/pending-report?tid=TEO_2024_12345
```

**Réponse 200 :**
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
      "indication": "Céphalées chroniques depuis 3 mois",
      "technique": "IRM 3T avec injection de gadolinium",
      "results": "Analyse IA TÉO Hub : Volumétrie normale...",
      "conclusion": ""
    },
    "source_type": "teo_hub",
    "ai_modules": ["brain_volumetry", "lesion_detection"],
    "modality": "MR",
    "metadata": {
      "teo_version": "2.1.0",
      "confidence_score": 0.94
    },
    "status": "retrieved",
    "created_at": "2024-12-16T10:30:00Z"
  }
}
```

**Réponse 404 :**
```json
{
  "success": false,
  "error": "Report not found or expired"
}
```

---

### 4.4 DELETE /pending-report?tid=XXX

Supprime un rapport après utilisation.

**Requête :**
```http
DELETE http://localhost:8741/pending-report?tid=TEO_2024_12345
```

**Réponse 200 :**
```json
{
  "success": true,
  "message": "Report deleted successfully"
}
```

---

## 5. Exemples de Code Python

### 5.1 Client Python Complet

```python
"""
AIRADCR Desktop Client pour TÉO Hub
===================================
Client Python pour l'intégration avec l'API AIRADCR Desktop.

Installation:
    pip install requests

Usage:
    from airadcr_client import AiradcrDesktopClient
    
    client = AiradcrDesktopClient("airadcr_prod_7f3k9m2x5p8w1q4v6n0z")
    if client.is_desktop_available():
        result = client.store_report(...)
"""

import requests
import json
import logging
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, asdict
from datetime import datetime

# Configuration du logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("airadcr_client")


@dataclass
class StructuredReport:
    """Structure du rapport radiologique."""
    title: str
    indication: str = ""
    technique: str = ""
    results: str = ""
    conclusion: str = ""


@dataclass
class StoreReportResponse:
    """Réponse du stockage de rapport."""
    success: bool
    technical_id: str = ""
    retrieval_url: str = ""
    expires_at: str = ""
    message: str = ""
    error: str = ""


class AiradcrDesktopClient:
    """
    Client Python pour l'API AIRADCR Desktop.
    
    Permet à TÉO Hub d'envoyer des rapports pré-traités par IA
    au desktop AIRADCR pour pré-remplissage automatique.
    
    Attributes:
        api_key: Clé API de production
        base_url: URL du serveur local (défaut: http://localhost:8741)
        timeout: Timeout des requêtes en secondes
    """
    
    def __init__(
        self, 
        api_key: str, 
        base_url: str = "http://localhost:8741",
        timeout: int = 10
    ):
        """
        Initialise le client AIRADCR.
        
        Args:
            api_key: Clé API (format: airadcr_prod_XXXXX)
            base_url: URL du serveur Tauri local
            timeout: Timeout des requêtes HTTP
        """
        self.api_key = api_key
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self._session = requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "X-API-Key": api_key
        })
    
    def is_desktop_available(self) -> bool:
        """
        Vérifie si le desktop AIRADCR est disponible.
        
        Returns:
            True si le desktop répond, False sinon.
            
        Example:
            >>> client = AiradcrDesktopClient("airadcr_prod_xxx")
            >>> if client.is_desktop_available():
            ...     print("Desktop prêt!")
        """
        try:
            response = self._session.get(
                f"{self.base_url}/health",
                timeout=2
            )
            if response.status_code == 200:
                data = response.json()
                logger.info(f"Desktop disponible - Version: {data.get('version', 'unknown')}")
                return True
            return False
        except requests.exceptions.RequestException as e:
            logger.warning(f"Desktop non disponible: {e}")
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
    ) -> StoreReportResponse:
        """
        Stocke un rapport pré-rempli sur le desktop AIRADCR.
        
        Args:
            technical_id: Identifiant unique du rapport
            structured: Contenu structuré du rapport
            patient_id: ID patient (accepté en local)
            exam_uid: UID DICOM de l'examen
            accession_number: Numéro d'accession DICOM
            study_instance_uid: Study Instance UID DICOM
            source_type: Source du rapport (défaut: "teo_hub")
            ai_modules: Liste des modules IA utilisés
            modality: Modalité DICOM (MR, CT, US, etc.)
            metadata: Métadonnées additionnelles
            expires_in_hours: Durée de vie en heures
            
        Returns:
            StoreReportResponse avec le résultat de l'opération
            
        Raises:
            requests.exceptions.RequestException: Si erreur réseau
            
        Example:
            >>> result = client.store_report(
            ...     technical_id="TEO_2024_001",
            ...     structured=StructuredReport(
            ...         title="IRM Cérébrale",
            ...         indication="Céphalées",
            ...         results="Analyse IA: Normal"
            ...     ),
            ...     patient_id="PAT123",
            ...     ai_modules=["brain_volumetry"]
            ... )
            >>> print(result.retrieval_url)
        """
        # Construction du payload
        payload: Dict[str, Any] = {
            "technical_id": technical_id,
            "structured": asdict(structured),
            "source_type": source_type,
            "expires_in_hours": expires_in_hours
        }
        
        # Identifiants patients (acceptés en LOCAL uniquement)
        if patient_id:
            payload["patient_id"] = patient_id
        if exam_uid:
            payload["exam_uid"] = exam_uid
        if accession_number:
            payload["accession_number"] = accession_number
        if study_instance_uid:
            payload["study_instance_uid"] = study_instance_uid
        
        # Champs optionnels
        if ai_modules:
            payload["ai_modules"] = ai_modules
        if modality:
            payload["modality"] = modality
        if metadata:
            payload["metadata"] = metadata
        
        logger.info(f"Envoi rapport {technical_id} vers desktop...")
        
        try:
            response = self._session.post(
                f"{self.base_url}/pending-report",
                json=payload,
                timeout=self.timeout
            )
            
            data = response.json()
            
            if response.status_code == 201:
                logger.info(f"Rapport stocké: {data.get('retrieval_url')}")
                return StoreReportResponse(
                    success=True,
                    technical_id=data.get("technical_id", ""),
                    retrieval_url=data.get("retrieval_url", ""),
                    expires_at=data.get("expires_at", ""),
                    message=data.get("message", "")
                )
            else:
                logger.error(f"Erreur {response.status_code}: {data.get('error')}")
                return StoreReportResponse(
                    success=False,
                    error=data.get("error", f"HTTP {response.status_code}")
                )
                
        except requests.exceptions.Timeout:
            logger.error("Timeout lors de l'envoi du rapport")
            return StoreReportResponse(success=False, error="Request timeout")
        except requests.exceptions.RequestException as e:
            logger.error(f"Erreur réseau: {e}")
            return StoreReportResponse(success=False, error=str(e))
    
    def get_report(self, technical_id: str) -> Optional[Dict[str, Any]]:
        """
        Récupère un rapport par son technical_id.
        
        Args:
            technical_id: Identifiant du rapport
            
        Returns:
            Données du rapport ou None si non trouvé
        """
        try:
            response = self._session.get(
                f"{self.base_url}/pending-report",
                params={"tid": technical_id},
                timeout=self.timeout
            )
            
            if response.status_code == 200:
                data = response.json()
                return data.get("data")
            return None
            
        except requests.exceptions.RequestException:
            return None
    
    def delete_report(self, technical_id: str) -> bool:
        """
        Supprime un rapport.
        
        Args:
            technical_id: Identifiant du rapport
            
        Returns:
            True si supprimé, False sinon
        """
        try:
            response = self._session.delete(
                f"{self.base_url}/pending-report",
                params={"tid": technical_id},
                timeout=self.timeout
            )
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False


# =============================================================================
# EXEMPLE D'UTILISATION
# =============================================================================

if __name__ == "__main__":
    # Initialisation du client
    client = AiradcrDesktopClient(
        api_key="airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
    )
    
    # Vérifier la disponibilité du desktop
    if not client.is_desktop_available():
        print("❌ Desktop AIRADCR non disponible")
        exit(1)
    
    print("✅ Desktop AIRADCR disponible")
    
    # Créer un rapport structuré
    report = StructuredReport(
        title="IRM Cérébrale",
        indication="Céphalées chroniques depuis 3 mois, recherche de lésion expansive",
        technique="IRM 3T avec injection de gadolinium. Séquences T1, T2, FLAIR, diffusion, perfusion",
        results="""Analyse IA TÉO Hub (v2.1.0) :

VOLUMÉTRIE CÉRÉBRALE :
- Volume cérébral total : 1450 cm³ (percentile 55 pour l'âge)
- Ratio ventricules/cerveau : 2.1% (normal < 3%)

ANALYSE DES LÉSIONS :
- Score de confiance : 94%
- Aucune lésion suspecte détectée
- Substance blanche : aspect normal
- Pas d'effet de masse

STRUCTURES MÉDIANES :
- Ligne médiane non déviée
- Ventricules symétriques""",
        conclusion=""  # À compléter par le radiologue
    )
    
    # Envoyer le rapport
    result = client.store_report(
        technical_id=f"TEO_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
        structured=report,
        patient_id="PAT123456",
        exam_uid="1.2.840.113619.2.55.3.604688.12345",
        accession_number="ACC2024001",
        study_instance_uid="1.2.840.10008.5.1.4.1.1.2.12345",
        ai_modules=["brain_volumetry", "lesion_detection", "white_matter_analysis"],
        modality="MR",
        metadata={
            "teo_version": "2.1.0",
            "processing_time_ms": 1523,
            "confidence_score": 0.94,
            "site_id": "HOPITAL_CENTRAL"
        }
    )
    
    if result.success:
        print(f"✅ Rapport stocké avec succès!")
        print(f"   Technical ID: {result.technical_id}")
        print(f"   URL de récupération: {result.retrieval_url}")
        print(f"   Expire le: {result.expires_at}")
    else:
        print(f"❌ Erreur: {result.error}")
```

### 5.2 Intégration dans Pipeline TÉO Hub

```python
"""
Exemple d'intégration dans le pipeline de traitement TÉO Hub.
"""

from airadcr_client import AiradcrDesktopClient, StructuredReport

class TeoHubPipeline:
    """Pipeline de traitement TÉO Hub avec intégration AIRADCR."""
    
    def __init__(self):
        self.airadcr_client = AiradcrDesktopClient(
            api_key="airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
        )
    
    def process_study(self, dicom_study: dict) -> dict:
        """
        Traite une étude DICOM et envoie le résultat à AIRADCR.
        
        Args:
            dicom_study: Métadonnées DICOM de l'étude
            
        Returns:
            Résultat du traitement avec URL AIRADCR
        """
        # 1. Extraction des métadonnées DICOM
        patient_id = dicom_study.get("PatientID")
        study_uid = dicom_study.get("StudyInstanceUID")
        accession = dicom_study.get("AccessionNumber")
        modality = dicom_study.get("Modality")
        
        # 2. Traitement IA (votre logique existante)
        ai_results = self._run_ai_analysis(dicom_study)
        
        # 3. Construction du rapport structuré
        structured = StructuredReport(
            title=self._generate_title(dicom_study),
            indication=dicom_study.get("StudyDescription", ""),
            technique=self._format_technique(dicom_study),
            results=ai_results["formatted_results"],
            conclusion=""
        )
        
        # 4. Envoi vers AIRADCR Desktop (si disponible)
        airadcr_url = None
        if self.airadcr_client.is_desktop_available():
            result = self.airadcr_client.store_report(
                technical_id=f"TEO_{accession}",
                structured=structured,
                patient_id=patient_id,
                exam_uid=study_uid,
                accession_number=accession,
                study_instance_uid=study_uid,
                ai_modules=ai_results["modules_used"],
                modality=modality,
                metadata={
                    "confidence": ai_results["confidence"],
                    "processing_time": ai_results["processing_time"]
                }
            )
            
            if result.success:
                airadcr_url = f"https://airadcr.com/app?tid=TEO_{accession}"
        
        return {
            "status": "completed",
            "ai_results": ai_results,
            "airadcr_url": airadcr_url,
            "accession_number": accession
        }
    
    def _run_ai_analysis(self, study: dict) -> dict:
        """Votre logique d'analyse IA existante."""
        # ... implémentation ...
        pass
    
    def _generate_title(self, study: dict) -> str:
        """Génère le titre du rapport."""
        modality = study.get("Modality", "")
        description = study.get("StudyDescription", "")
        return f"{modality} {description}".strip()
    
    def _format_technique(self, study: dict) -> str:
        """Formate la description technique."""
        # ... implémentation ...
        pass
```

---

## 6. Exemples de Code C#

### 6.1 Client C# Complet

```csharp
/*
 * AIRADCR Desktop Client pour TÉO Hub
 * ====================================
 * Client C# pour l'intégration avec l'API AIRADCR Desktop.
 * 
 * Dépendances NuGet:
 *   - System.Net.Http.Json
 *   - System.Text.Json
 * 
 * Usage:
 *   var client = new AiradcrDesktopClient("airadcr_prod_xxx");
 *   if (await client.IsDesktopAvailableAsync())
 *   {
 *       var result = await client.StoreReportAsync(...);
 *   }
 */

using System;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using System.Collections.Generic;

namespace TeoHub.AiradcrIntegration
{
    #region Data Models

    /// <summary>
    /// Structure du rapport radiologique.
    /// </summary>
    public class StructuredReport
    {
        [JsonPropertyName("title")]
        public string Title { get; set; } = "";

        [JsonPropertyName("indication")]
        public string Indication { get; set; } = "";

        [JsonPropertyName("technique")]
        public string Technique { get; set; } = "";

        [JsonPropertyName("results")]
        public string Results { get; set; } = "";

        [JsonPropertyName("conclusion")]
        public string Conclusion { get; set; } = "";
    }

    /// <summary>
    /// Requête de stockage de rapport.
    /// </summary>
    public class PendingReportRequest
    {
        [JsonPropertyName("technical_id")]
        public string TechnicalId { get; set; } = "";

        [JsonPropertyName("patient_id")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? PatientId { get; set; }

        [JsonPropertyName("exam_uid")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? ExamUid { get; set; }

        [JsonPropertyName("accession_number")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? AccessionNumber { get; set; }

        [JsonPropertyName("study_instance_uid")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? StudyInstanceUid { get; set; }

        [JsonPropertyName("structured")]
        public StructuredReport Structured { get; set; } = new();

        [JsonPropertyName("source_type")]
        public string SourceType { get; set; } = "teo_hub";

        [JsonPropertyName("ai_modules")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string[]? AiModules { get; set; }

        [JsonPropertyName("modality")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? Modality { get; set; }

        [JsonPropertyName("metadata")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public Dictionary<string, object>? Metadata { get; set; }

        [JsonPropertyName("expires_in_hours")]
        public int ExpiresInHours { get; set; } = 24;
    }

    /// <summary>
    /// Réponse du stockage de rapport.
    /// </summary>
    public class StoreReportResponse
    {
        [JsonPropertyName("success")]
        public bool Success { get; set; }

        [JsonPropertyName("technical_id")]
        public string TechnicalId { get; set; } = "";

        [JsonPropertyName("retrieval_url")]
        public string RetrievalUrl { get; set; } = "";

        [JsonPropertyName("expires_at")]
        public string ExpiresAt { get; set; } = "";

        [JsonPropertyName("message")]
        public string Message { get; set; } = "";

        [JsonPropertyName("error")]
        public string Error { get; set; } = "";
    }

    /// <summary>
    /// Réponse du health check.
    /// </summary>
    public class HealthResponse
    {
        [JsonPropertyName("status")]
        public string Status { get; set; } = "";

        [JsonPropertyName("version")]
        public string Version { get; set; } = "";

        [JsonPropertyName("timestamp")]
        public string Timestamp { get; set; } = "";
    }

    #endregion

    /// <summary>
    /// Client C# pour l'API AIRADCR Desktop.
    /// </summary>
    public class AiradcrDesktopClient : IDisposable
    {
        private readonly HttpClient _httpClient;
        private readonly string _baseUrl;
        private readonly JsonSerializerOptions _jsonOptions;

        /// <summary>
        /// Initialise le client AIRADCR.
        /// </summary>
        /// <param name="apiKey">Clé API de production</param>
        /// <param name="baseUrl">URL du serveur local</param>
        /// <param name="timeoutSeconds">Timeout en secondes</param>
        public AiradcrDesktopClient(
            string apiKey,
            string baseUrl = "http://localhost:8741",
            int timeoutSeconds = 10)
        {
            _baseUrl = baseUrl.TrimEnd('/');
            _httpClient = new HttpClient
            {
                Timeout = TimeSpan.FromSeconds(timeoutSeconds)
            };
            _httpClient.DefaultRequestHeaders.Add("X-API-Key", apiKey);

            _jsonOptions = new JsonSerializerOptions
            {
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
                DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
            };
        }

        /// <summary>
        /// Vérifie si le desktop AIRADCR est disponible.
        /// </summary>
        public async Task<bool> IsDesktopAvailableAsync()
        {
            try
            {
                using var cts = new System.Threading.CancellationTokenSource(TimeSpan.FromSeconds(2));
                var response = await _httpClient.GetAsync($"{_baseUrl}/health", cts.Token);
                
                if (response.IsSuccessStatusCode)
                {
                    var health = await response.Content.ReadFromJsonAsync<HealthResponse>();
                    Console.WriteLine($"[AIRADCR] Desktop disponible - Version: {health?.Version}");
                    return true;
                }
                return false;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[AIRADCR] Desktop non disponible: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Stocke un rapport pré-rempli sur le desktop AIRADCR.
        /// </summary>
        public async Task<StoreReportResponse> StoreReportAsync(PendingReportRequest request)
        {
            try
            {
                Console.WriteLine($"[AIRADCR] Envoi rapport {request.TechnicalId}...");

                var json = JsonSerializer.Serialize(request, _jsonOptions);
                var content = new StringContent(json, Encoding.UTF8, "application/json");

                var response = await _httpClient.PostAsync($"{_baseUrl}/pending-report", content);
                var responseJson = await response.Content.ReadAsStringAsync();

                var result = JsonSerializer.Deserialize<StoreReportResponse>(responseJson, _jsonOptions)
                    ?? new StoreReportResponse { Success = false, Error = "Désérialisation échouée" };

                if (result.Success)
                {
                    Console.WriteLine($"[AIRADCR] Rapport stocké: {result.RetrievalUrl}");
                }
                else
                {
                    Console.WriteLine($"[AIRADCR] Erreur: {result.Error}");
                }

                return result;
            }
            catch (TaskCanceledException)
            {
                return new StoreReportResponse { Success = false, Error = "Request timeout" };
            }
            catch (Exception ex)
            {
                return new StoreReportResponse { Success = false, Error = ex.Message };
            }
        }

        /// <summary>
        /// Raccourci pour créer et envoyer un rapport.
        /// </summary>
        public async Task<StoreReportResponse> StoreReportAsync(
            string technicalId,
            StructuredReport structured,
            string? patientId = null,
            string? examUid = null,
            string? accessionNumber = null,
            string? studyInstanceUid = null,
            string[]? aiModules = null,
            string? modality = null,
            Dictionary<string, object>? metadata = null,
            int expiresInHours = 24)
        {
            var request = new PendingReportRequest
            {
                TechnicalId = technicalId,
                Structured = structured,
                PatientId = patientId,
                ExamUid = examUid,
                AccessionNumber = accessionNumber,
                StudyInstanceUid = studyInstanceUid,
                AiModules = aiModules,
                Modality = modality,
                Metadata = metadata,
                ExpiresInHours = expiresInHours
            };

            return await StoreReportAsync(request);
        }

        public void Dispose()
        {
            _httpClient?.Dispose();
        }
    }

    // =========================================================================
    // EXEMPLE D'UTILISATION
    // =========================================================================

    class Program
    {
        static async Task Main(string[] args)
        {
            using var client = new AiradcrDesktopClient(
                apiKey: "airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
            );

            // Vérifier la disponibilité
            if (!await client.IsDesktopAvailableAsync())
            {
                Console.WriteLine("❌ Desktop AIRADCR non disponible");
                return;
            }

            Console.WriteLine("✅ Desktop AIRADCR disponible");

            // Créer le rapport structuré
            var structured = new StructuredReport
            {
                Title = "IRM Cérébrale",
                Indication = "Céphalées chroniques depuis 3 mois",
                Technique = "IRM 3T avec injection de gadolinium. Séquences T1, T2, FLAIR, diffusion",
                Results = @"Analyse IA TÉO Hub (v2.1.0) :

VOLUMÉTRIE CÉRÉBRALE :
- Volume cérébral total : 1450 cm³ (percentile 55)
- Ratio ventricules/cerveau : 2.1% (normal)

ANALYSE DES LÉSIONS :
- Score de confiance : 94%
- Aucune lésion suspecte détectée",
                Conclusion = "" // À compléter par le radiologue
            };

            // Envoyer le rapport
            var result = await client.StoreReportAsync(
                technicalId: $"TEO_{DateTime.Now:yyyyMMdd_HHmmss}",
                structured: structured,
                patientId: "PAT123456",
                examUid: "1.2.840.113619.2.55.3.12345",
                accessionNumber: "ACC2024001",
                studyInstanceUid: "1.2.840.10008.5.1.4.1.1.2.12345",
                aiModules: new[] { "brain_volumetry", "lesion_detection" },
                modality: "MR",
                metadata: new Dictionary<string, object>
                {
                    { "teo_version", "2.1.0" },
                    { "confidence_score", 0.94 },
                    { "site_id", "HOPITAL_CENTRAL" }
                }
            );

            if (result.Success)
            {
                Console.WriteLine($"✅ Rapport stocké!");
                Console.WriteLine($"   Technical ID: {result.TechnicalId}");
                Console.WriteLine($"   URL: {result.RetrievalUrl}");
                Console.WriteLine($"   Expire: {result.ExpiresAt}");
            }
            else
            {
                Console.WriteLine($"❌ Erreur: {result.Error}");
            }
        }
    }
}
```

### 6.2 Intégration ASP.NET Core

```csharp
/*
 * Service d'intégration pour ASP.NET Core / .NET 6+
 */

using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace TeoHub.Services
{
    /// <summary>
    /// Service injectable pour l'intégration AIRADCR.
    /// </summary>
    public interface IAiradcrService
    {
        Task<bool> IsAvailableAsync();
        Task<string?> SendReportAsync(DicomStudy study, AiAnalysisResult aiResult);
    }

    public class AiradcrService : IAiradcrService
    {
        private readonly AiradcrDesktopClient _client;
        private readonly ILogger<AiradcrService> _logger;

        public AiradcrService(ILogger<AiradcrService> logger)
        {
            _logger = logger;
            _client = new AiradcrDesktopClient(
                apiKey: Environment.GetEnvironmentVariable("AIRADCR_API_KEY") 
                    ?? "airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
            );
        }

        public async Task<bool> IsAvailableAsync()
        {
            return await _client.IsDesktopAvailableAsync();
        }

        public async Task<string?> SendReportAsync(DicomStudy study, AiAnalysisResult aiResult)
        {
            if (!await IsAvailableAsync())
            {
                _logger.LogWarning("AIRADCR Desktop non disponible");
                return null;
            }

            var result = await _client.StoreReportAsync(
                technicalId: $"TEO_{study.AccessionNumber}",
                structured: new StructuredReport
                {
                    Title = $"{study.Modality} {study.StudyDescription}",
                    Indication = study.StudyDescription,
                    Technique = study.ProtocolName,
                    Results = aiResult.FormattedReport,
                    Conclusion = ""
                },
                patientId: study.PatientId,
                examUid: study.StudyInstanceUid,
                accessionNumber: study.AccessionNumber,
                studyInstanceUid: study.StudyInstanceUid,
                aiModules: aiResult.ModulesUsed,
                modality: study.Modality
            );

            if (result.Success)
            {
                _logger.LogInformation("Rapport envoyé: {Url}", result.RetrievalUrl);
                return $"https://airadcr.com/app?tid=TEO_{study.AccessionNumber}";
            }

            _logger.LogError("Erreur envoi rapport: {Error}", result.Error);
            return null;
        }
    }

    // Enregistrement dans Program.cs / Startup.cs
    public static class ServiceCollectionExtensions
    {
        public static IServiceCollection AddAiradcrIntegration(this IServiceCollection services)
        {
            services.AddSingleton<IAiradcrService, AiradcrService>();
            return services;
        }
    }
}
```

---

## 7. Gestion des Erreurs

### 7.1 Codes HTTP

| Code | Signification | Action Recommandée |
|------|---------------|-------------------|
| **200** | Succès (GET/DELETE) | Traiter la réponse |
| **201** | Créé (POST) | Stocker `technical_id` et `retrieval_url` |
| **400** | Requête invalide | Vérifier le payload JSON |
| **401** | Non autorisé | Vérifier la clé API |
| **404** | Non trouvé | Rapport expiré ou ID incorrect |
| **409** | Conflit | `technical_id` déjà utilisé |
| **500** | Erreur serveur | Retry après délai |

### 7.2 Messages d'Erreur

```json
// 400 - Champ manquant
{"success": false, "error": "Missing required field: technical_id"}

// 400 - Format invalide
{"success": false, "error": "Invalid technical_id format"}

// 401 - Authentification
{"success": false, "error": "Invalid or missing API key"}

// 404 - Non trouvé
{"success": false, "error": "Report not found or expired"}

// 409 - Doublon
{"success": false, "error": "Report with this technical_id already exists"}

// 500 - Erreur interne
{"success": false, "error": "Database error: ..."}
```

### 7.3 Stratégie de Retry

```python
import time
from typing import Callable

def with_retry(
    func: Callable,
    max_retries: int = 3,
    base_delay: float = 1.0
):
    """Exécute une fonction avec retry exponentiel."""
    for attempt in range(max_retries):
        try:
            return func()
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            delay = base_delay * (2 ** attempt)
            print(f"Retry {attempt + 1}/{max_retries} dans {delay}s...")
            time.sleep(delay)
```

---

## 8. Bonnes Pratiques

### 8.1 Avant Envoi

- [ ] **Toujours** vérifier `/health` avant `POST`
- [ ] Générer des `technical_id` **uniques et prévisibles** (ex: `TEO_{accession}`)
- [ ] Définir `source_type: "teo_hub"` pour traçabilité
- [ ] Inclure les `ai_modules` utilisés

### 8.2 Format des Identifiants

```
technical_id : TEO_[ACCESSION_NUMBER]
              TEO_ACC2024001
              
patient_id   : Format RIS local
              PAT123456

exam_uid     : UID DICOM standard
              1.2.840.113619.2.XXX.YYY
```

### 8.3 Gestion du Fallback

```python
def send_report_with_fallback(report_data: dict) -> str:
    """Envoie au desktop, fallback vers cloud si indisponible."""
    
    # Essayer le desktop local d'abord
    if airadcr_client.is_desktop_available():
        result = airadcr_client.store_report(**report_data)
        if result.success:
            return f"https://airadcr.com/app?tid={result.technical_id}"
    
    # Fallback: cloud (sans identifiants patients)
    cloud_data = {
        k: v for k, v in report_data.items()
        if k not in ['patient_id', 'exam_uid', 'accession_number', 'study_instance_uid']
    }
    return send_to_cloud(cloud_data)
```

---

## 9. Tests et Validation

### 9.1 Tests cURL

```bash
# 1. Vérifier le health
curl -s http://localhost:8741/health | jq

# 2. Stocker un rapport
curl -X POST http://localhost:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -d '{
    "technical_id": "TEO_TEST_001",
    "patient_id": "PAT123456",
    "exam_uid": "1.2.3.4.5",
    "accession_number": "ACC001",
    "structured": {
      "title": "Test IRM",
      "indication": "Test indication",
      "results": "Analyse IA: Test réussi"
    },
    "source_type": "teo_hub",
    "ai_modules": ["test_module"]
  }' | jq

# 3. Récupérer le rapport
curl -s "http://localhost:8741/pending-report?tid=TEO_TEST_001" | jq

# 4. Supprimer le rapport
curl -X DELETE "http://localhost:8741/pending-report?tid=TEO_TEST_001" | jq
```

### 9.2 Script PowerShell de Test

```powershell
# test-airadcr-api.ps1

$BaseUrl = "http://localhost:8741"
$ApiKey = "airadcr_prod_7f3k9m2x5p8w1q4v6n0z"
$Headers = @{
    "Content-Type" = "application/json"
    "X-API-Key" = $ApiKey
}

Write-Host "=== Test API AIRADCR Desktop ===" -ForegroundColor Cyan

# Test 1: Health Check
Write-Host "`n[1/4] Health Check..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$BaseUrl/health" -Method GET
    Write-Host "✅ Desktop disponible - Version: $($health.version)" -ForegroundColor Green
} catch {
    Write-Host "❌ Desktop non disponible" -ForegroundColor Red
    exit 1
}

# Test 2: Store Report
Write-Host "`n[2/4] Store Report..." -ForegroundColor Yellow
$testId = "TEO_TEST_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
$body = @{
    technical_id = $testId
    patient_id = "PAT_TEST_123"
    exam_uid = "1.2.3.4.5.6.7.8.9"
    accession_number = "ACC_TEST_001"
    structured = @{
        title = "IRM Test"
        indication = "Test automatisé"
        results = "Analyse IA: Test en cours"
    }
    source_type = "teo_hub"
    ai_modules = @("test_module")
} | ConvertTo-Json -Depth 3

try {
    $result = Invoke-RestMethod -Uri "$BaseUrl/pending-report" -Method POST -Headers $Headers -Body $body
    Write-Host "✅ Rapport stocké: $($result.retrieval_url)" -ForegroundColor Green
} catch {
    Write-Host "❌ Erreur: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Get Report
Write-Host "`n[3/4] Get Report..." -ForegroundColor Yellow
try {
    $report = Invoke-RestMethod -Uri "$BaseUrl/pending-report?tid=$testId" -Method GET
    Write-Host "✅ Rapport récupéré - Patient: $($report.data.patient_id)" -ForegroundColor Green
} catch {
    Write-Host "❌ Erreur: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Delete Report
Write-Host "`n[4/4] Delete Report..." -ForegroundColor Yellow
try {
    $delete = Invoke-RestMethod -Uri "$BaseUrl/pending-report?tid=$testId" -Method DELETE
    Write-Host "✅ Rapport supprimé" -ForegroundColor Green
} catch {
    Write-Host "❌ Erreur: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n=== Tests terminés ===" -ForegroundColor Cyan
```

### 9.3 Checklist Pré-Production

- [ ] Clé API de production configurée
- [ ] Desktop AIRADCR installé et en cours d'exécution
- [ ] Port 8741 accessible depuis TÉO Hub
- [ ] Tests cURL validés
- [ ] Gestion des erreurs implémentée
- [ ] Logs de debug activés pour la phase de test
- [ ] Stratégie de fallback définie

---

## 10. Annexes

### 10.1 Schéma JSON Complet

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "PendingReportRequest",
  "type": "object",
  "required": ["technical_id", "structured"],
  "properties": {
    "technical_id": {
      "type": "string",
      "maxLength": 100,
      "pattern": "^[A-Za-z0-9_-]+$",
      "description": "Identifiant unique du rapport"
    },
    "patient_id": {
      "type": "string",
      "maxLength": 50,
      "description": "ID patient local (accepté en mode local)"
    },
    "exam_uid": {
      "type": "string",
      "maxLength": 100,
      "description": "UID DICOM de l'examen"
    },
    "accession_number": {
      "type": "string",
      "maxLength": 50,
      "description": "Numéro d'accession DICOM"
    },
    "study_instance_uid": {
      "type": "string",
      "maxLength": 100,
      "description": "Study Instance UID DICOM"
    },
    "structured": {
      "type": "object",
      "required": ["title"],
      "properties": {
        "title": { "type": "string" },
        "indication": { "type": "string" },
        "technique": { "type": "string" },
        "results": { "type": "string" },
        "conclusion": { "type": "string" }
      }
    },
    "source_type": {
      "type": "string",
      "default": "teo_hub",
      "enum": ["teo_hub", "ris_local", "pacs_local", "external"]
    },
    "ai_modules": {
      "type": "array",
      "items": { "type": "string" }
    },
    "modality": {
      "type": "string",
      "enum": ["MR", "CT", "US", "XA", "CR", "DX", "MG", "NM", "PT", "RF", "OT"]
    },
    "metadata": {
      "type": "object",
      "additionalProperties": true
    },
    "expires_in_hours": {
      "type": "integer",
      "minimum": 1,
      "maximum": 168,
      "default": 24
    }
  }
}
```

### 10.2 Modalités DICOM Supportées

| Code | Description |
|------|-------------|
| MR | Imagerie par Résonance Magnétique |
| CT | Tomodensitométrie |
| US | Échographie |
| XA | Angiographie |
| CR | Radiographie Numérisée |
| DX | Radiographie Numérique |
| MG | Mammographie |
| NM | Médecine Nucléaire |
| PT | TEP (PET Scan) |
| RF | Radioscopie |
| OT | Autre |

### 10.3 FAQ Technique

**Q: Que se passe-t-il si le desktop n'est pas disponible ?**
> R: Implémentez un fallback vers le cloud Supabase, mais sans les identifiants patients.

**Q: Les rapports sont-ils chiffrés ?**
> R: Oui, la base SQLite utilise SQLCipher pour le chiffrement au repos.

**Q: Puis-je envoyer plusieurs rapports en parallèle ?**
> R: Oui, chaque requête est indépendante. Utilisez des `technical_id` uniques.

**Q: Comment gérer les doublons ?**
> R: L'API retourne 409 si un `technical_id` existe déjà. Utilisez `DELETE` puis `POST` pour remplacer.

**Q: Quelle est la taille maximale d'un rapport ?**
> R: Pas de limite stricte, mais recommandé < 1 MB pour les performances.

---

## 📞 Support

- **Email** : support@airadcr.com
- **Documentation** : https://docs.airadcr.com
- **GitHub Issues** : https://github.com/airadcr/desktop/issues

---

*Document généré le 16 décembre 2024 - Version 1.0.0*
