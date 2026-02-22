# ⌨️ Référence complète - Raccourcis clavier AIRADCR Desktop

**Version:** 3.0 - Système unifié  
**Date:** 2025-10-24  
**Architecture:** Rust (Tauri GlobalShortcut) + React (Listeners) + iframe airadcr.com

---

## 🎯 Vue d'ensemble du système unifié

```
┌─────────────────────────────────────────────────────────┐
│  SpeechMike / Clavier physique                          │
│  Génèrent EXACTEMENT les mêmes raccourcis:              │
│  Ctrl+Shift+D, Ctrl+Shift+P, Ctrl+Shift+T, Ctrl+Shift+S │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Tauri GlobalShortcut (Rust)                            │
│  - Capture GLOBALE (fonctionne même hors focus)         │
│  - Émet des événements Tauri                            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  React Listeners (useSecureMessaging.ts)                │
│  - Écoute les événements Tauri                          │
│  - Envoie postMessage à l'iframe                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  iframe airadcr.com                                     │
│  - Écoute les postMessage                               │
│  - Déclenche les actions de dictée/injection            │
└─────────────────────────────────────────────────────────┘
```

**🎯 PRINCIPE CLÉ:** Une seule couche de raccourcis, aucune complexité, aucun doublon.

> **📱 Note macOS :** Tous les raccourcis `Ctrl+` fonctionnent identiquement sur macOS via Tauri GlobalShortcutManager. L'injection clipboard utilise automatiquement `Cmd+V` (au lieu de `Ctrl+V`) grâce à la fonction `clipboard_modifier()` dans `main.rs`. Les commandes de détection de fenêtre utilisent `active_win_pos_rs` comme fallback sur macOS au lieu des API Win32.

---

## 🎤 Raccourcis unifiés

### 📋 Table unique des raccourcis

| Raccourci | Action | Source | Événement Tauri | Message iframe |
|-----------|--------|--------|-----------------|----------------|
| **Ctrl+Space** | Démarrer/Terminer dictée | Clavier (ergonomique) | `airadcr:dictation_startstop` | `airadcr:toggle_recording` |
| **Ctrl+Shift+Space** | Pause/Reprendre dictée | Clavier (ergonomique) | `airadcr:dictation_pause` | `airadcr:toggle_pause` |
| **Ctrl+Shift+D** | Démarrer/Terminer dictée | Clavier **OU** SpeechMike Record | `airadcr:dictation_startstop` | `airadcr:toggle_recording` |
| **Ctrl+Shift+P** | Pause/Reprendre dictée | Clavier **OU** SpeechMike Pause/Play | `airadcr:dictation_pause` | `airadcr:toggle_pause` |
| **Ctrl+Shift+T** | Injecter texte brut (Insert) | Clavier **OU** SpeechMike Instruction | `airadcr:inject_raw` | `airadcr:request_injection` (type: 'brut') |
| **Ctrl+Shift+S** | Injecter rapport structuré (EOL) | Clavier **OU** SpeechMike Programmable 1 | `airadcr:inject_structured` | `airadcr:request_injection` (type: 'structuré') |

---

## 🎤 Contrôle de dictée

### **Ctrl+Shift+D** - Démarrer/Terminer dictée (Toggle intelligent)

**Fonction:** Démarre ou termine l'enregistrement audio selon l'état actuel.

**États et actions:**

| État actuel | Action déclenchée | Nouvel état | Description UI |
|------------|-------------------|-------------|----------------|
| **Idle** | Démarre l'enregistrement | **Recording** | Bouton "🎤 Enregistrer" → "⏹️ Arrêter" |
| **Recording** | Termine la dictée | **Idle** | Bouton "⏹️ Arrêter" → Finalisation + transcription |
| **Paused** | Termine la dictée | **Idle** | Bouton "▶️ Continuer" → Finalisation + transcription |

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+Shift+D` globalement, émet `airadcr:dictation_startstop`
- **React (useSecureMessaging.ts):** Écoute `airadcr:dictation_startstop`, envoie `postMessage` type `airadcr:toggle_recording` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleStartStopToggle()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1178-1188)
- `src/hooks/useSecureMessaging.ts` (lignes ~272-275)

**Comportement SpeechMike:**
- Bouton **Record** → Génère `Ctrl+Shift+D` via profil XML

---

### **Ctrl+Shift+P** - Pause/Reprendre dictée (Toggle)

**Fonction:** Met en pause ou reprend l'enregistrement audio selon l'état actuel.

**États et actions:**

| État actuel | Action déclenchée | Nouvel état | Description UI |
|------------|-------------------|-------------|----------------|
| **Recording** | Met en pause | **Paused** | Bouton "⏸️ En pause" |
| **Paused** | Reprend | **Recording** | Bouton "⏹️ Arrêter" |
| **Idle** | Aucune action | **Idle** | Pas de changement |

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+Shift+P` globalement, émet `airadcr:dictation_pause`
- **React (useSecureMessaging.ts):** Écoute `airadcr:dictation_pause`, envoie `postMessage` type `airadcr:toggle_pause` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handlePauseResumeToggle()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1190-1200)
- `src/hooks/useSecureMessaging.ts` (lignes ~278-281)

