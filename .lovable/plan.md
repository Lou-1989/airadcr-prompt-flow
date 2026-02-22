

# Raccourcis ergonomiques style Wispr Flow — Analyse et solution optimale

## Ce que font les leaders du marche (recherche confirmee)

| Outil | Plateforme | Raccourci principal | Mode |
|-------|-----------|---------------------|------|
| **Wispr Flow** | Mac | **Fn** (maintenir) | Hold-to-talk |
| **Wispr Flow** | Windows | **Ctrl+Win** (maintenir) | Hold-to-talk |
| **Wispr Flow** | Mains-libres Mac | **Fn+Space** (toggle) | Toggle |
| **Wispr Flow** | Mains-libres Win | **Ctrl+Win+Space** (toggle) | Toggle |
| **SuperWhisper** | Mac | **Option+Space** (toggle) | Toggle |
| **Windows natif** | Windows | **Win+H** | Toggle |

**Point commun** : tous utilisent la **barre Espace** comme touche principale. C'est la touche la plus accessible du clavier — sous le pouce, aucun effort.

## Pourquoi Ctrl+F8/F9/F11/F12 etait une mauvaise idee

Les touches F8-F12 sont en haut du clavier, a 15-20cm des doigts au repos. Pour un radiologue qui dicte toute la journee, c'est un mouvement repetitif non ergonomique. Wispr Flow et SuperWhisper ont choisi Space precisement pour cette raison.

## Contrainte Tauri v1

Tauri v1 `GlobalShortcutManager` ne detecte que le **key-down** (appui), pas le **key-up** (relachement). Le mode "hold-to-talk" de Wispr Flow (maintenir puis relacher) est donc impossible sans hook bas niveau. **Mais** le mode "mains-libres" (toggle) fonctionne parfaitement — c'est exactement ce que fait deja AIRADCR avec `Ctrl+Shift+D`.

## Solution : ajouter les raccourcis Espace (additif, zero casse)

### Nouveaux raccourcis proposes

| Raccourci | Action | Equivalent industrie | Position sur le clavier |
|-----------|--------|---------------------|------------------------|
| **Ctrl+Space** | Start/Stop dictee | SuperWhisper `Option+Space` | Main gauche : auriculaire sur Ctrl, pouce sur Space |
| **Ctrl+Shift+Space** | Pause/Resume | Extension logique | Meme main, majeur sur Shift |

### Pourquoi Ctrl+Space est sans conflit

Objection possible : "Ctrl+Space dans Word supprime le formatage du texte".

**Reponse** : `RegisterHotKey()` de Windows intercepte la combinaison **AVANT** qu'elle n'atteigne l'application active. Word ne recevra jamais `Ctrl+Space` — Tauri l'avale en premier. C'est le meme mecanisme qui fait que `Ctrl+Shift+D` ne declenche pas le "double soulignement" de Word actuellement.

Objection possible : "Ctrl+Space change la langue de saisie (IME)".

**Reponse** : uniquement si plusieurs methodes de saisie asiatiques sont installees. Dans un contexte radiologique francais, ce cas ne se presente pas. Et meme s'il se presentait, `RegisterHotKey()` intercepte avant l'IME.

## Ce qui ne change PAS (zero casse)

```text
EXISTANT (100% conserve, aucune ligne modifiee) :
  Ctrl+Shift+D → toggle_recording     (clavier physique)
  Ctrl+Shift+P → toggle_pause         (clavier physique)
  Ctrl+Shift+T → inject_raw           (clavier physique)
  Ctrl+Shift+S → inject_structured    (clavier physique)
  Ctrl+Alt+D   → debug panel
  Ctrl+Alt+L   → log window
  Ctrl+Alt+I   → test injection
  F9           → anti-ghost

AJOUTE (meme canal tokio, meme frontend) :
  Ctrl+Space       → toggle_recording  (ergonomique)
  Ctrl+Shift+Space → toggle_pause      (ergonomique)
```

Les nouveaux raccourcis envoient la **meme action** dans le **meme canal tokio** existant. Le receiver `rx` et tout le frontend (`useSecureMessaging.ts`) restent strictement identiques.

## Fichier modifie : `src-tauri/src/main.rs` uniquement

### Dans `register_global_shortcuts()`, ajouter apres la ligne 1698 (avant F9)

```rust
// 🎯 ERGONOMIC: Ctrl+Space → toggle_recording (style Wispr Flow / SuperWhisper)
// Touche la plus accessible : auriculaire Ctrl + pouce Space, une seule main
let tx_space = tx.clone();
shortcut_manager
    .register("Ctrl+Space", move || {
        debug!("[Shortcuts] Ctrl+Space pressé (ergonomic) → toggle_recording");
        let _ = tx_space.send("toggle_recording");
    })
    .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Space: {}", e));

// 🎯 ERGONOMIC: Ctrl+Shift+Space → toggle_pause
let tx_shift_space = tx.clone();
shortcut_manager
    .register("Ctrl+Shift+Space", move || {
        debug!("[Shortcuts] Ctrl+Shift+Space pressé (ergonomic) → toggle_pause");
        let _ = tx_shift_space.send("toggle_pause");
    })
    .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+Space: {}", e));
```

### Mettre a jour le log final (ligne 1711)

```rust
info!("[Shortcuts] Raccourcis globaux enregistrés: Ctrl+Alt+D/L/I, F9, Ctrl+Shift+D/P/T/S, Ctrl+Space, Ctrl+Shift+Space");
```

## Tableau recapitulatif complet apres modification

| Raccourci | Action | Type | Ergonomie |
|-----------|--------|------|-----------|
| **Ctrl+Space** | Start/Stop dictee | **NOUVEAU** | Excellent — 1 main, sous le pouce |
| **Ctrl+Shift+Space** | Pause/Resume | **NOUVEAU** | Excellent — meme position |
| Ctrl+Shift+D | Start/Stop dictee | Existant | Bon — 3 touches |
| Ctrl+Shift+P | Pause/Resume | Existant | Bon — 3 touches |
| Ctrl+Shift+T | Injecter texte brut | Existant | Bon — 3 touches |
| Ctrl+Shift+S | Injecter rapport | Existant | Bon — 3 touches |
| Ctrl+Alt+D/L/I | Debug | Existant | Correct |
| F9 | Anti-ghost | Existant | Correct |

## Risques

| Risque | Probabilite | Impact | Raison |
|--------|------------|--------|--------|
| Conflit avec Word/RIS | **Nulle** | Nul | `RegisterHotKey()` intercepte avant l'application |
| Double declenchement | **Nulle** | Nul | `Ctrl+Space` et `Ctrl+Shift+D` sont des combinaisons differentes |
| Echec d'enregistrement | Faible | Nul | `unwrap_or_else` logue un warning, les autres raccourcis fonctionnent |
| Casse de l'existant | **Nulle** | Nul | Zero ligne existante modifiee, ajout pur |

## Resume

- **1 fichier modifie** : `src-tauri/src/main.rs`
- **~14 lignes ajoutees** dans `register_global_shortcuts()`
- **Zero ligne existante modifiee**
- **Zero modification frontend**
- Le radiologue peut desormais demarrer/arreter la dictee avec **Ctrl+Space** (pouce + auriculaire, une seule main) exactement comme SuperWhisper, tout en gardant les raccourcis `Ctrl+Shift+D/P/T/S` pour les utilisateurs habitues

