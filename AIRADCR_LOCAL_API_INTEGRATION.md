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

### 5. 🔍 Rechercher un rapport par identifiants RIS (NEW)

Le RIS peut rechercher un rapport sans connaître le `technical_id`, en utilisant ses propres identifiants.

```http
GET http://localhost:8741/find-report?accession_number=ACC2024001

# Ou combinaison d'identifiants
GET http://localhost:8741/find-report?patient_id=PAT123&accession_number=ACC2024001
GET http://localhost:8741/find-report?exam_uid=1.2.3.4.5.6.7.8.9
```

**Paramètres de recherche** (au moins un requis) :
- `accession_number` - Numéro d'accession DICOM
- `patient_id` - ID patient RIS
- `exam_uid` - UID DICOM de l'examen

**Réponse succès (200):**
```json
{
  "success": true,
  "found": true,
  "data": {
    "technical_id": "EXAM_2024_001",
    "patient_id": "PAT123456",
    "accession_number": "ACC2024001",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées",
      "results": "Analyse IA: Normal..."
    },
    "status": "pending",
    "created_at": "2024-12-16T10:00:00Z"
  }
}
```

**Réponse non trouvé (404):**
```json
{
  "success": true,
  "found": false,
  "message": "No report found with these identifiers"
}
```

### 6. 🚀 Ouvrir un rapport dans AIRADCR (RIS → Navigation) (NEW)

Le RIS peut déclencher l'ouverture d'un rapport directement dans l'interface AIRADCR.

```http
# Par technical_id direct
POST http://localhost:8741/open-report?tid=EXAM_2024_001

# Ou par identifiants RIS (recherche automatique)
POST http://localhost:8741/open-report?accession_number=ACC2024001
POST http://localhost:8741/open-report?patient_id=PAT123&accession_number=ACC2024001
```

**Comportement** :
1. Si `tid` fourni : utilise directement ce technical_id
2. Sinon : recherche par identifiants RIS (accession_number, patient_id, exam_uid)
3. Émet un événement Tauri vers le frontend
4. L'iframe navigue vers `https://airadcr.com/app?tid=XXX`
5. La fenêtre AIRADCR s'affiche et prend le focus

**Réponse succès (200):**
```json
{
  "success": true,
  "navigated": true,
  "technical_id": "EXAM_2024_001",
  "message": "Navigation triggered successfully"
}
```

**Réponse erreur (400):**
```json
{
  "success": false,
  "navigated": false,
  "message": "No identifier provided. Use 'tid' or RIS identifiers"
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

/**
 * Hook pour rechercher un rapport par identifiants RIS
 */
export function useFindReport() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const findReport = async (params: {
    accession_number?: string;
    patient_id?: string;
    exam_uid?: string;
  }): Promise<LocalReport | null> => {
    const queryParams = new URLSearchParams();
    if (params.accession_number) queryParams.set('accession_number', params.accession_number);
    if (params.patient_id) queryParams.set('patient_id', params.patient_id);
    if (params.exam_uid) queryParams.set('exam_uid', params.exam_uid);

    if (!queryParams.toString()) {
      setError('Au moins un identifiant requis');
      return null;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `${TAURI_LOCAL_URL}/find-report?${queryParams}`,
        { signal: AbortSignal.timeout(5000) }
      );

      const data = await response.json();

      if (response.ok && data.success && data.data) {
        return data.data;
      } else if (response.status === 404) {
        setError('Aucun rapport trouvé');
        return null;
      } else {
        setError(data.error || 'Erreur de recherche');
        return null;
      }
    } catch (err) {
      setError('Impossible de contacter le desktop');
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { findReport, loading, error };
}

/**
 * Hook pour ouvrir un rapport dans l'interface AIRADCR
 */
export function useOpenReport() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openReport = async (params: {
    tid?: string;
    accession_number?: string;
    patient_id?: string;
    exam_uid?: string;
  }): Promise<{ success: boolean; technical_id?: string; navigated_to?: string }> => {
    const queryParams = new URLSearchParams();
    if (params.tid) queryParams.set('tid', params.tid);
    if (params.accession_number) queryParams.set('accession_number', params.accession_number);
    if (params.patient_id) queryParams.set('patient_id', params.patient_id);
    if (params.exam_uid) queryParams.set('exam_uid', params.exam_uid);

    if (!queryParams.toString()) {
      setError('Au moins un identifiant requis');
      return { success: false };
    }

    setLoading(true);
    setError(null);

    try {
      const response = await fetch(
        `${TAURI_LOCAL_URL}/open-report?${queryParams}`,
        { method: 'POST', signal: AbortSignal.timeout(5000) }
      );

      const data = await response.json();

      if (response.ok && data.success) {
        return {
          success: true,
          technical_id: data.technical_id,
          navigated_to: data.navigated_to
        };
      } else {
        setError(data.error || 'Erreur de navigation');
        return { success: false };
      }
    } catch (err) {
      setError('Impossible de contacter le desktop');
      return { success: false };
    } finally {
      setLoading(false);
    }
  };

  return { openReport, loading, error };
}
```

