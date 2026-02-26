

# Ecran de chargement professionnel + Audit UX + Correction unminimize/doublon

## Contexte

L'application affiche actuellement un ecran blanc pendant le chargement de l'iframe airadcr.com. Il n'y a aucun feedback visuel pour l'utilisateur. De plus, deux bugs de comportement ont ete identifies : la fenetre ne se restaure pas quand elle est minimisee (appel RIS), et un double appel sur le meme CR recharge inutilement l'iframe.

---

## 1. Ecran de chargement professionnel (Splash Screen)

**Fichier** : `src/components/WebViewContainer.tsx`

Actuellement, l'iframe est affichee immediatement mais le contenu met du temps a charger -- l'utilisateur voit un ecran blanc.

**Correction** : Ajouter un etat `isLoading` (true par defaut), afficher un splash screen par-dessus l'iframe tant que `onLoad` n'a pas ete appele.

Le splash screen affichera :
- Le logo AirADCR (`/lovable-uploads/IMG_9255.png`) avec une animation de pulsation douce
- Le texte "AirADCR" en titre
- Le sous-titre "Dictee intelligente pour radiologie"
- Une barre de progression animee (indeterminee) utilisant le composant `Progress` existant
- Un fond blanc propre, coherent avec le design system medical

L'iframe reste montee en arriere-plan (invisible via `opacity-0`) pour que le chargement commence immediatement. Une fois `onLoad` declenche, transition douce en fondu (fade-out du splash, fade-in de l'iframe).

**Justification** : L'iframe doit rester montee pour ne pas retarder le chargement. Le splash masque le blanc sans ralentir le demarrage.

---

## 2. Correction : fenetre non restauree apres minimisation (unminimize manquant)

**Fichier** : `src-tauri/src/http_server/handlers.rs` (lignes 943-945)

Le code actuel dans `open_report` :
```rust
let _ = window.show();
let _ = window.set_focus();
```

Le deep link handler (main.rs lignes 1614-1616) fait correctement :
```rust
let _ = window.show();
let _ = window.unminimize();
let _ = window.set_focus();
```

**Correction** : Ajouter `window.unminimize()` entre `show()` et `set_focus()` dans le handler `open_report`.

**Justification** : Sur Windows, `show()` rend la fenetre visible mais ne la restaure pas depuis la barre des taches. `unminimize()` est necessaire pour que la fenetre retrouve sa taille normale.

---

## 3. Protection anti-doublon : meme CR appele deux fois

**Fichier** : `src/components/WebViewContainer.tsx`

Actuellement, chaque evenement `airadcr:navigate_to_report` met a jour `currentUrl`, meme si le tid est identique. Cela provoque un rechargement complet de l'iframe et une perte potentielle du travail en cours.

**Correction** : Comparer le nouveau `tid` avec le `tid` deja en cours. Si identique, ne pas mettre a jour `currentUrl`. Extraire le tid actuel de l'URL courante pour la comparaison.

```text
Flux actuel :
  RIS appelle POST /open-report?tid=ABC
  --> evenement navigate_to_report(ABC)
  --> setCurrentUrl(url?tid=ABC)   <-- TOUJOURS, meme si deja ABC

Flux corrige :
  RIS appelle POST /open-report?tid=ABC
  --> evenement navigate_to_report(ABC)
  --> si tid actuel == ABC --> SKIP (pas de rechargement)
  --> si tid different --> setCurrentUrl(url?tid=ABC)
```

**Justification** : En milieu hospitalier, un clic accidentel ou un double-clic sur le bouton contextuel ne doit pas faire perdre le travail en cours.

---

## 4. Audit UX rapide -- micro-ameliorations sans alourdir

### 4a. Ecran d'erreur de chargement ameliore

**Fichier** : `src/components/WebViewContainer.tsx`

L'ecran d'erreur actuel a un bouton "Reessayer" qui recharge toute l'iframe. Amelioration : ajouter un compteur de tentatives automatiques (retry automatique apres 5s, 3 tentatives max) avant d'afficher le bouton manuel.

### 4b. Transition fluide apres navigation RIS

**Fichier** : `src/components/WebViewContainer.tsx`

Quand un nouveau tid arrive (navigation RIS), re-afficher brievement le splash screen pendant le chargement de la nouvelle page dans l'iframe. Cela evite le flash blanc entre deux rapports.

---

## Fichiers modifies

| Fichier | Modification |
|---|---|
| `src/components/WebViewContainer.tsx` | Splash screen, anti-doublon tid, transition navigation, retry auto |
| `src-tauri/src/http_server/handlers.rs` | Ajout `window.unminimize()` dans `open_report` |

## Fichiers NON modifies

| Fichier | Raison |
|---|---|
| `src/pages/Index.tsx` | Pas de changement necessaire, wrapper simple |
| `src/index.css` | Les animations CSS peuvent etre inline ou Tailwind, pas besoin d'ajouter au design system |
| `src/App.tsx` | Aucun impact sur la logique applicative |
| `src/components/DebugPanel.tsx` | Deja corrige dans le commit precedent |
| `src-tauri/src/main.rs` | Le deep link handler est deja correct |

