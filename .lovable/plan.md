

# Rechargement libre du meme rapport + Nettoyage pipeline post-injection

## Analyse de l'architecture existante

Apres audit complet du code, voici les constats :

### Navigation (WebViewContainer.tsx)
Le bloc anti-doublon (lignes 78-83) empeche le rechargement si le meme `tid` est deja dans l'URL. Le radiologue ne peut pas relancer un rapport deja ouvert -- c'est trop restrictif.

### Pipeline post-injection (useSecureMessaging.ts)
Apres une injection reussie, le hook envoie un `airadcr:injection_status` avec `success: true` a l'iframe, mais **ne nettoie jamais le rapport en SQLite**. Il reste avec le status `retrieved` pendant 24h. Cela signifie que le pipeline n'est pas pret a recommencer immediatement.

### Bonne nouvelle : la commande Tauri existe deja
`delete_pending_report_cmd` est deja enregistree dans `main.rs` (ligne 1739) et fonctionne. Pas besoin de creer une nouvelle commande Rust.

---

## Modifications prevues

### 1. WebViewContainer.tsx -- Autoriser le rechargement du meme rapport

Supprimer le bloc anti-doublon (lignes 78-83). A la place, toujours recharger avec un cache-buster et re-afficher le splash screen pour masquer la transition.

```text
Avant :
  tid identique --> SKIP (bloque le rechargement)

Apres :
  tid identique --> splash screen + rechargement avec cache-buster
  tid different --> splash screen + rechargement normal
```

Le `extractTid` reste utile pour le logging.

### 2. useSecureMessaging.ts -- Nettoyage SQLite apres injection reussie

Dans `processNextInjection`, apres reception de `success: true`, appeler la commande Tauri existante `delete_pending_report_cmd` pour supprimer le rapport de SQLite. Cela libere le pipeline immediatement.

Le nettoyage est "fire and forget" : si l'appel echoue (pas de Tauri, erreur DB), le cleanup automatique (toutes les 10 minutes) prend le relais. Aucun risque de blocage.

Pour extraire le `tid` du rapport injecte, on utilise l'URL courante de l'iframe. Alternativement, on peut stocker le `tid` dans le payload d'injection si disponible.

---

## Pipeline corrige

```text
1. TEO Hub POST /pending-report --> SQLite (status="pending")
2. RIS POST /open-report --> evenement Tauri --> iframe charge le rapport
3. GET /pending-report --> donnees retournees, status="retrieved"
4. Radiologue dicte/valide --> iframe envoie postMessage "airadcr:inject"
5. Injection dans le RIS via clipboard
6. Succes --> DELETE rapport de SQLite (via delete_pending_report_cmd)
7. Pipeline pret pour nouveau cycle immediatement
```

---

## Risques et protections

| Risque | Protection |
|---|---|
| Double-clic RIS sur le meme rapport | Rechargement propre avec splash, pas de crash |
| Injection echouee | Rapport conserve en SQLite, pas de suppression |
| Tauri absent (mode web) | invoke() echoue silencieusement, cleanup auto prend le relais |
| Rapport supprime trop tot | Suppression UNIQUEMENT apres `success: true` confirme |

## Fichiers modifies

| Fichier | Modification |
|---|---|
| `src/components/WebViewContainer.tsx` | Suppression du bloc anti-doublon tid, rechargement toujours autorise avec cache-buster |
| `src/hooks/useSecureMessaging.ts` | Appel `delete_pending_report_cmd` apres injection reussie |

## Fichiers NON modifies

| Fichier | Raison |
|---|---|
| `src-tauri/src/main.rs` | `delete_pending_report_cmd` existe deja |
| `src-tauri/src/http_server/handlers.rs` | `unminimize()` deja corrige |
| `src-tauri/src/database/queries.rs` | `delete_pending_report` fonctionne deja |

