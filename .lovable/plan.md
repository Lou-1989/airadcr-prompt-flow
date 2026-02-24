

# Audit fonctionnel -- Est-ce que le SpeechMike va marcher en production ?

## Verdict : CA MARCHERA PARTIELLEMENT -- 3 problemes bloquants a corriger

L'implementation HID est techniquement solide (detection, boutons, polling). Mais il y a des trous dans la chaine fonctionnelle entre le hardware et l'application. Voici ce qui va marcher, ce qui ne va pas marcher, et ce qu'il faut corriger.

---

## CE QUI VA MARCHER (sans modification)

| Fonctionnalite | Status | Detail |
|---|---|---|
| Detection auto du SpeechMike au branchement USB | OK | Thread de polling avec retry 3s/5s/10s, hotplug |
| Appui bouton Record -> demarre/arrete la dictee | OK | Record -> `toggle_recording` -> event Tauri -> postMessage vers iframe |
| Appui bouton Stop/Play -> pause/resume | OK | Stop/Play -> `toggle_pause` -> meme pipeline |
| Appui bouton Instruction -> injection texte brut | OK | Instr -> `inject_raw` -> demande injection a l'iframe |
| Appui bouton F1 -> injection rapport structure | OK | F1A -> `inject_structured` -> idem |
| Appui bouton EOL/Prio -> finaliser et injecter | OK | EolPrio -> `inject_structured` |
| Reconnexion auto si debranche/rebranche | OK | Boucle externe avec retry |
| Multi-modeles (LFH35xx, SMP37xx, PowerMic IV) | OK | 8 filtres VID/PID + usage_page |
| Slider filtering (modeles avec switch) | OK | Masque bits 5-6 applique |
| Switch auto mode Keyboard -> HID | OK | Detection + `set_event_mode_hid()` |
| Coexistence avec raccourcis clavier | OK | Meme canal `tokio::mpsc` |

---

## CE QUI NE VA PAS MARCHER (bugs a corriger)

### BUG 1 -- CRITIQUE : Les LEDs ne changent JAMAIS automatiquement

**Symptome** : L'utilisateur appuie sur Record, la dictee demarre dans l'iframe, mais la LED du SpeechMike reste eteinte/verte. Aucun feedback visuel.

**Cause racine** : La fonction `notifyRecordingState()` est definie et exportee dans `useSecureMessaging.ts` (ligne 33-45), mais **elle n'est jamais appelee**. Personne ne la consomme.

Le flux prevu etait :
```text
Iframe airadcr.com envoie postMessage "airadcr:recording_started"
  -> handleSecureMessage() devrait le capter
  -> appeler notifyRecordingState('started')
  -> invoke('speechmike_set_led', { ledState: 'recording' })
  -> LED rouge fixe
```

Mais dans `handleSecureMessage()` (lignes 123-261), il n'y a **aucun `case` pour** :
- `airadcr:recording_started`
- `airadcr:recording_paused`
- `airadcr:recording_finished`

Ces types sont declares dans `SecurityConfig.ts` (lignes 67-69) comme autorises, mais le switch/case ne les traite pas.

**Correction** : Ajouter 3 cases dans `handleSecureMessage` pour appeler `notifyRecordingState`.

---

### BUG 2 -- MOYEN : Conflit HID double-open pour les LEDs

**Symptome** : Sur certains modeles Windows, `speechmike_set_led` echoue silencieusement avec "Cannot open device by path". La LED ne change pas meme si le code est corrige (Bug 1).

**Cause** : `speechmike_set_led` (main.rs ligne 1074) cree un **nouveau** `HidApi` et re-ouvre le device par path, alors que le polling thread a deja le device ouvert. Sur Windows, `hid.dll` autorise generalement les ouvertures multiples, mais :
- Certains firmwares SpeechMike refusent le double-open
- Il y a une latence de ~5ms par ouverture/fermeture inutile
- Risque de race condition avec le `read_timeout(10ms)` du polling