### Utilisation dans un composant

```tsx
// components/DictationInterface.tsx
import { useLocalDesktopReport, useFindReport, useOpenReport } from '@/hooks/useLocalDesktopReport';
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

### Composant de recherche RIS

```tsx
// components/RISReportSearch.tsx
import { useFindReport, useOpenReport } from '@/hooks/useLocalDesktopReport';
import { useState } from 'react';

export function RISReportSearch() {
  const [accessionNumber, setAccessionNumber] = useState('');
  const [foundReport, setFoundReport] = useState(null);
  
  const { findReport, loading: findLoading, error: findError } = useFindReport();
  const { openReport, loading: openLoading, error: openError } = useOpenReport();

  const handleSearch = async () => {
    const report = await findReport({ accession_number: accessionNumber });
    setFoundReport(report);
  };

  const handleOpen = async () => {
    if (foundReport?.technical_id) {
      const result = await openReport({ tid: foundReport.technical_id });
      if (result.success) {
        console.log('Navigation vers:', result.navigated_to);
      }
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-2">
        <Input
          placeholder="Numéro d'accession"
          value={accessionNumber}
          onChange={(e) => setAccessionNumber(e.target.value)}
        />
        <Button onClick={handleSearch} disabled={findLoading}>
          {findLoading ? 'Recherche...' : 'Rechercher'}
        </Button>
      </div>

      {findError && <Alert variant="destructive">{findError}</Alert>}
      {openError && <Alert variant="destructive">{openError}</Alert>}

      {foundReport && (
        <Card>
          <CardHeader>
            <CardTitle>{foundReport.structured?.title}</CardTitle>
            <CardDescription>
              Patient: {foundReport.patient_id} | Accession: {foundReport.accession_number}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {foundReport.structured?.indication}
            </p>
          </CardContent>
          <CardFooter>
            <Button onClick={handleOpen} disabled={openLoading}>
              {openLoading ? 'Ouverture...' : 'Ouvrir dans AIRADCR'}
            </Button>
          </CardFooter>
        </Card>
      )}
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
CREATE INDEX idx_pending_accession ON pending_reports(accession_number);
CREATE INDEX idx_pending_exam_uid ON pending_reports(exam_uid);
```

---

## 🔄 Workflow Complet RIS ↔ TÉO Hub ↔ AIRADCR

### Architecture des acteurs

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────────────┐
│      RIS        │     │    TÉO Hub      │     │   AIRADCR Desktop       │
│  (Xplore, etc.) │     │   (AI Server)   │     │   (Tauri + localhost)   │
└────────┬────────┘     └────────┬────────┘     └────────────┬────────────┘
         │                       │                           │
         │  1. Envoie DICOM      │                           │
         │ ─────────────────────▶│                           │
         │                       │                           │
         │                       │  2. POST /pending-report  │
         │                       │ ─────────────────────────▶│
         │                       │                           │ SQLite
         │                       │  3. Retourne technical_id │
         │                       │ ◀─────────────────────────│
         │                       │                           │
         │  4. Notifie RIS       │                           │
         │ ◀─────────────────────│                           │
         │  (accession + tid)    │                           │
         │                       │                           │
         │  5. Bouton "Ouvrir"   │                           │
         │ ─────────────────────────────────────────────────▶│
         │  POST /open-report?accession_number=XXX           │
         │                       │                           │
         │                       │           6. Navigation   │
         │                       │              iframe       │
         │                       │              airadcr.com  │
         │                       │              ?tid=XXX     │
         │                       │                           │
         │  7. GET /pending-report?tid=XXX                   │
         │                       │ ◀─────────────────────────│
         │                       │                           │
         │  8. Formulaire pré-rempli avec données patient    │
         │                       │                           │
         │  9. Radiologiste dicte → Injection dans RIS       │
         │ ◀─────────────────────────────────────────────────│
```

### Étapes détaillées

#### Étape 1-3 : TÉO Hub analyse et stocke

TÉO Hub reçoit les images DICOM, effectue l'analyse IA, et envoie le rapport pré-rempli à AIRADCR Desktop :

```bash
# TÉO Hub → AIRADCR Desktop
curl -X POST http://localhost:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -d '{
    "technical_id": "TEO_ACC2024001_MR",
    "patient_id": "PAT123456",
    "accession_number": "ACC2024001",
    "exam_uid": "1.2.3.4.5.6.7.8.9",
    "structured": {
      "title": "IRM Cérébrale",
      "indication": "Céphalées chroniques",
      "technique": "IRM 3T séquences T1, T2, FLAIR, diffusion",
      "results": "VOLUMÉTRIE HIPPOCAMPIQUE:\n- Hippocampe droit: 3.2 cm³ (normal)\n- Hippocampe gauche: 3.1 cm³ (normal)\n\nANALYSE LÉSIONNELLE:\n- Aucune lésion focale détectée",
      "conclusion": ""
    },
    "source_type": "teo_hub",
    "ai_modules": ["hippocampal_volumetry", "lesion_detection"],
    "modality": "MR"
  }'
```

#### Étape 4 : TÉO Hub notifie le RIS

TÉO Hub informe le RIS que le rapport est prêt (via HL7, API, ou webhook selon intégration).

#### Étape 5-6 : RIS ouvre AIRADCR

Quand l'utilisateur clique sur "Ouvrir dans AIRADCR" dans le RIS :

```bash
# RIS → AIRADCR Desktop (recherche par accession_number)
curl -X POST "http://localhost:8741/open-report?accession_number=ACC2024001"

# Réponse
{
  "success": true,
  "navigated": true,
  "technical_id": "TEO_ACC2024001_MR",
  "message": "Navigation triggered successfully"
}
```

#### Étape 7-8 : airadcr.com récupère les données

L'iframe navigue vers `https://airadcr.com/app?tid=TEO_ACC2024001_MR` qui appelle automatiquement :

```bash
GET http://localhost:8741/pending-report?tid=TEO_ACC2024001_MR
```

Le formulaire de dictée est pré-rempli avec :
- Identifiants patient (patient_id, accession_number)
- Données structurées (titre, indication, technique, résultats IA)
- Métadonnées (modalité, modules IA utilisés)

#### Étape 9 : Injection du rapport final

Après dictée et validation, le rapport est injecté dans le RIS via le système d'injection existant (Ctrl+Shift+S ou SpeechMike).

---

## 🧪 Tests cURL complets

```bash
# 1. Vérifier le desktop
curl http://localhost:8741/health

# 2. TÉO Hub stocke un rapport
curl -X POST http://localhost:8741/pending-report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: airadcr_prod_7f3k9m2x5p8w1q4v6n0z" \
  -d '{
    "technical_id": "TEST_001",
    "patient_id": "PAT123456",
    "accession_number": "ACC001",
    "structured": {"title": "Radio Thorax", "indication": "Toux"},
    "modality": "CR"
  }'

# 3. RIS recherche par accession_number
curl "http://localhost:8741/find-report?accession_number=ACC001"

# 4. RIS ouvre le rapport dans AIRADCR
curl -X POST "http://localhost:8741/open-report?accession_number=ACC001"

# 5. Récupérer le rapport (fait automatiquement par airadcr.com)
curl "http://localhost:8741/pending-report?tid=TEST_001"

# 6. Nettoyer
curl -X DELETE "http://localhost:8741/pending-report?tid=TEST_001"
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

### Q: Le RIS doit-il connaître le technical_id de TÉO Hub ?

**Non !** Le RIS peut utiliser ses propres identifiants (accession_number, patient_id, exam_uid) pour rechercher (`/find-report`) et ouvrir (`/open-report`) un rapport. AIRADCR fait la correspondance automatiquement.

### Q: Quelle est la différence entre `/find-report` et `/open-report` ?

- **`/find-report`** : Recherche et retourne les données du rapport (lecture seule)
- **`/open-report`** : Recherche ET déclenche la navigation dans l'interface AIRADCR

### Q: Plusieurs rapports peuvent-ils exister pour le même patient ?

Oui. La recherche retourne le rapport le plus récent correspondant aux critères. Utilisez des identifiants plus spécifiques (accession_number + exam_uid) pour cibler un examen précis.
