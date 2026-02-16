

# Correction de l'integration TÉO Hub apres changement d'URL

## Probleme identifie

Le changement d'URL vers `?tori=true` casse la construction des URLs de navigation RIS. Actuellement, quand le RIS demande l'ouverture d'un rapport, l'URL generee est **invalide** :

```text
AVANT (correct) :  https://airadcr.com/app?tid=XXX
APRES (casse)   :  https://airadcr.com/app?tori=true?tid=XXX
                                                     ^
                                              Devrait etre &
```

Cela affecte **2 endroits** dans le code :

1. **Frontend** (`WebViewContainer.tsx` ligne 43) : utilise `?tid=` alors que l'URL de base contient deja un `?`
2. **Backend Rust** (`handlers.rs` ligne 353) : utilise une URL en dur **sans** `?tori=true`, donc desynchronisee avec la nouvelle config

## Corrections a appliquer

### 1. WebViewContainer.tsx (ligne 43)

Remplacer la concatenation naive par une construction d'URL correcte qui utilise `&` si l'URL contient deja des parametres :

```typescript
// Avant
const newUrl = `${PRODUCTION_CONFIG.AIRADCR_URL}?tid=${encodeURIComponent(tid)}`;

// Apres
const separator = PRODUCTION_CONFIG.AIRADCR_URL.includes('?') ? '&' : '?';
const newUrl = `${PRODUCTION_CONFIG.AIRADCR_URL}${separator}tid=${encodeURIComponent(tid)}`;
```

### 2. handlers.rs (ligne 353)

Mettre a jour l'URL de retour dans la reponse `POST /pending-report` pour inclure `tori=true` et utiliser `&` :

```rust
// Avant
retrieval_url: format!("https://airadcr.com/app?tid={}", body.technical_id),

// Apres
retrieval_url: format!("https://airadcr.com/app?tori=true&tid={}", body.technical_id),
```

### 3. handlers.rs (ligne 817 - open-report)

Verifier que l'evenement Tauri `airadcr:navigate_to_report` envoie uniquement le `tid` (pas l'URL complete) -- c'est deja le cas, donc pas de changement necessaire ici. La correction du frontend (point 1) suffit.

### 4. openapi.yaml (documentation)

Mettre a jour les exemples d'URL dans la specification OpenAPI pour refleter le nouveau format avec `?tori=true&tid=XXX`.

## Fichiers a modifier

| Fichier | Modification |
|---|---|
| `src/components/WebViewContainer.tsx` | Utiliser `&` au lieu de `?` pour ajouter le parametre `tid` |
| `src-tauri/src/http_server/handlers.rs` | Ajouter `tori=true` dans l'URL de retour |
| `src-tauri/src/http_server/openapi.yaml` | Mettre a jour les exemples d'URL |

## Impact

Sans ces corrections, **toute ouverture de rapport depuis le RIS echouera** car l'URL sera malformee et airadcr.com ne reconnaitra pas le parametre `tid`.

