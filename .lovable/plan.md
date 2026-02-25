

# Plan : Mise à jour documentation + correctif `study_uid` dans `open-report`

## Probleme identifie dans le code

Le developpeur demande : `POST /open-report?patient_id=...&study_uid=...`

Or dans le code actuel (`handlers.rs` ligne 147-152), la struct `OpenReportQuery` accepte uniquement `exam_uid`, **PAS** `study_uid` :

```rust
pub struct OpenReportQuery {
    pub tid: Option<String>,
    pub accession_number: Option<String>,
    pub patient_id: Option<String>,
    pub exam_uid: Option<String>,       // <-- seul champ accepte
    // study_uid manquant !
}
```

Alors que `TeoHubFetchQuery` (ligne 170-174) accepte les DEUX :

```rust
pub struct TeoHubFetchQuery {
    pub patient_id: Option<String>,
    pub study_uid: Option<String>,      // accepte
    pub exam_uid: Option<String>,       // accepte aussi
}
```

**Consequence** : si le developpeur appelle `POST /open-report?patient_id=PAT001&study_uid=1.2.3...`, le `study_uid` sera **silencieusement ignore** par Actix-Web (champ inconnu dans le query string). Le fallback TEO Hub ne sera jamais declenche car `has_exam` sera `false`.

## Modifications necessaires

### 1. `src-tauri/src/http_server/handlers.rs` — Ajouter `study_uid` a `OpenReportQuery`

Modifier la struct (ligne 146-152) :

```rust
#[derive(Deserialize)]
pub struct OpenReportQuery {
    pub tid: Option<String>,
    pub accession_number: Option<String>,
    pub patient_id: Option<String>,
    pub exam_uid: Option<String>,
    pub study_uid: Option<String>,    // NOUVEAU
}
```

Modifier le fallback (ligne 828-844) pour resoudre `study_uid` ou `exam_uid` :

```rust
let has_exam = query.exam_uid.as_ref().map_or(false, |s| !s.is_empty())
    || query.study_uid.as_ref().map_or(false, |s| !s.is_empty());

// ...dans le fallback TEO Hub :
let exam_uid_val = query.exam_uid.as_deref()
    .or(query.study_uid.as_deref())
    .unwrap_or("");
```

### 2. `src-tauri/src/http_server/routes.rs` — Ajouter alias `/refresh_gui`

Ajouter une seule ligne :

```rust
.route("/refresh_gui", web::post().to(handlers::open_report))
```

Cela permet au developpeur d'utiliser l'URL exacte qu'il a demandee.

### 3. `GUIDE_DEVELOPPEUR_INTEGRATION.md` — Mise a jour complete

Remplacement complet du guide avec :

- **Section "Appel rapide"** en tout debut : le curl exact que le developpeur doit utiliser
- Ajout de `/refresh_gui` comme alias documente
- Correction : `study_uid` et `exam_uid` sont tous les deux acceptes partout
- Schema pipeline complet verifie ligne par ligne contre le code
- Tableau des endpoints mis a jour avec colonnes : Methode, URL, Auth, Params, Description
- Reponses JSON exactes copiees depuis les structs Rust (`OpenReportResponse`, `TeoHubFetchResponse`, etc.)
- Section troubleshooting avec les codes d'erreur reels (401, 400, 502, 503)
- Section pre-requis TÉO Hub (config.toml + keychain)

## Verification croisee code/documentation

| Element | Code (handlers.rs) | Documentation | Statut |
|---------|-------------------|---------------|--------|
| POST /open-report | ligne 796 | Section 5-6 | OK |
| Fallback TEO Hub | ligne 840-874 | Section 6 | OK |
| GET /teo-hub/fetch | ligne 1039 | Section 6 Option B | OK |
| study_uid dans open-report | MANQUANT | Mentionne exam_uid | A CORRIGER |
| /refresh_gui alias | ABSENT | Non documente | A AJOUTER |
| Struct reponse open-report | lignes 154-167 | Section 5 | OK |
| Struct reponse teo-hub/fetch | lignes 176-187 | Section 6 | OK |
| Auth X-API-Key | validate_api_key() | Section 3 | OK |

## Sequencement

1. Corriger `OpenReportQuery` : ajouter `study_uid` + logique de resolution
2. Ajouter route `/refresh_gui` dans `routes.rs`
3. Regenerer `GUIDE_DEVELOPPEUR_INTEGRATION.md` complet et verifie

## Impact

- **Breaking change** : NON (ajout de champs optionnels uniquement)
- **Rebuild necessaire** : OUI (code Rust modifie)
- **Risque** : FAIBLE (ajout de champs, pas de suppression)

