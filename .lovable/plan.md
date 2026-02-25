

# Desactiver l'authentification API pour tests/demo

## Contexte

Vous etes bloque par une erreur 401 persistante sur l'API locale (port 8741). Plutot que de continuer a deboguer le format des cles API, on va ajouter un **mode demo/test** qui desactive completement la verification des cles API.

## Ce qui sera modifie

### 1. Nouveau parametre dans la configuration (`src-tauri/src/config.rs`)

Ajout d'un champ `disable_api_auth` (defaut: `false`) dans `AppConfig`. Quand il est a `true`, tous les endpoints acceptent les requetes sans header `X-API-Key`.

```toml
# Dans config.toml
disable_api_auth = true
```

### 2. Fonction utilitaire d'authentification (`src-tauri/src/http_server/handlers.rs`)

Creer une fonction `is_auth_disabled()` qui verifie le flag de config. Modifier les 4 blocs d'authentification (lignes ~309, ~478, ~805, ~1054) pour court-circuiter la validation si le mode demo est actif :

```rust
// Avant
if !validate_api_key(&state.db, api_key) {
    return HttpResponse::Unauthorized()...
}

// Apres
if !is_auth_disabled() && !validate_api_key(&state.db, api_key) {
    return HttpResponse::Unauthorized()...
}
```

Un warning sera affiche dans les logs a chaque demarrage :

```
[SECURITY] API authentication DISABLED - demo/test mode only!
```

### 3. Indicateur visuel dans le Debug Panel (`src/components/DebugPanel.tsx`)

Afficher un badge rouge "AUTH DISABLED" quand le mode est actif, pour rappeler que la securite est desactivee.

## Comment l'utiliser

1. Ouvrir le fichier `config.toml` (emplacement : `%APPDATA%/airadcr-desktop/config.toml` sous Windows)
2. Ajouter la ligne `disable_api_auth = true`
3. Redemarrer l'application
4. Toutes les requetes API passent sans cle

## Ce que vous dites a votre dev

> "J'ai active le mode test sur l'exe. Tu peux appeler `POST http://127.0.0.1:8741/pending-report` et `POST http://127.0.0.1:8741/open-report?tid=XXX` sans header X-API-Key. Ca marchera directement."

## Securite

- Le flag est `false` par defaut -- aucun impact en production
- Un warning est logue a chaque requete non authentifiee
- Le Debug Panel affiche clairement que l'auth est desactivee
- **A reactiver avant mise en production**

## Fichiers modifies

| Fichier | Modification |
|---------|-------------|
| `src-tauri/src/config.rs` | Ajout champ `disable_api_auth: bool` |
| `src-tauri/src/http_server/handlers.rs` | 4 blocs auth modifies + fonction `is_auth_disabled()` |
| `src/components/DebugPanel.tsx` | Badge "AUTH DISABLED" si mode actif |

