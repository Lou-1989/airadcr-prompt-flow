# ⌨️ Référence complète - Raccourcis clavier AIRADCR Desktop

**Version:** 2.0  
**Date:** 2025-10-05  
**Architecture:** Rust (Tauri GlobalShortcut) + React (Listeners) + iframe airadcr.com

---

## 🎯 Vue d'ensemble de l'architecture

```
┌─────────────────────────────────────────────────────────┐
│  SpeechMike / Clavier physique                          │
│  (Ctrl+F9, Ctrl+F10, Ctrl+F11, Ctrl+F12)               │
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
│  App.tsx React Listeners                                │
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

---

## 🎤 Raccourcis de contrôle de dictée

### 📋 Raccourcis recommandés (avec Ctrl)

Ces raccourcis sont les **nouveaux standards** pour l'utilisation avec le SpeechMike et fonctionnent **globalement** (même quand AirADCR n'a pas le focus).

### **Ctrl+F9** - Pause/Reprise dictée (Toggle)

**Fonction:** Met en pause ou reprend l'enregistrement audio selon l'état actuel.

**États et actions:**

| État actuel | Action déclenchée | Nouvel état | Description UI |
|------------|-------------------|-------------|----------------|
| **Recording** | Met en pause | **Paused** | Bouton "⏸️ En pause" |
| **Paused** | Reprend | **Recording** | Bouton "⏹️ Arrêter" |
| **Idle** | Aucune action | **Idle** | Pas de changement |

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+F9` globalement, émet `airadcr:dictation_pause_toggle`
- **React (App.tsx):** Écoute `airadcr:dictation_pause_toggle`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handlePauseResumeToggle()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~480-490)
- `src/App.tsx` (lignes ~108-112)
- `src/components/dictation-recorder/DictationInput.tsx` (airadcr.com)
- `src/components/widget/WidgetInterface.tsx` (airadcr.com)

**Comportement SpeechMike:**
- Bouton **Pause** (avec modificateur Ctrl) → Envoie `Ctrl+F9`

---

### **Ctrl+F10** - Démarrer/Terminer dictée (Toggle intelligent)

**Fonction:** Démarre ou termine l'enregistrement audio selon l'état actuel.

**États et actions:**

| État actuel | Action déclenchée | Nouvel état | Description UI |
|------------|-------------------|-------------|----------------|
| **Idle** | Démarre l'enregistrement | **Recording** | Bouton "🎤 Enregistrer" → "⏹️ Arrêter" |
| **Recording** | Termine la dictée | **Idle** | Bouton "⏹️ Arrêter" → Finalisation + transcription |
| **Paused** | Termine la dictée | **Idle** | Bouton "▶️ Continuer" → Finalisation + transcription |

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+F10` globalement, émet `airadcr:dictation_startstop_toggle`
- **React (App.tsx):** Écoute `airadcr:dictation_startstop_toggle`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleStartStopToggle()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~495-505)
- `src/App.tsx` (lignes ~114-118)
- `src/components/dictation-recorder/DictationInput.tsx` (airadcr.com)
- `src/components/widget/WidgetInterface.tsx` (airadcr.com)

**Comportement SpeechMike:**
- Bouton **Record** (avec modificateur Ctrl) → Envoie `Ctrl+F10`

---

## 📋 Raccourcis d'injection de texte

### **Ctrl+F11** - Injecter le texte dicté brut

**Fonction:** Injecte le texte de dictée brut (non structuré) dans l'application cible externe (RIS, Word, etc.).

**Conditions requises:**
- ✅ Le texte de dictée (`dictationText`) ne doit **pas** être vide
- ✅ La position d'injection doit être verrouillée (optionnel)

**Format d'injection:**
```
=== [Titre du rapport] ===