**Comportement SpeechMike:**
- Bouton **Pause/Play** → Génère `Ctrl+Shift+P` via profil XML

---

## 📋 Injection de texte

### **Ctrl+Shift+T** - Injecter le texte dicté brut (Insert)

**Fonction:** Injecte le texte de dictée brut depuis la textarea dans l'application cible externe (RIS, Word, etc.).

**Conditions requises:**
- ✅ Le texte de dictée (`dictationText`) ne doit **pas** être vide
- ✅ La position d'injection doit être verrouillée (optionnel)

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+Shift+T` globalement, émet `airadcr:inject_raw`
- **React (useSecureMessaging.ts):** Écoute `airadcr:inject_raw`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleInjectRawText()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1195-1205)
- `src/hooks/useSecureMessaging.ts` (lignes ~284-287)

**Comportement SpeechMike:**
- Bouton **Instruction** → Génère `Ctrl+Shift+T` via profil XML

---

### **Ctrl+Shift+S** - Injecter le rapport structuré complet (EOL)

**Fonction:** Injecte le rapport structuré complet selon les préférences utilisateur dans l'application cible externe.

**Conditions requises:**
- ✅ Le rapport doit être valide (`isReportValid === true`)
- ✅ Aucun export ne doit être en cours (`isExporting === null`)
- ✅ La position d'injection doit être verrouillée (optionnel)

**Format d'injection personnalisé (généré par `generateCustomContent()`):**
```
TITRE: [Titre complet du rapport]

INDICATION:
[Indication formatée]

TECHNIQUE:
[Technique d'examen détaillée]

RÉSULTATS:
[Résultats structurés et formatés]

CONCLUSION:
[Conclusion du radiologue]

RÉFÉRENCES DOCUMENTAIRES:
[Liens vers études de référence]

TRADUCTION:
[Traduction si disponible]

EXPLICATION:
[Explications complémentaires si disponibles]
```

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+Shift+S` globalement, émet `airadcr:inject_structured`
- **React (useSecureMessaging.ts):** Écoute `airadcr:inject_structured`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleInjectStructuredReport()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1202-1212)
- `src/hooks/useSecureMessaging.ts` (lignes ~290-293)

**Comportement SpeechMike:**
- Bouton **Programmable 1** → Génère `Ctrl+Shift+S` via profil XML

---

## 🐛 Raccourcis de débogage

### **Ctrl+Alt+D** - Toggle panneau de débogage

**Fonction:** Affiche/masque le panneau de débogage avec informations système.

**Informations affichées:**
- État de l'application Tauri
- Position Always-on-Top
- Informations système (OS, architecture)
- État de verrouillage de la cible d'injection

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1140-1150)
- `src/App.tsx`
- `src/components/DebugPanel.tsx`

---

### **Ctrl+Alt+L** - Afficher fenêtre de logs

**Fonction:** Ouvre une fenêtre popup affichant les logs en temps réel de l'application.

**Logs affichés:**
- Messages Tauri (Rust)
- Messages React (TypeScript)
- Communications postMessage
- Erreurs et warnings

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1152-1162)
- `src/App.tsx`
- `src/components/DevLogWindow.tsx`

---

### **Ctrl+Alt+I** - Test d'injection

**Fonction:** Déclenche un test d'injection à la position actuelle du curseur.

**Comportement:**
- Injecte un texte de test : `"🧪 TEST INJECTION - AIRADCR Desktop"`
- Utilise la position verrouillée si disponible
- Sinon, utilise la position actuelle du curseur

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1164-1174)
- `src/App.tsx`

---

### **F9** - Anti-ghost (Désactiver click-through)

**Fonction:** Force la fenêtre à accepter les clics de souris (désactive le mode click-through).

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~1214-1222)

---

## 🎛️ Configuration SpeechMike

### Profil SpeechControl XML recommandé

