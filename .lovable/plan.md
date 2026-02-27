

# Audit global macOS - AIRADCR Desktop

## Verdict : 80% fonctionnel -- 3 problemes bloquants restants, 4 ameliorations recommandees

Les corrections precedentes (Entitlements, Info.plist, injection avec clic, resolution dynamique) sont en place. Voici ce qui reste.

---

## Ce qui fonctionne sur macOS

| Fonctionnalite | Status | Detail |
|---|---|---|
| Affichage permanent (always-on-top) | OK | `alwaysOnTop: true` + assertion 800ms |
| Microphone (dictee vocale) | OK | `NSMicrophoneUsageDescription` + entitlement `audio-input` |
| SpeechMike USB HID | OK | `hidapi` cross-platform, detection + LED + polling |
| Serveur HTTP local (port 8741) | OK | actix-web cross-platform |
| Deep links `airadcr://` | OK | `CFBundleURLTypes` enregistre dans Info.plist |
| Base de donnees SQLCipher | OK | `keyring` utilise macOS Keychain |
| System tray | OK | Menu contextuel fonctionnel |
| Clipboard Cmd+V | OK | `clipboard_modifier()` retourne `Key::Meta` |
| Injection avec clic focus | OK | Bloc `#[cfg(target_os = "macos")]` avec move + clic + paste |
| Resolution dynamique | OK | `system_profiler SPDisplaysDataType -json` |
| Logging + dossier logs | OK | `open` commande macOS pour ouvrir le dossier |
| Raccourcis globaux | PARTIEL | Fonctionnent mais utilisent `Ctrl` au lieu de `Cmd` |
| Entitlements automation | OK | `com.apple.security.automation.apple-events` present |

---

## Problemes bloquants

### BLOQUANT 1 : Pas de verification/demande de permission Accessibility

**Probleme** : Sur macOS Ventura+ (13+), Enigo ne peut simuler aucune touche (Cmd+V, Cmd+C) sans que l'utilisateur ait accorde la permission **Accessibility** dans Preferences Systeme > Confidentialite > Accessibilite. Sans cette permission, l'injection echoue **silencieusement** -- pas d'erreur, pas de log, juste rien ne se passe.

**Impact** : Le radiologue installe l'app, clique sur "Injecter", et rien ne se passe. Aucun message ne lui explique pourquoi.

**Solution** : Ajouter une verification au demarrage via `AXIsProcessTrusted()` (Core Foundation). Si non accorde, afficher un message explicatif et ouvrir automatiquement le panneau Accessibilite. Implementer cela comme une commande Tauri `check_accessibility_permission` appelee au demarrage cote frontend.

### BLOQUANT 2 : DMG non signe et non notarise

**Probleme** : Le workflow CI ne fait aucune signature ni notarization pour macOS. Le DMG genere declenchera Gatekeeper :
- macOS Sonoma (14) : "Application non identifiee" avec bouton "Annuler" uniquement
- macOS Sequoia (15) : Bloquer plus dur, necessite terminal `xattr -cr`

**Impact** : Les radiologues ne pourront pas installer l'application sans manipulation technique avancee.

**Solution** : Ajouter les etapes `codesign` + `xcrun notarytool` dans le workflow GitHub Actions pour macOS. Necessite un certificat Apple Developer ID ($99/an).

### BLOQUANT 3 : Raccourcis macOS non-standard

**Probleme** : Tous les raccourcis utilisent `Ctrl+Shift+D/P/T/S` et `Ctrl+Space`. Sur macOS, la convention est `Cmd+Shift+...` et `Cmd+Space` (qui entre en conflit avec Spotlight). Les radiologues Mac ne trouveront pas les raccourcis intuitifs.

**Impact** : UX degradee. `Ctrl+Space` peut ne pas fonctionner si une autre app l'intercepte. Les raccourcis `Ctrl+Alt+D/L/I` entrent potentiellement en conflit avec des raccourcis systeme macOS.

**Solution** : Utiliser des raccourcis conditionnes par l'OS. Pour le `main.rs`, enregistrer `CmdOrCtrl+Shift+D/P/T/S` (Tauri les resout automatiquement par OS). Pour `Ctrl+Space`, utiliser `Alt+Space` sur macOS (pas de conflit Spotlight).

---

## Ameliorations non-bloquantes

### MOYEN 1 : Fallbacks hardcodes 1920x1080

Plusieurs fonctions non-Windows retournent `1920x1080` en cas d'echec :
- `get_window_at_point` (ligne 678)
- `get_physical_window_rect` (ligne 839)
- `get_window_client_rect_at_point` (lignes 950-954)

