# 📋 Résumé des Raccourcis Clavier - AIRADCR Desktop

**Version:** 3.0 - Système unifié  
**Date:** 2025-10-24

---

## ✅ Raccourcis Fonctionnels - Système Unifié

### 🎤 Dictée et Injection (Un seul système pour tout)

| Raccourci | Action | Source | Événement Tauri | Message iframe |
|-----------|--------|--------|----------------|----------------|
| **Ctrl+Shift+D** | Démarrer/Terminer dictée | Clavier **OU** SpeechMike Record | `airadcr:dictation_startstop` | `airadcr:toggle_recording` |
| **Ctrl+Shift+P** | Pause/Reprendre dictée | Clavier **OU** SpeechMike Pause/Play | `airadcr:dictation_pause` | `airadcr:toggle_pause` |
| **Ctrl+Shift+T** | Injecter texte brut (Insert) | Clavier **OU** SpeechMike Instruction | `airadcr:inject_raw` | `airadcr:request_injection` (type: 'brut') |
| **Ctrl+Shift+S** | Injecter rapport structuré (EOL) | Clavier **OU** SpeechMike Programmable 1 | `airadcr:inject_structured` | `airadcr:request_injection` (type: 'structuré') |

### 🎨 Debug (Ctrl+Alt)

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

## 🎯 Principe du Système Unifié

**✅ AVANT (Version 2.0):** 3 systèmes parallèles confus
- ❌ F10/F11/F12 pour SpeechMike
- ❌ Ctrl+F9/F10/F11/F12 pour legacy
- ❌ Ctrl+Shift+D/P/T/S nouveaux raccourcis
- ❌ Couches intermédiaires multiples
- ❌ Listeners dupliqués
- ❌ Confusion sur quel raccourci utiliser

**✅ APRÈS (Version 3.0):** Un seul système, zéro confusion
- ✅ **Ctrl+Shift+D/P/T/S** partout
- ✅ SpeechMike génère directement `Ctrl+Shift+*`
- ✅ Clavier utilise directement `Ctrl+Shift+*`
- ✅ **Aucune couche intermédiaire**
- ✅ **4 raccourcis uniques** pour tout
- ✅ Code simplifié, moins de bugs

---

## 🔧 Modifications Effectuées

### 1. **airadcr_speechmike_ctrlf_profile.xml**
- ✅ **SpeechMike génère directement `Ctrl+Shift+*`**
  - Record → `Ctrl+Shift+D`
  - Pause/Play → `Ctrl+Shift+P`
  - Instruction → `Ctrl+Shift+T`
  - Programmable 1 → `Ctrl+Shift+S`
- ❌ Supprimé tous les `Ctrl+F*`

### 2. **src-tauri/src/main.rs**
- ✅ **Garde uniquement `Ctrl+Shift+D/P/T/S`**
- ❌ **Supprimé F10/F11/F12** (lignes 1225-1259)
- ❌ **Supprimé Ctrl+F9/F10/F11/F12** (lignes 1261-1303)
- ✅ **Garde Ctrl+Alt+D/L/I** pour debug
- ✅ **Garde F9** pour anti-ghost

### 3. **src/hooks/useSecureMessaging.ts**
- ✅ **Garde uniquement les nouveaux raccourcis:**
  - `airadcr:dictation_startstop` (ligne 272)
  - `airadcr:dictation_pause` (ligne 278)
  - `airadcr:inject_raw` (ligne 284)
  - `airadcr:inject_structured` (ligne 290)
- ❌ **Supprimé tous les listeners legacy:**
  - `airadcr:dictation_startstop_toggle`
  - `airadcr:dictation_pause_toggle`
  - `airadcr:inject_raw_text`
  - `airadcr:inject_structured_report`
  - `airadcr:speechmike_record`
  - `airadcr:speechmike_pause`
  - `airadcr:speechmike_finish`

### 4. **airadcr_speechmike_profile.xml**
- ✅ **Mise à jour vers v3.0 avec `Ctrl+Shift+*`**

### 5. **Documentation**
- ✅ **KEYBOARD_SHORTCUTS_REFERENCE.md** réécrit pour v3.0
- ✅ **KEYBOARD_SHORTCUTS_SUMMARY.md** (ce fichier) mis à jour