**Fichier à utiliser:** `airadcr_speechmike_ctrlf_profile.xml` ou `airadcr_speechmike_profile.xml`

| Bouton physique SpeechMike | Raccourci envoyé | Action AIRADCR |
|----------------------------|------------------|----------------|
| 🔴 **Record** | `Ctrl+Shift+D` | Démarrer/Terminer dictée |
| ⏸️ **Pause** | `Ctrl+Shift+P` | Pause/Reprise toggle |
| ▶️ **Play** | `Ctrl+Shift+P` | Reprise (même que Pause) |
| 📋 **Instruction** | `Ctrl+Shift+T` | Injecter texte brut (Insert) |
| 🔧 **Programmable 1** | `Ctrl+Shift+S` | Injecter rapport structuré (EOL) |
| ⏮️ **Rewind** | (Désactivé) | Aucune action |
| ⏭️ **Fast Forward** | (Désactivé) | Aucune action |

---

## 🔄 Workflow complet (exemple d'utilisation)

### Scénario: Radiologue dictant un scanner thoracique

```
1. 📍 POSITION INITIALE
   └─ Utilisateur ouvre RIS et clique dans le champ "Compte rendu"
   
2. 🎤 DÉMARRAGE DICTÉE (Ctrl+Shift+D)
   ├─ Rust: Capture Ctrl+Shift+D globalement
   ├─ React: Envoie postMessage à l'iframe
   └─ iframe: Démarre enregistrement audio
       └─ Toast: "🎤 Dictée démarrée"
   
3. 🗣️ DICTÉE EN COURS
   └─ "Scanner thoracique. Indication pneumonie. Technique spiralée..."
   
4. ⏸️ PAUSE (Ctrl+Shift+P)
   ├─ Rust: Capture Ctrl+Shift+P globalement
   ├─ React: Envoie postMessage à l'iframe
   └─ iframe: Met en pause (MediaRecorder.pause())
       └─ Toast: "⏸️ Dictée en pause"
   
5. ▶️ REPRISE (Ctrl+Shift+P)
   ├─ Rust: Capture Ctrl+Shift+P globalement (même touche)
   └─ iframe: Reprend enregistrement
       └─ Toast: "▶️ Dictée reprise"
   
6. 🗣️ SUITE DE LA DICTÉE
   └─ "...infiltrat lobe inférieur droit. Conclusion infection confirmée."
   
7. ✅ TERMINER (Ctrl+Shift+D)
   ├─ Rust: Capture Ctrl+Shift+D globalement (même touche qu'au démarrage)
   ├─ iframe: Fusionne chunks audio
   ├─ Envoi à Voxtral API pour transcription
   ├─ Structuration automatique (GPT-4)
   └─ Toast: "✅ Transcription complète"
   
8. 📋 INJECTION RAPPORT STRUCTURÉ (Ctrl+Shift+S)
   ├─ Rust: Capture Ctrl+Shift+S globalement
   ├─ React: Envoie postMessage à l'iframe
   ├─ iframe: Récupère rapport structuré depuis ExportPanel
   ├─ Tauri: Reçoit commande perform_injection_at_position_direct()
   │   ├─ Sauvegarde clipboard original
   │   ├─ Copie rapport dans clipboard
   │   ├─ Clic à la position verrouillée dans RIS
   │   └─ Ctrl+V du rapport
   └─ RIS: Rapport inséré formaté
   
9. ✅ WORKFLOW TERMINÉ
   └─ Temps total: ~30-45 secondes (vs 5-10 min manuellement)
```

---

## 🧪 Tests de validation

### Test 1: Raccourcis clavier directs (sans SpeechMike)

**Procédure:**
1. Lancer AIRADCR Desktop
2. Appuyer sur `Ctrl+Shift+D` → Vérifier démarrage dictée ✅
3. Dicter 5 secondes de test
4. Appuyer sur `Ctrl+Shift+P` → Vérifier mise en pause ✅
5. Appuyer sur `Ctrl+Shift+P` → Vérifier reprise ✅
6. Appuyer sur `Ctrl+Shift+D` → Vérifier finalisation ✅
7. Attendre transcription (5-10s)
8. Appuyer sur `Ctrl+Shift+S` → Vérifier injection dans RIS ✅

**Résultat attendu:** Toutes les étapes fonctionnent correctement.

---

### Test 2: Capture globale (hors focus)

**Procédure:**
1. Lancer AIRADCR Desktop
2. Ouvrir Word ou RIS
3. Donner le focus à Word/RIS (cliquer dedans)
4. Appuyer sur `Ctrl+Shift+D` → Vérifier que l'enregistrement démarre dans AIRADCR ✅

