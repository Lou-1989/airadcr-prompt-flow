

# Diagnostic et corrections du Debug Panel TEO Hub

## Bugs identifies dans le code actuel

### Bug 1 : `has_api_token` toujours `false` apres migration keychain (CRITIQUE)

**Fichier** : `src-tauri/src/main.rs` ligne 1371

Le code actuel :
```rust
has_api_token: !cfg.teo_hub.api_token.is_empty(),
```

Apres la migration keychain (config.rs lignes 225-235), le champ `api_token` est **vide** dans le config.toml car le token a ete deplace vers le keychain OS. Resultat : le Debug Panel affiche toujours "API Token: Non configure" meme si le token est bien present dans le keychain.

**Correction** : Verifier aussi le keychain OS via `crate::database::keychain::get_teo_token()`.

### Bug 2 : `get_connection_status()` retourne toujours `Unknown`

**Fichier** : `src-tauri/src/teo_client/mod.rs` lignes 273-281

La fonction ne fait que verifier si le client est enabled. Elle ne garde aucun etat du dernier health check. Elle retourne toujours `Unknown` ou `Disabled`, jamais `Connected`.

**Impact** : Le statut affiché ne reflete jamais l'etat reel de la connexion entre deux health checks manuels.

**Pas de correction necessaire** car le frontend gere deja son propre etat via `connectionStatus` dans `TeoHubConfig.tsx`. La commande `teo_get_connection_status` n'est d'ailleurs jamais appelee cote frontend.

### Bug 3 : Label "Deconnecte" trompeur

**Fichier** : `src/components/TeoHubConfig.tsx` ligne 122

Le badge affiche "Deconnecte" quand le health check TÉO Hub echoue. L'utilisateur confond avec sa session AIRADCR (il est bien connecte en tant que Dr. Lounes). Ce n'est pas une deconnexion de l'app, c'est le serveur TÉO Hub distant qui ne repond pas.

## Plan de corrections

### 1. Corriger `has_api_token` dans `teo_get_config` (Rust)

**Fichier** : `src-tauri/src/main.rs` (ligne 1371)

Remplacer :
```rust
has_api_token: !cfg.teo_hub.api_token.is_empty(),
```
Par :
```rust
has_api_token: !cfg.teo_hub.api_token.is_empty() 
    || crate::database::keychain::get_teo_token()
        .map(|opt| opt.is_some()).unwrap_or(false),
```

**Justification** : Apres migration, le token est dans le keychain, pas dans le TOML. Sans cette correction, l'interface affiche un faux negatif.

### 2. Ajouter une commande `get_runtime_info` (Rust)

**Fichier** : `src-tauri/src/main.rs`

Nouvelle commande Tauri qui expose :
- `disable_api_auth` : etat reel du bypass
- `http_port` : port configure (note : le port reel binde n'est pas stocke actuellement)
- `config_path` : chemin du fichier config.toml charge
- `teo_hub_enabled` : si le client TÉO est actif

**Justification** : Permet au Debug Panel d'afficher l'etat runtime sans deviner. Resout le probleme du 401 persistant en rendant visible si le bypass est actif ou non.

### 3. Corriger les labels dans `TeoHubConfig.tsx`

**Fichier** : `src/components/TeoHubConfig.tsx`

| Ancien label | Nouveau label | Raison |
|---|---|---|
| "Deconnecte" | "TÉO Hub non joignable" | Evite la confusion avec la session utilisateur |
| "Connecte" | "TÉO Hub OK" | Coherence |

### 4. Ajouter un bloc "Etat Runtime" dans le Debug Panel

**Fichier** : `src/components/DebugPanel.tsx`

Nouveau bloc en haut de l'onglet "Systeme" qui appelle `invoke('get_runtime_info')` et affiche :
- Badge rouge clignotant `AUTH DESACTIVEE` ou vert `AUTH ACTIVEE`
- Port HTTP configure
- Chemin config.toml (tronque)

**Justification** : Le dev peut verifier en 5 secondes si le bypass est actif, sans curl ni log.

## Fichiers modifies

| Fichier | Modification | Justification |
|---|---|---|
| `src-tauri/src/main.rs` | Fix `has_api_token` + ajout `get_runtime_info` | Bug keychain + visibilite runtime |
| `src/components/TeoHubConfig.tsx` | Labels corriges | UX : eviter confusion session vs TÉO Hub |
| `src/components/DebugPanel.tsx` | Bloc runtime info | Diagnostic auth bypass en un coup d'oeil |

## Fichiers NON modifies (justification)

| Fichier | Raison de ne pas toucher |
|---|---|
| `src-tauri/src/config.rs` | La logique de chargement est correcte, le `disable_api_auth` est bien lu |
| `src-tauri/src/http_server/handlers.rs` | Le bypass `is_auth_disabled()` fonctionne correctement |
| `src-tauri/src/http_server/mod.rs` | Le port actif n'est pas stocke mais ce n'est pas bloquant |
| `src-tauri/src/teo_client/mod.rs` | `get_connection_status()` n'est pas utilise cote frontend |
| `src-tauri/src/teo_client/models.rs` | Les structures sont correctes |

