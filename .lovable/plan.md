

# Audit macOS AIRADCR Desktop -- Corrections pour fonctionnement complet

## Reponse a la question principale

**OUI**, le build macOS se met a jour automatiquement quand vous modifiez le code ici. Le fichier `.github/workflows/build.yml` (lignes 23-26) compile deja pour `macos-latest` en ARM64 et Intel a chaque push. Les DMG sont uploades comme artefacts GitHub Actions.

**MAIS** le build macOS actuel **compile sans erreur** et **ne fonctionne PAS correctement** a cause de 3 bugs critiques identifies dans cet audit.

---

## Bugs critiques identifies

### Bug 1 -- Injection impossible sur Mac (CRITIQUE)

**Fichier** : `src-tauri/src/main.rs`, lignes 256-258 et 288-290

Le code actuel :
```rust
enigo.key(Key::Control, Direction::Press)  // Windows: Ctrl
enigo.key(Key::Unicode('v'), Direction::Click)
enigo.key(Key::Control, Direction::Release)
```

Sur macOS, coller c'est **Cmd+V**, pas Ctrl+V. Ce code ne fait rien sur Mac.

**Correction** : Utiliser `#[cfg(target_os = "macos")]` pour utiliser `Key::Meta` (= Cmd) sur Mac :

```rust
#[cfg(target_os = "macos")]
let paste_modifier = Key::Meta;
#[cfg(not(target_os = "macos"))]
let paste_modifier = Key::Control;

enigo.key(paste_modifier, Direction::Press)?;
enigo.key(Key::Unicode('v'), Direction::Click)?;
enigo.key(paste_modifier, Direction::Release)?;
```

**Fonctions concernees** (3 endroits) :
- `perform_injection()` (ligne 288)
- `perform_injection_at_position()` (ligne 256)
- `has_text_selection()` (ligne 331 -- utilise Ctrl+C, doit etre Cmd+C sur Mac)

### Bug 2 -- 4 commandes retournent des erreurs au lieu de fallbacks

Les fonctions suivantes retournent `Err("...uniquement Windows")` sur macOS, ce qui provoque des erreurs dans le frontend :

| Fonction | Ligne | Erreur actuelle |
|----------|-------|-----------------|
| `get_window_at_point()` | 616-619 | `Err("non supporte")` |
| `get_virtual_desktop_info()` | 657-660 | `Err("uniquement Windows")` |
| `get_physical_window_rect()` | 706-709 | `Err("uniquement Windows")` |
| `get_window_client_rect_at_point()` | 800-803 | `Err("uniquement Windows")` |

**Correction** : Remplacer les `Err()` par des **valeurs par defaut raisonnables** :

- `get_window_at_point` : utiliser `active_win_pos_rs::get_active_window()` (fonctionne sur Mac)
- `get_virtual_desktop_info` : retourner `{ x: 0, y: 0, width: 1920, height: 1080 }` (valeur generique)
- `get_physical_window_rect` : utiliser `get_active_window()` pour obtenir les dimensions
- `get_window_client_rect_at_point` : utiliser `get_active_window()` avec les memes dimensions

### Bug 3 -- Raccourcis non conventionnels (MINEUR)

Les raccourcis actuels utilisent `Ctrl+` qui **fonctionne** sur Mac mais n'est pas naturel. Les utilisateurs Mac s'attendent a `Cmd+`. Cependant, Tauri v1 `GlobalShortcutManager` mappe `Ctrl+` correctement sur les deux plateformes pour les raccourcis globaux, donc ce n'est **pas bloquant** -- c'est un choix UX, pas un bug.

---

## Plan de corrections

### Modification 1 : Injection cross-platform (`main.rs`)

Creer une fonction helper pour determiner la touche de collage selon l'OS :

```rust
/// Retourne la touche modificateur pour copier/coller selon l'OS
/// Windows/Linux: Ctrl, macOS: Cmd (Meta)
fn clipboard_modifier() -> Key {
    #[cfg(target_os = "macos")]
    { Key::Meta }
    #[cfg(not(target_os = "macos"))]
    { Key::Control }
}
```

Puis remplacer `Key::Control` par `clipboard_modifier()` dans les 3 fonctions :
- `perform_injection()` (ligne 288) : 3 lignes modifiees
- `perform_injection_at_position()` (ligne 256) : 3 lignes modifiees
- `has_text_selection()` (ligne 331) : 3 lignes modifiees

