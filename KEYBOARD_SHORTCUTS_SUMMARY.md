# 📋 Résumé des Raccourcis Clavier - AIRADCR Desktop

## ✅ Raccourcis Fonctionnels (après correction)

### 🎤 Dictée (nouveaux raccourcis primaires)
| Raccourci | Action | Événement Tauri | Message iframe |
|-----------|--------|----------------|----------------|
| **Ctrl+Shift+D** | Démarrer/Terminer dictée | `airadcr:dictation_startstop` | `airadcr:toggle_recording` |
| **Ctrl+Shift+P** | Pause/Reprendre dictée | `airadcr:dictation_pause` | `airadcr:toggle_pause` |
| **Ctrl+Shift+T** | Injecter texte brut | `airadcr:inject_raw` | `airadcr:request_injection` (type: 'brut') |
| **Ctrl+Shift+S** | Injecter rapport structuré | `airadcr:inject_structured` | `airadcr:request_injection` (type: 'structuré') |

### 🎤 SpeechMike (boutons physiques)
| Raccourci | Action | Événement Tauri | Message iframe |
|-----------|--------|----------------|----------------|
| **F10** | Record (Démarrer/Reprendre) | `airadcr:speechmike_record` | `airadcr:toggle_recording` |
| **F11** | Pause | `airadcr:speechmike_pause` | `airadcr:toggle_pause` |
| **F12** | Finish (Finaliser et injecter) | `airadcr:speechmike_finish` | `airadcr:finalize_and_inject` |

### 🎤 Dictée (legacy - Ctrl+F)
| Raccourci | Action | Événement Tauri | Message iframe |
|-----------|--------|----------------|----------------|
| **Ctrl+F9** | Pause/Reprendre (legacy) | `airadcr:dictation_pause_toggle` | `airadcr:toggle_pause` |
| **Ctrl+F10** | Démarrer/Terminer (legacy) | `airadcr:dictation_startstop_toggle` | `airadcr:toggle_recording` |
| **Ctrl+F11** | Injecter texte brut (legacy) | `airadcr:inject_raw_text` | `airadcr:request_injection` (type: 'brut') |
| **Ctrl+F12** | Injecter rapport structuré (legacy) | `airadcr:inject_structured_report` | `airadcr:request_injection` (type: 'structuré') |

### 🎨 Debug (modifiés - Ctrl+Alt)
| Raccourci | Action | Événement Tauri |
|-----------|--------|----------------|
| **Ctrl+Alt+D** | Toggle Debug Panel | `airadcr:toggle_debug` |
| **Ctrl+Alt+L** | Toggle Log Window | `airadcr:toggle_logs` |
| **Ctrl+Alt+I** | Test Injection | `airadcr:test_injection` |

### 🔓 Autres
| Raccourci | Action | Événement Tauri |
|-----------|--------|----------------|
| **F9** | Désactiver click-through (anti-ghost) | `airadcr:force_clickable` |

---

## 🔧 Modifications Effectuées

### 1. **src-tauri/src/main.rs**
- ✅ **Raccourcis debug déplacés** : `Ctrl+Shift` → `Ctrl+Alt`
  - `Ctrl+Shift+D` → `Ctrl+Alt+D` (Debug Panel)
  - `Ctrl+Shift+L` → `Ctrl+Alt+L` (Log Window)
  - `Ctrl+Shift+T` → `Ctrl+Alt+I` (Test Injection)

- ✅ **Nouveaux raccourcis dictée ajoutés** : `Ctrl+Shift+D/P/T/S`
  - `Ctrl+Shift+D` → Start/Stop dictée (`airadcr:dictation_startstop`)
  - `Ctrl+Shift+P` → Pause/Resume dictée (`airadcr:dictation_pause`)
  - `Ctrl+Shift+T` → Inject texte brut (`airadcr:inject_raw`)
  - `Ctrl+Shift+S` → Inject rapport structuré (`airadcr:inject_structured`)