INDICATION:
[Texte d'indication]

TECHNIQUE:
[Texte de technique]

RÉSULTATS:
[Texte des résultats bruts]
```

**Implémentation:**
- **Rust (Tauri):** Enregistre `Ctrl+F11` globalement, émet `airadcr:inject_raw_text`
- **React (App.tsx):** Écoute `airadcr:inject_raw_text`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleInjectRawText()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~510-520)
- `src/App.tsx` (lignes ~120-124)
- `src/components/dictation-recorder/DictationInput.tsx` (page principale uniquement)

**Comportement SpeechMike:**
- Bouton **Instruction** → Envoie `Ctrl+F11`

---

### **Ctrl+F12** - Injecter le rapport structuré complet

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
- **Rust (Tauri):** Enregistre `Ctrl+F12` globalement, émet `airadcr:inject_structured_report`
- **React (App.tsx):** Écoute `airadcr:inject_structured_report`, envoie `postMessage` à l'iframe
- **Web (airadcr.com):** Écoute `postMessage`, appelle `handleInjectStructuredReport()`

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~525-535)
- `src/App.tsx` (lignes ~126-130)
- `src/components/ExportPanel.tsx` (rapport structuré final)

**Comportement SpeechMike:**
- Bouton **Programmable 1** → Envoie `Ctrl+F12`

---

## 🔄 Raccourcis de compatibilité (Legacy)

Ces raccourcis **sans Ctrl** sont conservés pour la rétrocompatibilité avec les anciens profils SpeechMike, mais les nouveaux raccourcis **avec Ctrl** (ci-dessus) sont recommandés.

| Touche | Action | Disponible | Note |
|--------|--------|------------|------|
| `F10` | Enregistrer / Reprendre dictée (Legacy) | ✅ Global | ⚠️ Utilisez `Ctrl+F10` de préférence |
| `F11` | Mettre en pause dictée (Legacy) | ✅ Global | ⚠️ Utilisez `Ctrl+F9` de préférence |
| `F12` | Finaliser et injecter (Production uniquement - Legacy) | ✅ Global | ⚠️ Utilisez `Ctrl+F12` de préférence |

---

## 🐛 Raccourcis de débogage (développement)

### **Ctrl+Shift+D** - Toggle panneau de débogage

**Fonction:** Affiche/masque le panneau de débogage avec informations système.

**Informations affichées:**
- État de l'application Tauri
- Position Always-on-Top
- Informations système (OS, architecture)
- État de verrouillage de la cible d'injection

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~450-460)
- `src/App.tsx` (lignes ~95-99)
- `src/components/DebugPanel.tsx`

---

### **Ctrl+Shift+L** - Afficher fenêtre de logs

**Fonction:** Ouvre une fenêtre popup affichant les logs en temps réel de l'application.

**Logs affichés:**
- Messages Tauri (Rust)
- Messages React (TypeScript)
- Communications postMessage
- Erreurs et warnings

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~465-475)
- `src/App.tsx` (lignes ~101-105)
- `src/components/DevLogWindow.tsx`

---

### **Ctrl+Shift+T** - Test d'injection

**Fonction:** Déclenche un test d'injection à la position actuelle du curseur.

**Comportement:**
- Injecte un texte de test : `"🧪 TEST INJECTION - AIRADCR Desktop"`
- Utilise la position verrouillée si disponible
- Sinon, utilise la position actuelle du curseur

**Fichiers concernés:**
- `src-tauri/src/main.rs` (lignes ~478-488)
- `src/App.tsx` (lignes ~107)

---

## 🎛️ Mapping SpeechMike (recommandé)

### Profil SpeechControl XML recommandé

Pour mapper le SpeechMike sur les nouveaux raccourcis `Ctrl+F*`, voici la configuration recommandée :

| Bouton physique SpeechMike | Modificateur | Touche envoyée | Action AIRADCR |
|----------------------------|-------------|----------------|----------------|
| 🔴 **Record** | Ctrl | `Ctrl+F10` | Démarrer/Terminer dictée |
| ⏸️ **Pause** | Ctrl | `Ctrl+F9` | Pause/Reprise toggle |
| 📋 **Instruction** | Aucun | `Ctrl+F11` | Injecter texte brut |
| 🔧 **Programmable 1** | Aucun | `Ctrl+F12` | Injecter rapport structuré |

**Fichier XML à utiliser:** `airadcr_speechmike_ctrlf_profile.xml`

---

## 🔄 Workflow complet (exemple d'utilisation)

### Scénario: Radiologue dictant un scanner thoracique

```
1. 📍 POSITION INITIALE
   └─ Utilisateur ouvre RIS et clique dans le champ "Compte rendu"
   
2. 🎤 DÉMARRAGE DICTÉE (Ctrl+F10)
   ├─ Rust: Capture Ctrl+F10 globalement
   ├─ React: Envoie postMessage à l'iframe
   └─ iframe: Démarre enregistrement audio
       └─ Toast: "🎤 Dictée démarrée"
   
3. 🗣️ DICTÉE EN COURS
   └─ "Scanner thoracique. Indication pneumonie. Technique spiralée..."
   
4. ⏸️ PAUSE (Ctrl+F9)
   ├─ Rust: Capture Ctrl+F9 globalement
   ├─ React: Envoie postMessage à l'iframe
   └─ iframe: Met en pause (MediaRecorder.pause())
       └─ Toast: "⏸️ Dictée en pause"
   
5. ▶️ REPRISE (Ctrl+F9)
   ├─ Rust: Capture Ctrl+F9 globalement (même touche)
   └─ iframe: Reprend enregistrement
       └─ Toast: "▶️ Dictée reprise"
   
6. 🗣️ SUITE DE LA DICTÉE
   └─ "...infiltrat lobe inférieur droit. Conclusion infection confirmée."
   
7. ✅ TERMINER (Ctrl+F10)
   ├─ Rust: Capture Ctrl+F10 globalement (même touche qu'au démarrage)
   ├─ iframe: Fusionne chunks audio
   ├─ Envoi à Voxtral API pour transcription
   ├─ Structuration automatique (GPT-4)
   └─ Toast: "✅ Transcription complète"
   
8. 📋 INJECTION RAPPORT STRUCTURÉ (Ctrl+F12)
   ├─ Rust: Capture Ctrl+F12 globalement
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
2. Appuyer sur `Ctrl+F10` → Vérifier démarrage dictée ✅
3. Dicter 5 secondes de test
4. Appuyer sur `Ctrl+F9` → Vérifier mise en pause ✅
5. Appuyer sur `Ctrl+F9` → Vérifier reprise ✅
6. Appuyer sur `Ctrl+F10` → Vérifier finalisation ✅
7. Attendre transcription (5-10s)
8. Appuyer sur `Ctrl+F12` → Vérifier injection dans RIS ✅

**Résultat attendu:** Toutes les étapes fonctionnent correctement.

---

### Test 2: Capture globale (hors focus)

**Procédure:**
1. Lancer AIRADCR Desktop
2. Ouvrir Word ou RIS
3. Donner le focus à Word/RIS (cliquer dedans)
4. Appuyer sur `Ctrl+F10` → Vérifier que l'enregistrement démarre dans AIRADCR ✅

**Résultat attendu:** AIRADCR réagit même sans avoir le focus (GlobalShortcut).

---

### Test 3: SpeechMike mappé sur Ctrl+F*

**Prérequis:** Profil `airadcr_speechmike_ctrlf_profile.xml` installé dans SpeechControl.

**Procédure:**
1. Brancher le SpeechMike Philips
2. Appuyer sur le bouton **Record** du SpeechMike → Doit envoyer `Ctrl+F10` ✅
3. Vérifier que la dictée démarre dans AIRADCR ✅
4. Appuyer sur le bouton **Pause** → Doit envoyer `Ctrl+F9` ✅
5. Appuyer sur le bouton **Instruction** → Doit envoyer `Ctrl+F11` ✅
6. Appuyer sur le bouton **Programmable 1** → Doit envoyer `Ctrl+F12` ✅

**Résultat attendu:** Le SpeechMike contrôle AIRADCR via les nouveaux raccourcis.

---

## 📊 Résumé des événements Tauri

| Raccourci clavier | Événement Tauri émis | Listener React | postMessage iframe | Action finale |
|------------------|----------------------|----------------|-------------------|---------------|
| `Ctrl+F9` | `airadcr:dictation_pause_toggle` | `App.tsx` ligne ~108 | `airadcr:dictation_pause_toggle` | Pause/Reprise |
| `Ctrl+F10` | `airadcr:dictation_startstop_toggle` | `App.tsx` ligne ~114 | `airadcr:dictation_startstop_toggle` | Start/Stop |
| `Ctrl+F11` | `airadcr:inject_raw_text` | `App.tsx` ligne ~120 | `airadcr:inject_raw_text` | Injecter brut |
| `Ctrl+F12` | `airadcr:inject_structured_report` | `App.tsx` ligne ~126 | `airadcr:inject_structured_report` | Injecter structuré |
| `Ctrl+Shift+D` | `airadcr:toggle_debug` | `App.tsx` ligne ~95 | - | Toggle debug |
| `Ctrl+Shift+L` | `airadcr:show_logs` | `App.tsx` ligne ~101 | - | Afficher logs |
| `Ctrl+Shift+T` | `airadcr:test_injection` | `App.tsx` ligne ~107 | - | Test injection |

---

## 🔐 Sécurité

### Communication postMessage sécurisée

L'iframe `airadcr.com` communique avec Tauri uniquement via `postMessage()` avec vérification d'origine :

```typescript
// src/hooks/useSecureMessaging.ts
const ALLOWED_ORIGINS = [
  'https://airadcr.com',
  'https://www.airadcr.com',
  'http://localhost:5173' // Dev uniquement
];

window.addEventListener('message', (event) => {
  if (!ALLOWED_ORIGINS.includes(event.origin)) {
    console.error('❌ Origine non autorisée:', event.origin);
    return;
  }
  // Traiter le message sécurisé...
});
```

### Permissions Tauri minimales

```json
// src-tauri/tauri.conf.json
"allowlist": {
  "globalShortcut": { "all": true },    // Raccourcis globaux uniquement
  "clipboard": { "all": true },         // Injection texte
  "window": { "all": true },            // Always on top
  "shell": { "open": false },           // ❌ Pas de shell
  "fs": { "all": false }                // ❌ Pas d'accès fichiers
}
```

---

## 📝 Historique des versions

### Version 2.0 (2025-10-05) - Migration Ctrl+F9-F12
- ✅ Ajout de `Ctrl+F9` (Pause/Reprise toggle)
- ✅ Ajout de `Ctrl+F10` (Start/Stop toggle)
- ✅ Ajout de `Ctrl+F11` (Injection texte brut)
- ✅ Ajout de `Ctrl+F12` (Injection rapport structuré)
- ✅ Architecture hybride Rust + React + iframe
- ✅ Support SpeechMike via profil `airadcr_speechmike_ctrlf_profile.xml`
- ✅ Documentation complète des workflows

### Version 1.0 (2025-10-04) - Raccourcis basiques
- ✅ Support F10/F11/F12 basique (SpeechMike legacy)
- ✅ Architecture Tauri GlobalShortcut
- ✅ Communication postMessage iframe

---

**FIN DE LA RÉFÉRENCE COMPLÈTE**

*Dernière mise à jour : 2025-10-05*  
*Version : 2.0*  
*Équipe AIRADCR*
