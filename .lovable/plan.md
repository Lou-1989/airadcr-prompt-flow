

# Audit de Securite Global AIRADCR Desktop -- Standards 2026

## Methodologie

Audit complet du code source Rust (backend Tauri), TypeScript (frontend), SQLite (base locale), serveur HTTP (Actix-Web), client TEO Hub, et configuration Cloud. Comparaison avec les standards de securite 2026 pour applications medicales desktop.

---

## FAILLES IDENTIFIEES

### CRITIQUE (a corriger avant mise en production)

#### 1. Injection Ctrl+V dans `perform_injection_at_position_direct` ne utilise pas `clipboard_modifier()`

**Fichier** : `src-tauri/src/main.rs`, lignes 560-562

Le code utilise `Key::Control` en dur au lieu de `clipboard_modifier()` :
```rust
enigo.key(Key::Control, Direction::Press)  // BUG: devrait etre clipboard_modifier()
enigo.key(Key::Unicode('v'), Direction::Click)
enigo.key(Key::Control, Direction::Release)
```

Les fonctions `perform_injection()` et `perform_injection_at_position()` ont ete corrigees mais `perform_injection_at_position_direct()` a ete oubliee. Sur macOS, cette fonction ne collera rien.

**Correction** : Remplacer `Key::Control` par `clipboard_modifier()` aux lignes 560-562.

---

#### 2. SQLite sans chiffrement -- donnees medicales en clair sur disque

**Fichier** : `src-tauri/src/database/mod.rs`, ligne 30

```rust
let conn = Connection::open(&db_path)?;  // Pas de chiffrement
```

La base `pending_reports.db` contient des identifiants patients (`patient_id`, `accession_number`, `study_instance_uid`) stockes **en clair** sur le disque. En 2026, les standards HIPAA/RGPD Medical exigent le chiffrement au repos pour les donnees de sante.

**Correction** : Migrer vers `rusqlite` avec la feature `sqlcipher` pour activer le chiffrement AES-256. Cela necessite :
- Ajouter `features = ["bundled-sqlcipher"]` dans Cargo.toml au lieu de `["bundled"]`
- Appeler `conn.execute_batch("PRAGMA key = '...'")` apres ouverture
- Stocker la cle de chiffrement dans le keychain OS (Windows Credential Manager / macOS Keychain) via `keyring` crate

---

#### 3. Endpoints `/health`, `/health/extended`, `/metrics` accessibles sans authentification

**Fichiers** : `src-tauri/src/http_server/routes.rs` lignes 13-19, `metrics.rs`

Ces endpoints exposent :
- Version exacte de l'application (facilite le ciblage de vulnerabilites connues)
- Nombre de rapports en attente
- Nombre de cles API actives
- Taille de la base de donnees
- Uptime du serveur
- Metriques Prometheus detaillees

Meme si le serveur ecoute sur `127.0.0.1`, tout logiciel malveillant sur la machine peut interroger ces endpoints. Un malware pourrait detecter la presence d'AIRADCR et sa configuration.

**Correction** : Ajouter une validation de cle admin ou API sur `/health/extended` et `/metrics`. Garder `/health` simple (juste `{"status":"ok"}`) sans version ni details.

---

#### 4. Cle de developpement loguee en clair dans la console

**Fichier** : `src-tauri/src/database/schema.rs`, ligne 184

```rust
println!("🔑 [Database] Clé de développement générée: {}", generated_key);
```

La cle API complete est imprimee dans stdout. Si les logs sont captures (par un outil de monitoring, un crash reporter, ou un fichier de log), la cle est compromise.

**Correction** : Ne jamais loguer la cle complete. Loguer uniquement le prefixe :
```rust
println!("🔑 [Database] Clé de développement générée: {}...", &generated_key[..16]);
```

---

### HAUTE (a corriger rapidement)

#### 5. Absence de validation `Content-Length` sur les requetes POST

**Fichier** : `src-tauri/src/http_server/mod.rs`

Aucune limite de taille de payload n'est configuree sur le serveur Actix-Web. Un attaquant local pourrait envoyer un JSON de plusieurs Go pour provoquer un denial-of-service (memoire saturee).

**Correction** : Ajouter une limite de taille de payload dans la configuration Actix :
```rust
App::new()
    .app_data(web::JsonConfig::default().limit(1_048_576)) // 1 MB max
```

---

#### 6. `window.eval()` avec contenu non sanitize dans `simulate_key_in_iframe`

**Fichier** : `src-tauri/src/main.rs`, lignes 883-908

```rust
let js_code = format!(r#"... '{}' ..."#, key, key, key_code, key_code, key);
window.eval(&js_code).map_err(|e| e.to_string())?;
```

Le parametre `key` est insere directement dans du code JavaScript sans echappement. Si un appel malveillant passe `key = "'; alert('xss'); '"`, du code arbitraire s'execute dans la WebView.