---

## ✅ Validation

### Tests recommandés

1. **Raccourcis Debug (Ctrl+Alt)**
   - [ ] `Ctrl+Alt+D` ouvre/ferme le Debug Panel
   - [ ] `Ctrl+Alt+L` ouvre/ferme la Log Window
   - [ ] `Ctrl+Alt+I` déclenche un test d'injection

2. **Raccourcis Dictée/Injection (Ctrl+Shift) - Clavier**
   - [ ] `Ctrl+Shift+D` démarre/termine la dictée
   - [ ] `Ctrl+Shift+P` met en pause/reprend la dictée
   - [ ] `Ctrl+Shift+T` injecte le texte brut
   - [ ] `Ctrl+Shift+S` injecte le rapport structuré

3. **SpeechMike (doit générer Ctrl+Shift+*)**
   - [ ] Bouton Record → Génère `Ctrl+Shift+D`
   - [ ] Bouton Pause → Génère `Ctrl+Shift+P`
   - [ ] Bouton Instruction → Génère `Ctrl+Shift+T`
   - [ ] Bouton Programmable 1 → Génère `Ctrl+Shift+S`

4. **Anti-ghost**
   - [ ] `F9` désactive le click-through

---

## 🔍 Cohérence du Code

### Flux d'événements (simplifié)

```
[SpeechMike OU Clavier]
Génère Ctrl+Shift+D/P/T/S
    ↓
[Rust main.rs]
Capture globale du raccourci
    ↓
Émet événement Tauri (ex: `airadcr:dictation_startstop`)
    ↓
[React useSecureMessaging.ts]
Écoute l'événement Tauri
    ↓
Envoie `postMessage` à iframe AirADCR
    ↓
[airadcr.com iframe]
Reçoit le message et exécute l'action
```

### Pas de doublons d'écouteurs ✅
- ✅ **Un seul listener par raccourci**
- ✅ **Un seul événement Tauri par action**
- ✅ **Un seul message iframe par action**
- ✅ **Aucune duplication de code**

---

## 📝 Logs Console

Lors de l'exécution, vous devriez voir :

```
✅ [Shortcuts] Raccourcis globaux enregistrés (Système unifié v3.0):
   🎨 Ctrl+Alt+D (Debug), Ctrl+Alt+L (Logs), Ctrl+Alt+I (Test)
   🔓 F9 (Anti-fantôme)
   🎤 Ctrl+Shift+D (Start/Stop dictée)
   🎤 Ctrl+Shift+P (Pause/Resume dictée)
   💉 Ctrl+Shift+T (Inject texte brut - Insert)
   💉 Ctrl+Shift+S (Inject rapport structuré - EOL)
   ✅ SpeechMike utilise les MÊMES raccourcis Ctrl+Shift+D/P/T/S
```

---

## 🎯 Recommandations

1. **Tester en mode dev:** `npm run tauri dev`
2. **Tester en mode build:** `npm run tauri build`
3. **Vérifier les logs:** Ouvrir `Ctrl+Alt+L` pour voir les logs en temps réel
4. **Profile SpeechMike:** Utiliser `airadcr_speechmike_ctrlf_profile.xml` pour mapper les boutons physiques

---

## 📊 Comparaison avec Version 2.0

| Aspect | Version 2.0 | Version 3.0 |
|--------|-------------|-------------|
| **Nombre de raccourcis** | 12+ raccourcis | 4 raccourcis uniques |
| **Systèmes parallèles** | 3 systèmes | 1 système unifié |
| **Listeners Tauri** | 11 listeners | 4 listeners |
| **Couches intermédiaires** | 2-3 couches | 1 couche |
| **SpeechMike vs Clavier** | Différents raccourcis | **Mêmes raccourcis** |
| **Confusion utilisateur** | ❌ Élevée | ✅ Nulle |
| **Maintenabilité** | ❌ Difficile | ✅ Simple |

---

**Date de mise à jour:** 2025-10-24  
**Version:** 3.0 - Système unifié Ctrl+Shift+D/P/T/S
