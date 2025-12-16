# 📋 Documentation API Locale AIRADCR Desktop

## Vue d'ensemble

Le serveur HTTP local Tauri (`localhost:8741`) permet aux RIS/PACS d'envoyer des rapports pré-structurés **avec identifiants patients** car les données ne quittent jamais la machine.

```
┌──────────────┐     POST /pending-report     ┌──────────────────┐
│   RIS/PACS   │ ─────────────────────────────▶│  Tauri Desktop   │
│   (Local)    │  patient_id, exam_uid, ...   │  localhost:8741  │
└──────────────┘                               └────────┬─────────┘
                                                        │ SQLite
       ┌────────────────────────────────────────────────▼─────────┐
       │                                                          │
       │  GET /pending-report?tid=XXX                             │
       │                                                          │
       ▼                                                          │
┌──────────────┐                               ┌──────────────────┘
│ airadcr.com  │ ◀────────────────────────────▶│
│   (iframe)   │   postMessage → Injection     │
└──────────────┘                               │
```

---

## 🔑 Différence Cloud vs Local

| Champ | Cloud (Supabase) | Local (Tauri) |
|-------|------------------|---------------|
| `patient_id` | ❌ Interdit | ✅ **Accepté** |
| `exam_uid` | ❌ Interdit | ✅ **Accepté** |
| `accession_number` | ❌ Interdit | ✅ **Accepté** |
| `study_instance_uid` | ❌ Interdit | ✅ **Accepté** |
| **Sécurité** | Internet (HTTPS) | localhost uniquement |
| **Stockage** | AWS Cloud | SQLite local |

---

## 📡 Endpoints API

### 1. Vérification disponibilité

```http
GET http://localhost:8741/health

Response 200:
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2024-12-16T10:00:00Z"
}
```

### 2. Stocker un rapport (RIS → Desktop)

```http
POST http://localhost:8741/pending-report
Content-Type: application/json
X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z

{
  "technical_id": "EXAM_2024_001",
  
  // ✅ Identifiants patients ACCEPTÉS en local
  "patient_id": "PAT123456",
  "exam_uid": "1.2.3.4.5.6.7.8.9",
  "accession_number": "ACC2024001",
  "study_instance_uid": "1.2.840.10008.xxx",
  
  // Données structurées du rapport
  "structured": {
    "title": "IRM Cérébrale",
    "indication": "Céphalées persistantes depuis 3 mois",
    "technique": "IRM 3T avec injection gadolinium",
    "results": "",
    "conclusion": ""
  },
  
  // Métadonnées optionnelles
  "source_type": "ris_local",
  "ai_modules": ["nodule_detection", "volumetry"],
  "modality": "MR",
  "metadata": {
    "ris_name": "RIS Hospital",
    "priority": "routine",
    "referring_physician": "Dr. Martin"
  },
  "expires_in_hours": 24
}
```

**Réponse succès (201):**
```json
{
  "success": true,
  "technical_id": "EXAM_2024_001",
  "retrieval_url": "https://airadcr.com/app?tid=EXAM_2024_001",
  "expires_at": "2024-12-17T10:00:00Z"
}
```

### 3. Récupérer un rapport (airadcr.com → Desktop)

```http
GET http://localhost:8741/pending-report?tid=EXAM_2024_001

Response 200:
{
  "success": true,
  "data": {
    "technical_id": "EXAM_2024_001",
    "patient_id": "PAT123456",
    "exam_uid": "1.2.3.4.5.6.7.8.9",
    "accession_number": "ACC2024001",
    "study_instance_uid": "1.2.840.10008.xxx",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées persistantes depuis 3 mois",
      "technique": "IRM 3T avec injection gadolinium",
      "results": "",
      "conclusion": ""
    },
    "source_type": "ris_local",
    "ai_modules": ["nodule_detection", "volumetry"],
    "modality": "MR",
    "metadata": {
      "ris_name": "RIS Hospital",
      "priority": "routine"
    },
    "status": "retrieved",
    "created_at": "2024-12-16T10:00:00Z"
  }
}
```

### 4. Supprimer un rapport

```http
DELETE http://localhost:8741/pending-report?tid=EXAM_2024_001

Response 200:
{
  "success": true,
  "deleted": true
}
```

---

## 💻 Intégration TypeScript (airadcr.com)

### Hook React pour récupérer les rapports

