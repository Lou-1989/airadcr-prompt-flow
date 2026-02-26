

# Ouverture systematique de la fenetre AIRADCR + Logging TÉO Hub

## Probleme actuel

Dans `open_report` (handlers.rs, lignes 906-918), quand aucun rapport n'est trouve (ni localement, ni via TÉO Hub), le handler retourne une erreur 400 **sans ouvrir la fenetre**. Le `show()` / `unminimize()` / `set_focus()` ne se produit qu'a la ligne 944 quand un `tid` valide existe.

Consequence : le radiologue clique dans le RIS, rien ne se passe visuellement. Il ne sait pas si l'application a recu la demande.

## Comportement souhaite

**Dans TOUS les cas** (200, 400, 401, 404, 500 du TÉO Hub), la fenetre AIRADCR doit :
1. S'afficher (`show`)
2. Se restaurer si minimisee (`unminimize`)
3. Prendre le focus (`set_focus`)

Le radiologue voit toujours l'application apparaitre. S'il y a un rapport, l'iframe navigue vers lui. S'il n'y en a pas, l'application s'ouvre sur la page d'accueil et le radiologue peut dicter librement.

## Logging

Le systeme de logging existe deja via `request_info.log_access()` qui ecrit dans la table SQLite `access_logs`. Cependant, le **fallback TÉO Hub** (lignes 866-886) ne log que de facon generique quand il echoue. Il faut enrichir le logging pour distinguer les differents codes de reponse TÉO Hub (400, 401, 404, 500) dans les logs d'acces.

---

## Modifications

### Fichier : `src-tauri/src/http_server/handlers.rs`

**1. Extraire l'ouverture de fenetre dans une fonction helper**

Creer une fonction `show_main_window()` qui fait `show + unminimize + set_focus` sur la fenetre "main". Cette fonction sera appelee au debut de `open_report`, avant toute logique de recherche de rapport.

```rust
fn show_main_window() {
    if let Some(app_handle) = APP_HANDLE.get() {
        if let Some(window) = app_handle.get_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
            log::debug!("📱 [HTTP] Fenêtre principale affichée et focalisée");
        }
    }
}
```

**2. Appeler `show_main_window()` au tout debut de `open_report`**

Juste apres la validation de l'API key (ligne 827), appeler `show_main_window()`. Ainsi, dans TOUS les cas (rapport trouve, pas trouve, erreur TÉO Hub), la fenetre s'ouvre.

**3. Enrichir le logging du fallback TÉO Hub**

Dans le bloc `Err(e)` du fallback TÉO (lignes 882-886), logger le type d'erreur TÉO Hub de maniere specifique via `request_info.log_access()` :

```rust
Err(e) => {
    let teo_error_detail = match &e {
        teo_client::errors::TeoClientError::NotFound(_) => "teo_hub_404_not_found",
        teo_client::errors::TeoClientError::Unauthorized(_) => "teo_hub_401_unauthorized",
        teo_client::errors::TeoClientError::HttpError(code, _) => {
            // Log le code HTTP specifique
            log::warn!("⚠️ [HTTP] TÉO Hub HTTP {}: {}", code, e);
            "teo_hub_http_error"
        },
        teo_client::errors::TeoClientError::NetworkError(_) => "teo_hub_network_error",
        _ => "teo_hub_other_error",
    };
    request_info.log_access(&state.db, 200, teo_error_detail, Some(&format!("TÉO fallback: {}", e)));
    log::warn!("⚠️ [HTTP] Fallback TÉO Hub échoué ({}): {}", teo_error_detail, e);
    None
}
```

**4. Quand aucun rapport n'est trouve, ouvrir AIRADCR sans `tid`**

Au lieu de retourner 400, emettre un evenement de navigation vers la page d'accueil (sans tid) et retourner 200. Le radiologue voit l'application s'ouvrir sur la page par defaut, prete a dicter.

```rust
// Quand technical_id est None : naviguer vers la page d'accueil
None => {
    log::info!("ℹ️ [HTTP] Aucun rapport trouvé, ouverture AIRADCR sans rapport");
    request_info.log_access(&state.db, 200, "no_report_found", 
        Some("Window opened without report - radiologist can dictate freely"));
    
    // Emettre navigation vers accueil (sans tid)
    if let Some(app_handle) = APP_HANDLE.get() {
        if let Some(window) = app_handle.get_window("main") {
            let _ = window.emit("airadcr:navigate_to_report", "");
        }
    }
    
    return HttpResponse::Ok().json(OpenReportResponse {
        success: true,
        message: Some("Window opened. No report available - radiologist can dictate freely.".to_string()),
        technical_id: None,
        navigated_to: Some("https://airadcr.com/app?tori=true".to_string()),
        error: None,
        source: Some("no_report".to_string()),
    });
}
```

---

## Pipeline final pour chaque code TÉO Hub

| TÉO Hub Response | Comportement AIRADCR |
|---|---|
| 200 OK (rapport recu) | Fenetre ouverte + navigation vers le rapport |
| 400 Bad Request | Fenetre ouverte + page d'accueil (log: teo_hub_bad_request) |
| 401 Unauthorized | Fenetre ouverte + page d'accueil (log: teo_hub_401_unauthorized) |
| 404 Not Found | Fenetre ouverte + page d'accueil (log: teo_hub_404_not_found) |
| 500 Server Error | Fenetre ouverte + page d'accueil (log: teo_hub_http_error) |
| Network Error | Fenetre ouverte + page d'accueil (log: teo_hub_network_error) |

## Fichiers modifies

| Fichier | Modification |
|---|---|
| `src-tauri/src/http_server/handlers.rs` | Fonction `show_main_window()`, appel au debut de `open_report`, logging enrichi du fallback TÉO, ouverture fenetre meme sans rapport |

## Ce qui ne change pas

- Le handler `POST /pending-report` (pas concerne)
- Le handler `GET /pending-report` (pas concerne)
- Le client TÉO Hub (`teo_client/mod.rs`) -- deja bien structure avec les bons types d'erreur
- Le systeme `access_logs` SQLite -- deja fonctionnel, on l'enrichit juste avec des resultats plus precis

