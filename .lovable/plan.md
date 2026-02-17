

# Correction des raccourcis clavier et du systeme d'injection

## Probleme identifie

Les raccourcis clavier (dictee, injection) ne fonctionnent plus car **4 evenements Tauri sont ecoutes en double** dans deux fichiers differents. Chaque raccourci envoie donc 2 messages identiques a l'iframe airadcr.com, ce qui provoque une annulation immediate (ex: Start + Stop = rien).

Ce probleme est **cote application web React uniquement** (ce projet). L'exe Tauri fonctionne correctement.

### Evenements dupliques

| Evenement | App.tsx | useSecureMessaging.ts |
|---|---|---|
| dictation_startstop | Ligne 93 | Ligne 272 |
| dictation_pause | Ligne 99 | Ligne 278 |
| inject_raw | Ligne 105 | Ligne 284 |
| inject_structured | Ligne 111 | Ligne 290 |

## Corrections prevues

### 1. Supprimer les listeners dupliques dans App.tsx

Retirer les 4 listeners de dictee/injection (lignes 92-114) de `App.tsx`. Ces evenements sont deja correctement geres dans `useSecureMessaging.ts` avec en plus :
- La queue FIFO d'injection
- Le debounce et la deduplication
- L'envoi via `sendSecureMessage` (avec validation de securite)

`App.tsx` conservera uniquement les 4 evenements qui lui sont propres :
- `airadcr:toggle_debug` (panneau debug)
- `airadcr:toggle_logs` (fenetre logs)
- `airadcr:test_injection` (test injection)
- `airadcr:force_clickable` (anti-ghost F9)

### 2. Ajouter un bouton "Test communication iframe" dans le panneau debug

Ajouter dans `DebugPanel.tsx` un bouton qui :
1. Envoie un `postMessage` de type `airadcr:request_status` a l'iframe
2. Attend une reponse pendant 3 secondes
3. Affiche "iframe repond" (vert) ou "iframe ne repond pas" (rouge)

Cela permettra de verifier que le site airadcr.com en mode `?tori=true` ecoute bien les messages, sans avoir a ouvrir la console du navigateur.

### 3. Ameliorer les logs dans useSecureMessaging.ts

Ajouter des logs avec timestamp pour chaque `sendSecureMessage` afin de faciliter le diagnostic en cas de probleme futur.

## Fichiers modifies

- `src/App.tsx` : Suppression des 4 listeners dupliques
- `src/components/DebugPanel.tsx` : Ajout du bouton de test communication iframe
- `src/hooks/useSecureMessaging.ts` : Logs ameliores avec timestamps

## Resultat attendu

Apres correction :
- Chaque raccourci clavier n'enverra qu'**un seul** postMessage (au lieu de deux)
- La dictee (Ctrl+Shift+D) demarrera/arretera correctement
- L'injection (Ctrl+Shift+T/S) fonctionnera sans doublon
- Le bouton de test permettra de diagnostiquer si un probleme vient du cote airadcr.com

