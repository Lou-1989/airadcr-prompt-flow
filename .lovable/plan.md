

# Plan de correction macOS -- Bugs microphone + crash injection

## Verification de l'analyse

Apres inspection complete du code, voici le statut reel :

| Point audite | Statut actuel | Action requise |
|---|---|---|
| Raccourcis CmdOrCtrl | DEJA CORRIGE | Aucune |
| Commandes Accessibility (check/request) | DEJA CORRIGE | Aucune |
| Frontend check Accessibility au demarrage | DEJA CORRIGE | Aucune |
| `sandbox` iframe manque `allow-microphone` | **BUG CONFIRME** | Correction requise |
| CSP manque `media-src` | **BUG CONFIRME** | Correction requise |
| Pas de garde Accessibility avant Enigo | **BUG CONFIRME** | Correction requise |
| `CmdOrCtrl+Space` conflit Spotlight macOS | **BUG CONFIRME** | Correction requise |
| Entitlement `allow-unsigned-executable-memory` | **MANQUANT** | Correction requise |
| Fallback hardcode 1920x1080 | **PRESENT** (lignes 823, 827) | Amelioration |

**5 corrections restantes, toutes macOS-only ou cross-platform sans impact Windows/Linux.**

---

## Corrections a implementer

### 1. Microphone : ajouter `allow-microphone` au sandbox iframe

**Fichier** : `src/security/SecurityConfig.ts` -- ligne 32

Le sandbox iframe bloque `getUserMedia()` car il ne contient pas `allow-microphone`. C'est la cause racine du bug microphone.

**Avant** :
```
sandbox: 'allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-clipboard-read allow-clipboard-write allow-modals'
```

**Apres** :
```
sandbox: 'allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-clipboard-read allow-clipboard-write allow-modals allow-microphone allow-camera'
```

Aussi ajouter `display-capture` dans l'attribut `allow` (ligne 30) pour future-proofing :
```
allow: 'clipboard-read; clipboard-write; fullscreen; microphone; camera; autoplay; display-capture'
```

**Impact Windows/Linux** : Aucun. Ces tokens n'ont d'effet que si le navigateur les supporte, et ils ne retirent rien.

### 2. CSP : ajouter `media-src` pour les flux audio WebKit

**Fichier** : `src-tauri/tauri.conf.json` -- ligne 147

Ajouter `media-src 'self' blob: mediastream:` dans la CSP pour autoriser les flux audio dans WKWebView.

**Impact Windows/Linux** : Aucun. La directive est inerte si non utilisee.

### 3. Garde Accessibility obligatoire avant chaque appel Enigo

**Fichier** : `src-tauri/src/main.rs`

Creer une fonction helper `ensure_accessibility()` qui verifie `AXIsProcessTrusted()` et retourne une erreur propre. L'appeler au debut de :
- `perform_injection_at_position` (ligne 248)
- `perform_injection` (ligne 295)
- `has_text_selection` (ligne 348)
- `perform_injection_at_position_direct` (ligne 464)

```rust
#[cfg(target_os = "macos")]
fn ensure_accessibility() -> Result<(), String> {
    unsafe {
        extern "C" { fn AXIsProcessTrusted() -> u8; }
        if AXIsProcessTrusted() == 0 {
            return Err("Permission Accessibilite macOS non accordee. Ouvrez Preferences Systeme > Confidentialite > Accessibilite et autorisez AIRADCR.".to_string());
        }
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn ensure_accessibility() -> Result<(), String> { Ok(()) }
```

**Impact Windows/Linux** : La version non-macOS retourne toujours `Ok(())`. Zero impact.

### 4. Remplacer `CmdOrCtrl+Space` par `Alt+Space`

**Fichier** : `src-tauri/src/main.rs` -- lignes 2039-2045

`Cmd+Space` est intercepte par Spotlight sur macOS. Le raccourci ne fonctionnera jamais sans que l'utilisateur desactive Spotlight. Remplacer par `Alt+Space` qui n'a pas de conflit.

De meme, remplacer `CmdOrCtrl+Shift+Space` (ligne 2050) par `Alt+Shift+Space`.

**Impact Windows/Linux** : Changement de `Ctrl+Space` vers `Alt+Space`. C'est un changement comportemental mais `Alt+Space` est plus accessible (pas de conflit Windows non plus, `Alt+Space` ouvre le menu systeme mais est intercepte par le global shortcut en premier).

### 5. Ajouter entitlement `allow-unsigned-executable-memory`

**Fichier** : `src-tauri/Entitlements.plist`

Ajouter `com.apple.security.cs.allow-unsigned-executable-memory` pour compatibilite avec le hardened runtime (requis par `enigo` et `hidapi` sur certaines versions macOS).

**Impact Windows/Linux** : Ce fichier n'est utilise que pour les builds macOS.

### 6. Ameliorer les fallbacks 1920x1080

**Fichier** : `src-tauri/src/main.rs` -- lignes 822-828

Remplacer les fallbacks silencieux par des logs `warn!` et des valeurs zero (au lieu de 1920x1080 qui peuvent fausser les calculs de positionnement).

**Impact Windows/Linux** : Aucun. Ces lignes sont dans un bloc `#[cfg(target_os = "macos")]`.

---

## Resume des fichiers modifies

| Fichier | Modification | Impact Windows |
|---|---|---|
| `src/security/SecurityConfig.ts` | `allow-microphone allow-camera` dans sandbox | Aucun |
| `src-tauri/tauri.conf.json` | `media-src` dans CSP | Aucun |
| `src-tauri/src/main.rs` | `ensure_accessibility()` + appels dans 4 fonctions | Aucun (noop) |
| `src-tauri/src/main.rs` | `Alt+Space` au lieu de `CmdOrCtrl+Space` | Mineur |
| `src-tauri/src/main.rs` | Fallbacks 0 au lieu de 1920x1080 | Aucun |
| `src-tauri/Entitlements.plist` | Entitlement memoire | macOS only |

