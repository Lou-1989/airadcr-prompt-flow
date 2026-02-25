

# Plan : Connecter POST /open-report au fetch automatique TÉO Hub + Guide complet

## Probleme actuel

Aujourd'hui, `POST /open-report` cherche le rapport **uniquement** dans la base SQLite locale. Si le rapport n'y est pas, il retourne une erreur 400/404. Le code TÉO Hub client (`teo_fetch_report`) existe mais n'est **jamais appele** par le serveur HTTP. Cela oblige le developpeur externe a faire 2 appels manuels (push le rapport + ouvrir).

## Solution : Fallback automatique TÉO Hub

Quand `POST /open-report` ne trouve pas le rapport en local, il doit automatiquement :
1. Appeler `teo_client::fetch_ai_report(patient_id, study_uid)` sur TÉO Hub
2. Convertir la reponse TÉO en format `pending_report` local
3. Stocker en SQLite
4. Emettre l'evenement Tauri de navigation
5. Afficher la fenetre

Le developpeur externe n'aura plus qu'a faire **un seul appel** : `POST /open-report?patient_id=XXX&exam_uid=YYY`

## Modifications techniques

### 1. Fichier `src-tauri/src/http_server/handlers.rs`

Dans la fonction `open_report` (ligne 795-828), modifier le bloc ou `technical_id` est `None` apres la recherche SQLite :

```text
Avant (ligne 815):
  Ok(None) => None,

Apres:
  Ok(None) => {
      // Fallback: tenter de recuperer depuis TEO Hub
      if teo_hub_enabled && has_patient && has_exam {
          match teo_client::fetch_ai_report(
              query.patient_id.as_deref().unwrap(),
              query.exam_uid.as_deref().unwrap()
          ).await {
              Ok(teo_report) => {
                  // Generer un technical_id
                  let tid = format!("teo_auto_{}", Uuid::new_v4().to_string()[..8]);
                  // Convertir structured_report TEO -> JSON
                  let structured = serde_json::json!({
                      "title": teo_report.result.structured_report.title,
                      "results": teo_report.result.structured_report.results,
                      "conclusion": teo_report.result.structured_report.conclusion,
                      "translated_text": teo_report.result.translation.translated_text
                  });
                  // Stocker en SQLite local
                  state.db.insert_pending_report(...);
                  Some(tid)
              }
              Err(e) => {
                  log::warn!("TEO Hub fallback failed: {}", e);
                  None
              }
          }
      } else { None }
  }
```

**Point important** : `open_report` est actuellement une fonction synchrone Actix-Web. Le fetch TEO Hub est async. Il faut s'assurer que le handler est bien `async` (il l'est deja : `pub async fn open_report`), et ajouter l'import `use crate::teo_client` + `use crate::config::get_config`.

### 2. Fichier `src-tauri/src/http_server/mod.rs`

Aucune modification necessaire -- le handler est deja async et le runtime Actix supporte les appels async dans les handlers.

### 3. Nouvel endpoint GET /teo-hub/fetch (optionnel mais recommande)

Ajouter un endpoint dedie pour permettre au RIS de declencher manuellement un fetch TEO Hub sans passer par `open-report` :

```
GET /teo-hub/fetch?patient_id=XXX&study_uid=YYY
Header: X-API-Key: VOTRE_CLE
```

Ce endpoint :
- Appelle TEO Hub
- Stocke le resultat en local
- Retourne le `technical_id` genere + `retrieval_url`
- Ne declenche PAS la navigation iframe (contrairement a `open-report`)

Ajout dans `routes.rs` :
```rust
.route("/teo-hub/fetch", web::get().to(handlers::fetch_from_teo_hub))
```

### 4. Fichier `GUIDE_DEVELOPPEUR_INTEGRATION.md`

Creer le guide complet a la racine du projet avec :
- Les 2 modes documentes (Push manuel ET Pull automatique TEO Hub)
- Commandes curl pour chaque endpoint
- Tableau recapitulatif des endpoints avec auth/codes HTTP
- Instructions cle API (3 methodes)
- Schema du pipeline complet
- Section troubleshooting

## Sequencement

1. Modifier `handlers.rs` : ajouter le fallback TEO Hub dans `open_report`
2. Ajouter le handler `fetch_from_teo_hub` dans `handlers.rs`
3. Ajouter la route dans `routes.rs`
4. Generer `GUIDE_DEVELOPPEUR_INTEGRATION.md` complet et verifie

## Impact

- **Pas de breaking change** : le mode Push (2 appels) continue de fonctionner
- **Nouveau mode Pull** : 1 seul appel suffit si TEO Hub est configure et accessible
- **Necessite un rebuild** : oui, cette modification est dans le code Rust backend
- **Configuration requise** : `teo_hub.enabled = true` dans `config.toml` + `API_TOKEN` configure

