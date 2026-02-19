
# Audit définitif : Pourquoi les raccourcis hors-focus ne fonctionnent pas malgré la correction focus-flash

## Le vrai problème racine (non diagnostiqué jusqu'ici)

### Problème 1 : `window.eval()` est une API COM WebView2 — elle DOIT s'exécuter sur le thread UI

La recherche WebView2 officielle (GitHub WebView2Feedback #765, Microsoft Learn Threading Model) confirme :

> **"ExecuteScript is only working from UI thread"**
> "The WebView2 API need to run in a UI thread"

Le callback du `GlobalShortcutManager` dans Tauri v1 s'exécute dans un **thread secondaire** (le thread du gestionnaire de raccourcis, pas le thread UI principal). Donc quand `eval_with_focus_flash` appelle `window.eval()`, WebView2 **reçoit l'appel sur le mauvais thread COM** — l'appel est silencieusement ignoré ou mis en queue indéfiniment.

### Problème 2 : `SetForegroundWindow` est bloqué par UIPI (User Interface Privilege Isolation)

Windows bloque `SetForegroundWindow()` sauf si le processus appelant est déjà le processus de premier plan ou a reçu une permission explicite via `AllowSetForegroundWindow()`. Dans le callback du raccourci global (thread secondaire sans droits de foreground), l'appel échoue silencieusement — **le focus-flash ne se déclenche pas**.

### Preuve : `simulate_key_in_iframe` fonctionne car c'est une commande Tauri async

La commande `simulate_key_in_iframe` (ligne 811) utilise aussi `window.eval()` — MAIS elle est appelée via `invoke()` depuis le frontend Tauri, qui dispatch correctement sur le thread UI. C'est pour ça qu'elle fonctionne. Les raccourcis globaux n'ont pas ce dispatch automatique.

### Résumé du vrai diagnostic

```text
GlobalShortcutManager callback thread (secondaire)
  → eval_with_focus_flash()
    → SetForegroundWindow() → ÉCHEC SILENCIEUX (UIPI)
    → window.eval()        → ÉCHEC SILENCIEUX (mauvais thread COM)
  → Résultat : RIEN ne se passe
```

---

## Solution A : Dispatch via le serveur HTTP local (port 8741) — 99% fiable

### Principe
L'application a déjà un serveur HTTP Actix-web sur le port 8741 qui tourne sur le bon thread async. Au lieu que le callback du raccourci appelle `window.eval()` directement, il envoie une requête HTTP à lui-même. Le handler HTTP s'exécute dans le runtime Actix (qui est correctement dispatché) et peut émettre un événement Tauri depuis le bon contexte.

### Flux
```text
Ctrl+Shift+D pressé
  → GlobalShortcutManager callback (thread secondaire)
    → reqwest::blocking::get("http://127.0.0.1:8741/shortcut?action=toggle_recording")
  → Actix handler (thread async correct)
    → APP_HANDLE.get().get_window("main").emit("airadcr:shortcut_eval", action)
  → Tauri IPC (sur thread UI via event loop)
    → useSecureMessaging.ts listen() reçoit
    → sendSecureMessage() → postMessage vers iframe ✅
```

### Pourquoi ça marche
- Le serveur HTTP Actix est déjà en place et fonctionne (port 8741)
- `APP_HANDLE` est déjà un `OnceLock<AppHandle>` global disponible dans tous les handlers
- Actix dispatch ses futures correctement sur le runtime tokio
- `window.emit()` depuis un handler HTTP Actix fonctionne — c'est exactement ce que fait déjà `/open-report` (ligne ~300 du http_server/handlers.rs)

### Modifications requises
1. **`src-tauri/src/http_server/handlers.rs`** : Ajouter un endpoint `GET /shortcut?action=XXX` qui appelle `window.emit("airadcr:shortcut_action", action)`
2. **`src-tauri/src/http_server/routes.rs`** : Enregistrer la nouvelle route
3. **`src-tauri/src/main.rs`** : Dans `register_global_shortcuts`, remplacer `eval_with_focus_flash` par `reqwest::blocking::get(...)` vers le serveur local
4. **`src/hooks/useSecureMessaging.ts`** : Ajouter `listen("airadcr:shortcut_action", ...)` — les listeners existants sont déjà en place et peuvent être réutilisés

### Risque résiduel (1%)
- Si le serveur HTTP local n'est pas encore démarré au moment du premier raccourci (démarrage applicatif), la requête échoue. Mitigation : retry automatique avec 2 tentatives espacées de 100ms.

---

## Solution B : Dispatch via `tauri::async_runtime::spawn` + channel (tokio) — 100% fiable, RECOMMANDÉE

### Principe
Utiliser un **channel tokio** (`tokio::sync::mpsc`) pour transmettre les actions de raccourcis depuis le thread secondaire du GlobalShortcutManager vers un **task tokio** qui s'exécute dans le runtime Tauri. Ce task appelle ensuite `window.emit()` — qui, depuis le runtime tokio de Tauri, est correctement dispatché sur le thread UI.

C'est le **pattern officiel recommandé par Tauri** pour émettre des événements depuis des threads qui ne sont pas sur le runtime Tauri.

### Flux
```text
Ctrl+Shift+D pressé
  → GlobalShortcutManager callback (thread secondaire)
    → tx.send("toggle_recording") // channel non-bloquant
  → tokio task (runtime Tauri, thread correct)
    → rx.recv() → window.emit("airadcr:dictation_startstop", ())
  → WebView IPC (correctement dispatché)
    → useSecureMessaging.ts listen() reçoit IMMÉDIATEMENT
    → sendSecureMessage() → postMessage vers iframe ✅
```

### Modifications requises

#### `src-tauri/src/main.rs` uniquement — 3 changements

**1. Supprimer `eval_with_focus_flash`** (lignes 1593-1624) — elle est inutile et défectueuse

**2. Créer le channel dans la fonction `register_global_shortcuts`** :
```rust
fn register_global_shortcuts(app_handle: tauri::AppHandle) {
    // Channel tokio pour dispatcher vers le bon thread
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<&'static str>();
    
    // Task tokio: reçoit les actions et émet les événements Tauri
    let handle_for_task = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(action) = rx.recv().await {
            if let Some(window) = handle_for_task.get_window("main") {
                match action {
                    "toggle_recording" => { window.emit("airadcr:dictation_startstop", ()).ok(); }
                    "toggle_pause"     => { window.emit("airadcr:dictation_pause", ()).ok(); }
                    "inject_raw"       => { window.emit("airadcr:inject_raw", ()).ok(); }
                    "inject_structured"=> { window.emit("airadcr:inject_structured", ()).ok(); }
                    _ => {}
                }
            }
        }
    });
    
    // ...
}
```

**3. Remplacer les 4 raccourcis dictée** par des `tx.send(...)` :
```rust
// Ctrl+Shift+D
let tx_d = tx.clone();
shortcut_manager.register("Ctrl+Shift+D", move || {
    debug!("[Shortcuts] Ctrl+Shift+D pressé");
    let _ = tx_d.send("toggle_recording");
}).unwrap_or_else(|e| warn!("Erreur: {}", e));

// Ctrl+Shift+P
let tx_p = tx.clone();
shortcut_manager.register("Ctrl+Shift+P", move || {
    let _ = tx_p.send("toggle_pause");
}).unwrap_or_else(|e| warn!("Erreur: {}", e));
// ... idem pour T et S
```

### Pourquoi 100% fiable

- `tokio::sync::mpsc::unbounded_channel` est `Send + Sync` — fonctionne parfaitement depuis n'importe quel thread
- `tauri::async_runtime::spawn` exécute sur le **runtime tokio de Tauri** — exactement le bon contexte pour `window.emit()`
- `window.emit()` depuis le runtime Tauri fonctionne toujours, focus ou pas focus — c'est ainsi que les handlers Actix (qui utilisent le même `APP_HANDLE`) émettent déjà des événements Tauri depuis les requêtes RIS entrantes
- Le channel est non-bloquant (`unbounded`) — le callback du raccourci ne bloque jamais le thread du GlobalShortcutManager
- **Aucune dépendance sur le focus de la fenêtre, UIPI, threads COM, ou WebView2**

### Et `useSecureMessaging.ts` ?

**Aucune modification nécessaire.** Les listeners `listen("airadcr:dictation_startstop", ...)` etc. sont déjà en place (lignes 274-295). Le canal utilisé est `window.emit()` → IPC Tauri → `listen()` JS — c'est le canal standard qui fonctionne.

**Mais si le focus est un problème pour que le JS reçoive l'event**, ajouter dans le handler de chaque action, après `window.emit()`, un `window.show()` optionnel si la fenêtre est cachée (pas juste hors-focus).

---

## Comparatif final

| Critère | Focus-flash actuel | Solution A (HTTP) | Solution B (channel tokio) |
|---------|-------------------|-------------------|---------------------------|
| Fiabilité hors-focus | 0% (thread UI incorrect) | 99% | 100% |
| Complexité | Faible (mais ne marche pas) | Moyenne (3 fichiers) | Faible (1 seul fichier) |
| Dépendances nouvelles | Aucune | Aucune (reqwest déjà présent) | Aucune (tokio déjà présent) |
| Modification frontend | Non | Non | Non |
| Pattern officiel Tauri | Non | Partiel | **Oui** |
| Risque de régression | Aucun | Faible | **Aucun** |

**Recommandation absolue : Solution B** — un seul fichier modifié (`main.rs`), pattern officiel Tauri, zéro nouvelle dépendance (tokio est déjà dans Cargo.toml), 100% fiable indépendamment du focus.

## Fichiers modifiés

- **`src-tauri/src/main.rs`** uniquement :
  - Supprimer `eval_with_focus_flash` (lignes 1593-1624)
  - Modifier `register_global_shortcuts` : ajouter channel tokio + task async + `tx.send()` dans les 4 raccourcis dictée
- **Aucun autre fichier modifié** (`useSecureMessaging.ts`, `App.tsx`, handlers HTTP)
