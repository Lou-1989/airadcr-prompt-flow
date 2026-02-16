

# Mise en conformite du client TEO Hub avec l'API reelle

## Probleme

L'implementation actuelle du client Rust a ete construite sur des hypotheses qui ne correspondent pas a l'API reelle documentee dans le script Python de test. **5 incompatibilites majeures** empechent tout appel fonctionnel.

## Incompatibilites detectees

### 1. Authentification : mauvais header

L'API reelle utilise le header `API_TOKEN`, pas `X-API-Key` ni `Authorization: Bearer`.

```text
ACTUEL (ne fonctionne pas) :  X-API-Key: <token>
REEL (TÉO Hub)             :  API_TOKEN: <token>
```

Le token fourni : `Dz1RyxZu8noENuX9Vno9URcBlsP0UXA1UgUDX0Fd7gJQL2tY4zvlIRDsxIISkrk7sJ8PR2vfC6mGOvQK`

### 2. GET /th_get_ai_report : mauvais parametres

L'API exige `patient_id` ET `study_uid` (les deux obligatoires). Le code actuel envoie uniquement `accession_number`.

```text
ACTUEL :  GET /th_get_ai_report?accession_number=ACC001
REEL   :  GET /th_get_ai_report?patient_id=11601&study_uid=5.1.600...
```

### 3. Reponse GET : structure differente

La reponse reelle est enveloppee dans un objet `result` contenant `translation` et `structured_report`, pas la structure plate `TeoAiReport` actuelle.

Reponse reelle :
```text
{
  "result": {
    "translation": {
      "language": "fr",
      "translated_text": "Mammographie\n\nIndication : ..."
    },
    "structured_report": {
      "title": "Mammographie",
      "results": "...",
      "conclusion": "Sein droit: BI-RADS 1..."
    }
  }
}
```

### 4. POST /th_post_approved_report : mauvais champs

Le body attendu est `{patient_id, study_uid, approved_report}`, pas la structure complexe `TeoApprovedReport` actuelle.

```text
ACTUEL :  {report_id, accession_number, approved_text, radiologist_id, ...}
REEL   :  {patient_id, study_uid, approved_report}
```

### 5. Health check : format different

```text
ACTUEL :  {status: "ok", version: "1.0.0", uptime_seconds: 3600}
REEL   :  {ok: true, service: "AiReportWebServer/1.0"}
```

## Plan de corrections

### Fichier 1 : `src-tauri/src/teo_client/models.rs`

Remplacer toutes les structures de donnees par celles correspondant a l'API reelle :

- `TeoHealthResponse` : champs `ok` (bool) et `service` (String)
- Nouvelle structure `TeoAiReportResponse` avec `result.translation` et `result.structured_report`
- `TeoApprovedReport` : simplifier en 3 champs (`patient_id`, `study_uid`, `approved_report`)
- `TeoApprovalResponse` : champ `status` (String)

### Fichier 2 : `src-tauri/src/teo_client/mod.rs`

- **`add_auth_headers()`** : remplacer `X-API-Key` par `API_TOKEN` comme nom de header
- **`fetch_ai_report()`** : changer la signature pour accepter `patient_id` + `study_uid` au lieu de `accession_number`, et construire l'URL avec ces deux parametres
- **`submit_approved_report()`** : adapter au nouveau format de body simplifie

### Fichier 3 : `src-tauri/src/config.rs`

- Renommer le champ `api_key` en `api_token` pour refleter le nom exact du header (ou ajouter un champ `api_token` dedie)
- Mettre a jour la valeur par defaut avec le token fourni par TÉO Hub

### Fichier 4 : `src-tauri/src/main.rs`

- Mettre a jour les commandes Tauri `teo_fetch_report` pour accepter `patient_id` + `study_uid`
- Mettre a jour `teo_submit_approved` pour le nouveau format

### Fichier 5 : `src/components/TeoHubConfig.tsx` (frontend)

- Adapter l'interface de configuration pour afficher `API_TOKEN` au lieu de `API Key`

### Fichier 6 : Documentation

- Mettre a jour `TEO_HUB_CLIENT_INTEGRATION.md` avec les vrais formats d'API

## Configuration requise pour tester

Avant de pouvoir faire un appel test, il faudra configurer dans `config.toml` :

```text
[teo_hub]
enabled = true
host = "192.168.1.253"
port = 54489
tls_enabled = false
api_token = "Dz1RyxZu8noENuX9Vno9URcBlsP0UXA1UgUDX0Fd7gJQL2tY4zvlIRDsxIISkrk7sJ8PR2vfC6mGOvQK"
```

Donnees de test disponibles (3 paires patient_id/study_uid pre-calculees dans la base TÉO Hub) :

```text
patient_id = "11601"  /  study_uid = "5.1.600.598386.1.618.1.905951.1.945557.148"
patient_id = "11600"  /  study_uid = "5.1.600.598386.1.618.1.905951.1.33160890.148"
patient_id = "006"    /  study_uid = "2.16.840.1.113669.632.20.798523242.537038478.10001221047"
```

## Fichiers a modifier

| Fichier | Description |
|---|---|
| `src-tauri/src/teo_client/models.rs` | Nouvelles structures conformes a l'API reelle |
| `src-tauri/src/teo_client/mod.rs` | Header API_TOKEN, parametres patient_id+study_uid |
| `src-tauri/src/config.rs` | Champ api_token, host par defaut 192.168.1.253 |
| `src-tauri/src/main.rs` | Commandes Tauri mises a jour |
| `src/components/TeoHubConfig.tsx` | UI adaptee |
| `TEO_HUB_CLIENT_INTEGRATION.md` | Documentation corrigee |