**Résultat attendu:** AIRADCR réagit même sans avoir le focus (GlobalShortcut).

---

### Test 3: SpeechMike mappé sur Ctrl+Shift+*

**Prérequis:** Profil `airadcr_speechmike_ctrlf_profile.xml` installé dans SpeechControl.

**Procédure:**
1. Brancher le SpeechMike Philips
2. Appuyer sur le bouton **Record** du SpeechMike → Doit envoyer `Ctrl+Shift+D` ✅
3. Vérifier que la dictée démarre dans AIRADCR ✅
4. Appuyer sur le bouton **Pause** → Doit envoyer `Ctrl+Shift+P` ✅
5. Appuyer sur le bouton **Instruction** → Doit envoyer `Ctrl+Shift+T` ✅
6. Appuyer sur le bouton **Programmable 1** → Doit envoyer `Ctrl+Shift+S` ✅

**Résultat attendu:** Le SpeechMike contrôle AIRADCR via les mêmes raccourcis que le clavier.

---

## 📊 Résumé des événements Tauri

| Raccourci clavier | Événement Tauri émis | Listener React | postMessage iframe | Action finale |
|------------------|----------------------|----------------|-------------------|---------------|
| `Ctrl+Shift+D` | `airadcr:dictation_startstop` | `useSecureMessaging.ts` | `airadcr:toggle_recording` | Start/Stop |
| `Ctrl+Shift+P` | `airadcr:dictation_pause` | `useSecureMessaging.ts` | `airadcr:toggle_pause` | Pause/Reprise |
| `Ctrl+Shift+T` | `airadcr:inject_raw` | `useSecureMessaging.ts` | `airadcr:request_injection` (brut) | Injecter brut |
| `Ctrl+Shift+S` | `airadcr:inject_structured` | `useSecureMessaging.ts` | `airadcr:request_injection` (structuré) | Injecter structuré |
| `Ctrl+Alt+D` | `airadcr:toggle_debug` | `App.tsx` | - | Toggle debug |
| `Ctrl+Alt+L` | `airadcr:toggle_logs` | `App.tsx` | - | Afficher logs |
| `Ctrl+Alt+I` | `airadcr:test_injection` | `App.tsx` | - | Test injection |
| `F9` | `airadcr:force_clickable` | - | - | Anti-ghost |

---

## 🔐 Sécurité

### Communication postMessage sécurisée

L'iframe `airadcr.com` communique avec Tauri uniquement via `postMessage()` avec vérification d'origine :

```typescript
// src/hooks/useSecureMessaging.ts
const ALLOWED_ORIGINS = [
  'https://airadcr.com',
  'https://www.airadcr.com',
];

const handleSecureMessage = (event: MessageEvent) => {
  if (!ALLOWED_ORIGINS.includes(event.origin)) {
    console.warn('❌ Message ignoré - origine non autorisée:', event.origin);
    return;
  }
  // ... traitement sécurisé
};
```

### Permissions Tauri minimales

Configuration dans `tauri.conf.json`:
```json
{
  "allowlist": {
    "window": {
      "all": false,
      "setAlwaysOnTop": true,
      "setPosition": true
    },
    "clipboard": {
      "all": false,
      "writeText": true,
      "readText": true
    },
    "globalShortcut": {
      "all": true
    }
  }
}
```

---

## 📝 Version History

### Version 3.0 (2025-10-24) - Système unifié
- ✅ **Un seul système de raccourcis:** `Ctrl+Shift+D/P/T/S`
- ✅ **SpeechMike et clavier utilisent les mêmes touches**
- ✅ **Suppression de toutes les couches intermédiaires**
- ❌ Suppression des raccourcis `Ctrl+F*` (legacy)
- ❌ Suppression des raccourcis `F10/F11/F12` directs
- ✅ **Code simplifié:** Moins de listeners, moins de confusion

### Version 2.0 (2025-10-05) - Migration vers Ctrl+F*
- ✅ Nouveaux raccourcis `Ctrl+F9/F10/F11/F12`
- ✅ Capture globale via GlobalShortcut
- ✅ SpeechMike mappé sur Ctrl+F*

### Version 1.0 (2025-09-28) - Fonctionnalités de base
- ✅ Dictée vocale
- ✅ Transcription automatique
- ✅ Injection dans RIS
- ✅ SpeechMike basique

---

**Date de mise à jour:** 2025-10-24  
**Version:** 3.0 - Système unifié Ctrl+Shift+D/P/T/S