**Solution** : Utiliser la meme logique `system_profiler` que `get_virtual_desktop_info` comme fallback, ou au minimum logger un warning quand le fallback est utilise.

### MOYEN 2 : `get_window_at_point` ne detecte pas la fenetre au point

Sur macOS, le fallback utilise `get_active_window()` qui retourne la fenetre **active**, pas celle sous le curseur. Cela signifie que le verrouillage de cible d'injection ne fonctionne pas correctement sur macOS.

**Solution** : Utiliser `CGWindowListCopyWindowInfo` via Core Graphics pour identifier la fenetre sous un point donne. Cela necessite l'ajout de la crate `core-graphics` dans `Cargo.toml`.

### MOYEN 3 : Pas de distinction client rect sur macOS

`get_window_client_rect_at_point` retourne les memes valeurs pour window rect et client rect sur macOS (lignes 937-948). Sur Windows, la distinction est importante pour les barres de titre et bordures.

### MINEUR : `has_text_selection` peut declencher un Cmd+C parasite

La detection de selection (lignes 340-373) simule Cmd+C dans l'application cible. Sur macOS, cela peut emettre un son "boop" si rien n'est selectionne, ce qui est desagreable en milieu medical.

---

## Plan de correction (par priorite)

### Fichier : `src-tauri/src/main.rs`

**1. Ajouter `check_accessibility_permission` (BLOQUANT 1)**

Nouvelle commande Tauri qui verifie si l'Accessibility est accordee :

```rust
#[cfg(target_os = "macos")]
#[tauri::command]
fn check_accessibility_permission() -> bool {
    // AXIsProcessTrusted() via objc/core-foundation
    // Retourne true si la permission est accordee
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn request_accessibility_permission() -> Result<(), String> {
    // Ouvre System Preferences > Accessibility
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

Cote frontend (`WebViewContainer.tsx` ou `App.tsx`), verifier au demarrage et afficher un message si non accorde.

**2. Corriger les raccourcis macOS (BLOQUANT 3)**

Remplacer dans `register_global_shortcuts` :
- `"Ctrl+Shift+D"` par `"CmdOrCtrl+Shift+D"`
- `"Ctrl+Shift+P"` par `"CmdOrCtrl+Shift+P"`
- `"Ctrl+Shift+T"` par `"CmdOrCtrl+Shift+T"`
- `"Ctrl+Shift+S"` par `"CmdOrCtrl+Shift+S"`
- `"Ctrl+Space"` par `"CmdOrCtrl+Space"` (ou `"Alt+Space"` sur macOS)
- `"Ctrl+Shift+Space"` par `"CmdOrCtrl+Shift+Space"`
- `"Ctrl+Alt+D"` par `"CmdOrCtrl+Alt+D"`
- `"Ctrl+Alt+L"` par `"CmdOrCtrl+Alt+L"`
- `"Ctrl+Alt+I"` par `"CmdOrCtrl+Alt+I"`

**3. Supprimer les fallbacks hardcodes (MOYEN 1)**

Remplacer les `1920x1080` par des appels a `system_profiler` ou au minimum un log `warn!` pour alerter.

### Fichier : `src-tauri/Cargo.toml`

Ajouter sous `[target.'cfg(target_os = "macos")'.dependencies]` :

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
```

Necessaire pour `AXIsProcessTrusted()`.

### Fichier : `.github/workflows/build.yml`

Ajouter les etapes de signature macOS (necessite les secrets `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`) :

```yaml
- name: Import Apple certificate
  if: contains(matrix.settings.platform, 'macos')
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    # Import certificate into temporary keychain
    ...

- name: Notarize DMG
  if: contains(matrix.settings.platform, 'macos')
  run: |
    xcrun notarytool submit ... --wait
    xcrun stapler staple ...
```

### Fichier : `src/components/WebViewContainer.tsx` (ou `src/App.tsx`)

Ajouter une verification Accessibility au demarrage :

```typescript
useEffect(() => {
  if (window.__TAURI__) {
    invoke('check_accessibility_permission').then((granted) => {
      if (!granted) {
        // Afficher un message demandant d'activer l'Accessibility
        toast.warning("Veuillez activer l'accessibilite pour AIRADCR...");
        invoke('request_accessibility_permission');
      }
    });
  }
}, []);
```

---

## Resume des fichiers a modifier

| Fichier | Modifications |
|---|---|
| `src-tauri/src/main.rs` | Commandes Accessibility, raccourcis CmdOrCtrl, fallbacks ameliores |
| `src-tauri/Cargo.toml` | Ajout `core-foundation` pour macOS |
| `.github/workflows/build.yml` | Signature + notarization macOS |
| `src/components/WebViewContainer.tsx` | Verification Accessibility au demarrage |