```typescript
// hooks/useLocalDesktopReport.ts
import { useState, useEffect } from 'react';

interface LocalReport {
  technical_id: string;
  patient_id?: string;
  exam_uid?: string;
  accession_number?: string;
  study_instance_uid?: string;
  structured: {
    title?: string;
    indication?: string;
    technique?: string;
    results?: string;
    conclusion?: string;
  };
  source_type: string;
  ai_modules?: string[];
  modality?: string;
  metadata?: Record<string, unknown>;
  status: string;
  created_at: string;
}

const TAURI_LOCAL_URL = 'http://localhost:8741';

export function useLocalDesktopReport(technicalId: string | null) {
  const [report, setReport] = useState<LocalReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isDesktopAvailable, setIsDesktopAvailable] = useState(false);

  // Vérifier disponibilité du desktop
  useEffect(() => {
    async function checkDesktop() {
      try {
        const response = await fetch(`${TAURI_LOCAL_URL}/health`, {
          method: 'GET',
          signal: AbortSignal.timeout(2000),
        });
        setIsDesktopAvailable(response.ok);
      } catch {
        setIsDesktopAvailable(false);
      }
    }
    checkDesktop();
  }, []);

  // Récupérer le rapport si desktop disponible et tid présent
  useEffect(() => {
    if (!technicalId || !isDesktopAvailable) return;

    async function fetchReport() {
      setLoading(true);
      setError(null);

      try {
        const response = await fetch(
          `${TAURI_LOCAL_URL}/pending-report?tid=${encodeURIComponent(technicalId)}`
        );

        if (!response.ok) {
          if (response.status === 404) {
            setError('Rapport non trouvé ou expiré');
          } else {
            setError(`Erreur ${response.status}`);
          }
          return;
        }

        const data = await response.json();
        if (data.success && data.data) {
          setReport(data.data);
        }
      } catch (err) {
        setError('Impossible de contacter le desktop AIRADCR');
      } finally {
        setLoading(false);
      }
    }

    fetchReport();
  }, [technicalId, isDesktopAvailable]);

  return { report, loading, error, isDesktopAvailable };
}
```

### Utilisation dans un composant

```tsx
// components/DictationInterface.tsx
import { useLocalDesktopReport } from '@/hooks/useLocalDesktopReport';
import { useSearchParams } from 'react-router-dom';

export function DictationInterface() {
  const [searchParams] = useSearchParams();
  const tid = searchParams.get('tid');
  
  const { report, loading, error, isDesktopAvailable } = useLocalDesktopReport(tid);

  // Pré-remplir le formulaire avec les données du rapport
  useEffect(() => {
    if (report?.structured) {
      setTitle(report.structured.title || '');
      setIndication(report.structured.indication || '');
      setTechnique(report.structured.technique || '');
      // ... autres champs
    }
  }, [report]);

  return (
    <div>
      {/* Indicateur de source */}
      {isDesktopAvailable && (
        <Badge variant="outline" className="text-green-600">
          🖥️ Desktop connecté
        </Badge>
      )}
      
      {/* Identifiants patients (LOCAL uniquement) */}
      {report?.patient_id && (
        <div className="bg-blue-50 p-2 rounded">
          <span className="font-medium">Patient:</span> {report.patient_id}
          {report.accession_number && (
            <span className="ml-4">Accession: {report.accession_number}</span>
          )}
        </div>
      )}
      
      {/* Formulaire de dictée */}
      {/* ... */}
    </div>
  );
}
```

---

## 🔐 Authentification API Keys

### Clé de production

```
X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z
```

### Créer une nouvelle clé (Admin)

```http
POST http://localhost:8741/api-keys
X-Admin-Key: [votre-clé-admin]
Content-Type: application/json

{
  "name": "RIS Hospital XYZ"
}
```

### Lister les clés

```http
GET http://localhost:8741/api-keys
X-Admin-Key: [votre-clé-admin]
```

### Révoquer une clé

```http
DELETE http://localhost:8741/api-keys/{prefix}
X-Admin-Key: [votre-clé-admin]
```

---

## 🔒 Sécurité

### Configuration CORS

Les origines autorisées sont :
- `http://localhost:*` (tous ports)
- `https://airadcr.com`
- `https://www.airadcr.com`

### Rate Limiting

- **60 requêtes/minute** par IP
- Burst autorisé de 60 requêtes

### Expiration automatique

- Rapports expirés après **24 heures** (configurable)
- Nettoyage automatique toutes les heures

---

## 🧪 Tests avec cURL

```bash
# 1. Vérifier le desktop
curl http://localhost:8741/health

# 2. Stocker un rapport avec identifiants patients
curl -X POST http://localhost:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -d '{
    "technical_id": "TEST_001",
    "patient_id": "PAT123456",
    "exam_uid": "1.2.3.4.5",
    "accession_number": "ACC001",
    "structured": {
      "title": "Radio Thorax",
      "indication": "Toux persistante"
    },
    "source_type": "ris_local",
    "modality": "CR"
  }'

# 3. Récupérer le rapport
curl "http://localhost:8741/pending-report?tid=TEST_001"

# 4. Supprimer
curl -X DELETE "http://localhost:8741/pending-report?tid=TEST_001"
```

---

## 📊 Schéma SQLite

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
    
    -- Statut
    status TEXT DEFAULT 'pending',
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    retrieved_at TEXT
);

-- Index pour recherche rapide
CREATE INDEX idx_pending_patient_id ON pending_reports(patient_id);
CREATE INDEX idx_pending_technical_id ON pending_reports(technical_id);
```

---

## ❓ FAQ

### Q: Les identifiants patients sont-ils sécurisés ?

**Oui**, en local les données ne quittent jamais la machine :
- Stockage SQLite local uniquement
- Aucune transmission réseau externe
- Le serveur écoute uniquement sur `127.0.0.1`

### Q: Que se passe-t-il si le desktop n'est pas lancé ?

Le hook `useLocalDesktopReport` détecte automatiquement l'indisponibilité et peut basculer vers le fallback Supabase (sans identifiants patients).

### Q: Comment migrer depuis la version cloud ?

Aucune migration nécessaire - les deux systèmes coexistent. Le frontend détecte automatiquement le desktop et l'utilise en priorité.