**Correction** : Faire envoyer la commande LED **depuis le polling thread** via un deuxieme canal (ex: `tokio::sync::watch`), au lieu de re-ouvrir le device.

---

### BUG 3 -- MINEUR : `notifyRecordingState` dans les deps du `handleSecureMessage` useCallback

**Symptome** : `notifyRecordingState` est liste dans les dependances du `useCallback` de `handleSecureMessage` (ligne 262) mais n'est pas utilise dans le corps de la fonction. Ce n'est pas un bug fonctionnel mais c'est un indicateur que l'integration n'a pas ete terminee.

---

## ANALYSE DE LA CHAINE COMPLETE (bout en bout)

```text
SpeechMike (Hardware)
  |
  | [HID USB Report 0x80, bitmask uint16 LE offset 7-8]
  v
speechmike/mod.rs (Thread Rust, polling 10ms)
  |
  | decode_input_report() -> button_to_action() -> action_to_channel_str()
  | tx.send("toggle_recording")
  v
tokio::mpsc channel (main.rs ligne 1773)
  |
  | rx.recv() -> window.emit("airadcr:dictation_startstop")
  v
useSecureMessaging.ts (Frontend React)
  |
  | listen('airadcr:dictation_startstop') -> sendSecureMessage('airadcr:toggle_recording')
  v
iframe airadcr.com (postMessage)
  |
  | [L'iframe demarre l'enregistrement vocal]
  | [L'iframe envoie postMessage "airadcr:recording_started"]  <-- EXISTE
  v
handleSecureMessage() (useSecureMessaging.ts)
  |
  | case 'airadcr:recording_started': ???  <-- MANQUANT !!!
  | notifyRecordingState('started')        <-- JAMAIS APPELE
  | invoke('speechmike_set_led')           <-- JAMAIS ATTEINT
  v
LED SpeechMike : RESTE ETEINTE  <-- BUG
```

---

## PLAN DE CORRECTION (3 modifications)

### Correction 1 -- Brancher les LEDs (useSecureMessaging.ts)

Ajouter dans le `switch(type)` de `handleSecureMessage` :

```text
case 'airadcr:recording_started':
  notifyRecordingState('started');   // -> LED rouge fixe
  break;

case 'airadcr:recording_paused':
  notifyRecordingState('paused');    // -> LED rouge clignotant
  break;

case 'airadcr:recording_finished':
  notifyRecordingState('finished');  // -> LED vert fixe
  break;
```

### Correction 2 -- Eliminer le double-open HID (main.rs + mod.rs)

Remplacer l'approche actuelle (re-ouvrir le device dans `speechmike_set_led`) par un canal de commande :
- Ajouter un `tokio::sync::mpsc::UnboundedSender<SimpleLedState>` dans `SpeechMikeState`
- Le polling thread ecoute ce canal et ecrit les LED sur le device qu'il a deja ouvert
- `speechmike_set_led` envoie simplement via le canal au lieu de toucher au HID

### Correction 3 -- Nettoyage deps useCallback

Retirer `notifyRecordingState` des deps du useCallback de `handleSecureMessage` une fois qu'il est effectivement utilise dans le corps (il y sera apres Correction 1, donc ce point se resout tout seul).

---

## ESTIMATION FINALE DE REUSSITE

| Scenario | AVANT corrections | APRES corrections |
|---|---|---|
| Boutons Record/Stop/Play/F1 declenchent les actions | 95% | 95% |
| LEDs changent selon l'etat de dictee | 0% (jamais appele) | 90% |
| LED fonctionne sur tous les modeles Windows | 0% | 85% (double-open corrige) |
| Chaine complete bout en bout | 60% | 90% |

**Conclusion** : Les boutons physiques du SpeechMike fonctionneront correctement pour declencher les actions (dictee, pause, injection). Mais les LEDs -- le feedback visuel essentiel pour le radiologue -- ne s'allumeront jamais dans l'etat actuel du code. Il faut imperativement brancher les 3 cases `recording_started/paused/finished` dans le message handler et corriger le double-open HID.