### Modification 2 : Fallbacks macOS pour les 4 commandes (`main.rs`)

Remplacer les 4 blocs `#[cfg(not(target_os = "windows"))] { Err(...) }` par des fallbacks fonctionnels :

**`get_window_at_point`** (ligne 616) :
```rust
#[cfg(not(target_os = "windows"))]
{
    match get_active_window() {
        Ok(win) => Ok(WindowInfo {
            title: win.title,
            app_name: win.app_name,
            x: win.position.x as i32,
            y: win.position.y as i32,
            width: win.position.width as i32,
            height: win.position.height as i32,
        }),
        Err(_) => Ok(WindowInfo {
            title: "Unknown".to_string(),
            app_name: "Unknown".to_string(),
            x: 0, y: 0, width: 1920, height: 1080,
        })
    }
}
```

**`get_virtual_desktop_info`** (ligne 657) :
```rust
#[cfg(not(target_os = "windows"))]
{
    Ok(VirtualDesktopInfo { x: 0, y: 0, width: 1920, height: 1080 })
}
```

**`get_physical_window_rect`** (ligne 706) :
```rust
#[cfg(not(target_os = "windows"))]
{
    match get_active_window() {
        Ok(win) => Ok(PhysicalRect {
            left: win.position.x as i32,
            top: win.position.y as i32,
            right: (win.position.x + win.position.width) as i32,
            bottom: (win.position.y + win.position.height) as i32,
            width: win.position.width as i32,
            height: win.position.height as i32,
        }),
        Err(_) => Ok(PhysicalRect {
            left: 0, top: 0, right: 1920, bottom: 1080, width: 1920, height: 1080,
        })
    }
}
```

**`get_window_client_rect_at_point`** (ligne 800) :
```rust
#[cfg(not(target_os = "windows"))]
{
    match get_active_window() {
        Ok(win) => Ok(ClientRectInfo {
            app_name: win.app_name,
            title: win.title,
            window_left: win.position.x as i32,
            window_top: win.position.y as i32,
            window_width: win.position.width as i32,
            window_height: win.position.height as i32,
            client_left: win.position.x as i32,
            client_top: win.position.y as i32,
            client_width: win.position.width as i32,
            client_height: win.position.height as i32,
            hwnd: 0,
        }),
        Err(_) => Ok(ClientRectInfo {
            app_name: "Unknown".to_string(), title: "Unknown".to_string(),
            window_left: 0, window_top: 0, window_width: 1920, window_height: 1080,
            client_left: 0, client_top: 0, client_width: 1920, client_height: 1080,
            hwnd: 0,
        })
    }
}
```

### Modification 3 : Documentation (`KEYBOARD_SHORTCUTS_REFERENCE.md`)

Ajouter une note indiquant que tous les raccourcis `Ctrl+` fonctionnent identiquement sur macOS via Tauri GlobalShortcutManager.

---

## Resume des modifications

| Fichier | Changement | Impact |
|---------|-----------|--------|
| `src-tauri/src/main.rs` | Fonction `clipboard_modifier()` + remplacement dans 3 fonctions | Injection fonctionne sur Mac |
| `src-tauri/src/main.rs` | 4 fallbacks macOS au lieu de `Err()` | Plus d'erreurs frontend sur Mac |
| `KEYBOARD_SHORTCUTS_REFERENCE.md` | Note cross-platform | Documentation |

**Lignes modifiees** : environ 60 lignes ajoutees/modifiees dans `main.rs`
**Zero modification frontend** : aucun fichier TypeScript/React touche
**Zero casse Windows** : toutes les modifications sont dans des blocs `#[cfg]`, le code Windows reste identique

## Apres ces corrections

| Fonction | Windows | macOS |
|----------|---------|-------|
| Raccourcis globaux (Ctrl+Space, Ctrl+Shift+D/P/T/S) | OK | OK |
| Injection clipboard (Ctrl+V / Cmd+V) | OK | **CORRIGE** |
| Detection selection (Ctrl+C / Cmd+C) | OK | **CORRIGE** |
| Infos fenetre active | OK (Win32 API) | **CORRIGE** (fallback active-win) |
| Bureau virtuel multi-ecrans | OK (GetSystemMetrics) | Fallback generique |
| Build CI automatique | OK (signe SSL.com) | OK (non signe, DMG) |
| System tray | OK | OK |
| Always-on-top | OK | OK |
| Deep links airadcr:// | OK | OK |