**Correction** : Valider `key` contre une liste blanche avant insertion :
```rust
let valid_keys = ["F10", "F11", "F12"];
if !valid_keys.contains(&key.as_str()) {
    return Err(format!("Touche non autorisée: {}", key));
}
```
(Note : la fonction `get_key_code` rejette deja les cles inconnues en retournant 0, mais la validation intervient APRES l'insertion dans le JS. Il faut valider AVANT.)

---

#### 7. Chemin de log Windows-only en dur avec backslashes

**Fichier** : `src-tauri/src/main.rs`, lignes 918-963

```rust
let log_dir = format!("{}\\AIRADCR\\logs", app_data);
let log_path = format!("{}\\app.log", log_dir);
```

Utilise `\\` (backslash Windows) en dur. Sur macOS/Linux, `APPDATA` n'existe pas et le fallback `"."` cree un dossier `.\AIRADCR\logs` dans le repertoire courant. Non fonctionnel hors Windows.

**Correction** : Utiliser `dirs::data_local_dir()` et `PathBuf::join()` :
```rust
let log_dir = dirs::data_local_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("AIRADCR")
    .join("logs");
```

---

#### 8. Rate limiting non protege contre les brute-force API keys

**Fichier** : `src-tauri/src/http_server/mod.rs`, lignes 35-38

```rust
.per_second(1)
.burst_size(60)
```

60 requetes/minute par IP est trop permissif pour proteger contre le brute-force de cles API. Un attaquant local pourrait tester 60 cles/minute. En 24h : 86 400 tentatives.

**Correction** : Ajouter un rate limiting specifique pour les echecs 401 :
- Apres 5 echecs consecutifs depuis la meme IP, bloquer pendant 5 minutes
- Loguer les tentatives excessives comme evenement de securite

---

### MOYENNE (a planifier)

#### 9. `DELETE /pending-report` sans authentification

**Fichier** : `src-tauri/src/http_server/handlers.rs`, lignes 446-482

Le endpoint `DELETE /pending-report?tid=XXX` ne verifie aucune cle API. Tout processus local peut supprimer des rapports medicaux.

**Correction** : Exiger `X-API-Key` pour les operations DELETE :
```rust
let api_key = req.headers().get("x-api-key")...;
if !validate_api_key(&state.db, api_key) { return 401; }
```

---

#### 10. Absence de `Strict-Transport-Security` header

**Fichier** : `index.html`

Le CSP ne contient pas de directive HSTS. Dans Tauri, c'est moins critique car l'app ne passe pas par un navigateur classique, mais pour les requetes vers airadcr.com, le header HSTS devrait etre present.

**Correction** : Ajouter dans index.html :
```html
<meta http-equiv="Strict-Transport-Security" content="max-age=31536000; includeSubDomains">
```

---

#### 11. `println!` utilise au lieu de `log::` dans les handlers HTTP

**Fichiers** : `handlers.rs` (lignes 292, 302, 358, 426, 435, 466, 500, 543, 579, 649, 691, 710, 734), `schema.rs`, `mod.rs`

L'utilisation de `println!` bypasse le systeme de logging structure, les niveaux de log, et les filtres de production. Des informations potentiellement sensibles apparaissent dans stdout sans controle.

**Correction** : Remplacer tous les `println!` par `log::info!`, `eprintln!` par `log::error!`, et `debug!` est deja correct.

---

#### 12. `POST /open-report` sans authentification

**Fichier** : `src-tauri/src/http_server/handlers.rs`, lignes 756-870

Ce endpoint permet a tout processus local de :
1. Naviguer l'iframe vers un rapport arbitraire
2. Afficher et focus la fenetre AIRADCR

Un logiciel malveillant pourrait forcer la navigation vers un tid specifique pour detourner l'attention du radiologue.

**Correction** : Exiger au minimum une cle API valide via `X-API-Key`.

---

#### 13. Dependance `md5` presente dans Cargo.toml mais non utilisee

**Fichier** : `src-tauri/Cargo.toml`, ligne 49

La crate `md5` est listee comme dependance mais n'est pas utilisee dans le code (SHA-256 est utilise partout). Sa presence augmente inutilement la surface d'attaque et la taille du binaire.

**Correction** : Supprimer `md5 = "0.7"` de Cargo.toml.

---

### FAIBLE (amelioration)

#### 14. Pas de validation TID dans `open-report` handler

**Fichier** : `src-tauri/src/http_server/handlers.rs`, lignes 764-770

Le `tid` recu via query parameter n'est pas valide via `validate_technical_id()` avant d'etre emis dans l'evenement Tauri. Un tid malveillant pourrait contenir des caracteres speciaux.

**Correction** : Ajouter `validate_technical_id(&tid)?` apres resolution du tid.

---

#### 15. CSP `script-src 'unsafe-inline'` dans index.html ET tauri.conf.json

**Fichiers** : `index.html` ligne 10, `tauri.conf.json`

`'unsafe-inline'` permet l'execution de scripts inline, affaiblissant la protection XSS. C'est necessaire pour Vite/React en dev mais devrait etre retire en production.

**Correction** : En production, utiliser des nonces CSP ou un hash des scripts inline.

---

#### 16. Token TEO Hub API stocke en clair dans config.toml

**Fichier** : `src-tauri/src/config.rs` -- champ `api_token`

Le token d'authentification TEO Hub est stocke en texte clair dans le fichier de configuration sur disque.

**Correction** : Stocker le token dans le keychain OS au lieu du fichier TOML.

---

## RESUME DES CORRECTIONS PAR PRIORITE

| # | Severite | Faille | Fichier | Effort |
|---|----------|--------|---------|--------|
| 1 | CRITIQUE | `clipboard_modifier()` oublie dans `perform_injection_at_position_direct` | main.rs | 3 lignes |
| 2 | CRITIQUE | SQLite non chiffree (donnees medicales) | Cargo.toml + mod.rs | Moyen |
| 3 | CRITIQUE | Endpoints de monitoring sans authentification | routes.rs + handlers | 20 lignes |
| 4 | CRITIQUE | Cle API loguee en clair | schema.rs | 1 ligne |
| 5 | HAUTE | Pas de limite Content-Length | mod.rs | 1 ligne |
| 6 | HAUTE | XSS potentiel dans `simulate_key_in_iframe` | main.rs | 5 lignes |
| 7 | HAUTE | Chemins de logs Windows-only | main.rs | 10 lignes |
| 8 | HAUTE | Rate limiting trop permissif pour brute-force | mod.rs | 15 lignes |
| 9 | MOYENNE | DELETE sans authentification | handlers.rs | 10 lignes |
| 10 | MOYENNE | Pas de HSTS | index.html | 1 ligne |
| 11 | MOYENNE | `println!` au lieu de `log::` | handlers.rs + schema.rs | 30 lignes |
| 12 | MOYENNE | POST /open-report sans authentification | handlers.rs | 10 lignes |
| 13 | MOYENNE | Dependance md5 inutilisee | Cargo.toml | 1 ligne |
| 14 | FAIBLE | Pas de validation TID dans open-report | handlers.rs | 3 lignes |
| 15 | FAIBLE | `unsafe-inline` dans CSP | index.html | Complexe |
| 16 | FAIBLE | Token TEO Hub en clair dans config | config.rs | Moyen |

## PLAN D'IMPLEMENTATION

### Phase 1 -- Corrections immediates (zero risque de casse)

Corrections purement additives ou de remplacement minimal :

1. **Faille #1** : Remplacer `Key::Control` par `clipboard_modifier()` dans `perform_injection_at_position_direct` (3 lignes)
2. **Faille #4** : Masquer la cle dans le log (1 ligne)
3. **Faille #5** : Ajouter `.app_data(web::JsonConfig::default().limit(1_048_576))` (1 ligne)
4. **Faille #6** : Valider `key` contre liste blanche AVANT insertion JS (5 lignes)
5. **Faille #13** : Supprimer `md5 = "0.7"` de Cargo.toml (1 ligne)

### Phase 2 -- Renforcement authentification (changement de comportement)

6. **Faille #3** : Proteger `/health/extended` et `/metrics` avec cle admin
7. **Faille #9** : Exiger API key pour DELETE /pending-report
8. **Faille #12** : Exiger API key pour POST /open-report
9. **Faille #14** : Valider TID dans open-report handler

### Phase 3 -- Ameliorations structurelles

10. **Faille #7** : Remplacer les chemins Windows-only par `PathBuf`
11. **Faille #8** : Renforcer le rate limiting pour echecs 401
12. **Faille #10** : Ajouter HSTS header
13. **Faille #11** : Migrer `println!` vers `log::` macros

### Phase 4 -- Chiffrement (changement majeur, necessite migration)

14. **Faille #2** : Migrer vers SQLCipher
15. **Faille #16** : Stocker les tokens dans le keychain OS

## CE QUI EST BIEN FAIT (points forts confirmes)

- SHA-256 pour toutes les cles API (pas de MD5)
- Comparaison en temps constant (`constant_time_compare`) contre timing attacks
- Masquage PII dans les logs (patient_id: 1234****)
- Deep link validation (max 64 chars, regex alphanumerique)
- CORS restrictif (uniquement localhost et airadcr.com)
- Rate limiting global via actix-governor
- Serveur HTTP ecoute uniquement sur 127.0.0.1
- Backup automatique SQLite quotidien
- Cle admin externalisee obligatoire en production
- CSP dans index.html + tauri.conf.json
- Protection clickjacking (X-Frame-Options: DENY)
- Sandbox iframe avec permissions minimales
- Validation postMessage par origin + type
- Single-instance protection
- Debounce et deduplication des injections