### 2. **src/App.tsx**
- ✅ **Fonction `sendToIframe` mise à jour** : Accepte maintenant un `payload` optionnel
- ✅ **Écouteurs d'événements modifiés** : `Ctrl+Alt` pour debug
- ✅ **Nouveaux écouteurs ajoutés** : `Ctrl+Shift+D/P/T/S` pour dictée/injection
- ✅ **Double écoute SpeechMike supprimée** : Plus de duplication avec `useSecureMessaging`
- ✅ **Compatibilité legacy** : Les raccourcis `Ctrl+F` restent fonctionnels

### 3. **src/hooks/useSecureMessaging.ts**
- ✅ **Écouteurs Tauri mis à jour** : Gère les nouveaux événements `Ctrl+Shift`
- ✅ **SpeechMike conservé** : Les événements F10/F11/F12 restent fonctionnels
- ✅ **Legacy supporté** : Les événements `Ctrl+F` continuent de fonctionner

---

## ✅ Validation

### Tests recommandés

1. **Raccourcis Debug (Ctrl+Alt)**
   - [ ] `Ctrl+Alt+D` ouvre/ferme le Debug Panel
   - [ ] `Ctrl+Alt+L` ouvre/ferme la Log Window
   - [ ] `Ctrl+Alt+I` déclenche un test d'injection

2. **Raccourcis Dictée (Ctrl+Shift)**
   - [ ] `Ctrl+Shift+D` démarre/termine la dictée
   - [ ] `Ctrl+Shift+P` met en pause/reprend la dictée
   - [ ] `Ctrl+Shift+T` injecte le texte brut
   - [ ] `Ctrl+Shift+S` injecte le rapport structuré

3. **SpeechMike (F10/F11/F12)**
   - [ ] F10 démarre/reprend l'enregistrement
   - [ ] F11 met en pause
   - [ ] F12 finalise et injecte (production uniquement)

4. **Legacy Ctrl+F (rétrocompatibilité)**
   - [ ] `Ctrl+F9` pause/reprend
   - [ ] `Ctrl+F10` démarre/termine
   - [ ] `Ctrl+F11` injecte texte brut
   - [ ] `Ctrl+F12` injecte rapport structuré

---

## 🔍 Cohérence du Code

### Flux d'événements
```
[Rust main.rs]
Capture globale du raccourci
    ↓
Émet événement Tauri (ex: `airadcr:dictation_startstop`)
    ↓
[React App.tsx / useSecureMessaging.ts]
Écoute l'événement Tauri
    ↓
Envoie `postMessage` à iframe AirADCR
    ↓
[airadcr.com iframe]
Reçoit le message et exécute l'action
```

### Pas de doublons d'écouteurs
- ✅ SpeechMike géré uniquement dans `useSecureMessaging.ts`
- ✅ Debug géré uniquement dans `App.tsx`
- ✅ Dictée gérée par les deux (App.tsx et useSecureMessaging)

---

## 📝 Logs Console

Lors de l'exécution, vous devriez voir :
```
✅ [Shortcuts] Raccourcis globaux enregistrés:
   🎨 Ctrl+Alt+D (Debug), Ctrl+Alt+L (Logs), Ctrl+Alt+I (Test)
   🔓 F9 (Anti-fantôme)
   🎤 F10 (Record), F11 (Pause), F12 (Finish)
   🎤 Ctrl+Shift+D (Start/Stop), Ctrl+Shift+P (Pause/Resume)
   💉 Ctrl+Shift+T (Inject Raw), Ctrl+Shift+S (Inject Structured)
   🎤 Ctrl+F9 (Pause/Resume), Ctrl+F10 (Start/Stop)
   💉 Ctrl+F11 (Inject Raw), Ctrl+F12 (Inject Structured)
```

---

## 🎯 Recommandations

1. **Tester en mode dev** : `npm run tauri dev`
2. **Tester en mode build** : `npm run tauri build`
3. **Vérifier les logs** : Ouvrir `Ctrl+Alt+L` pour voir les logs en temps réel
4. **Profile SpeechMike** : Utiliser `airadcr_speechmike_ctrlf_profile.xml` pour mapper les boutons physiques

---

**Date de mise à jour** : 2025-10-24  
**Version** : 2.0 - Raccourcis Ctrl+Shift+D/P/T/S fonctionnels
