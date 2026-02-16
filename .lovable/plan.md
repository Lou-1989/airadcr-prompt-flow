

# Ajout d'un bouton "Simuler appel RIS" dans le panneau Debug

## Probleme actuel

Les commandes PowerShell posent des problemes d'encodage (UTF-8, caracteres speciaux) et de gestion des cles API. Il est beaucoup plus fiable de tester le pipeline complet directement depuis l'application.

## Solution

Ajouter un bouton **"Simuler appel RIS"** dans l'onglet **TEO Hub** du panneau debug qui execute automatiquement les 3 etapes du pipeline :

1. **POST /pending-report** - Injecte un rapport de test dans le serveur local (SQLite)
2. **POST /open-report** - Declenche la navigation de l'iframe vers le rapport
3. Affiche le resultat de chaque etape dans un log visuel

Le bouton utilisera les commandes Tauri existantes (`invoke`) pour appeler directement les fonctions Rust, sans passer par HTTP, ce qui elimine les problemes d'authentification et d'encodage.

## Donnees de test utilisees

Les 3 jeux de donnees de la documentation TEO Hub seront proposes :

| Patient ID | Study UID | Description |
|-----------|-----------|-------------|
| 11601 | 5.1.600...945557.148 | Mammographie BI-RADS 1 bilateral |
| 11600 | 5.1.600...33160890.148 | Mammographie avec masse |
| 006 | 2.16.840...10001221047 | Mammographie foyer tissulaire |

## Modifications techniques

### 1. Fichier `src/components/TeoHubConfig.tsx`

- Ajouter une section "Simulation RIS" avec :
  - Un menu deroulant pour choisir le jeu de donnees patient
  - Un bouton "Simuler appel RIS complet"
  - Un log de resultats affichant chaque etape (succes/echec)
- La simulation appellera directement le serveur local via `fetch("http://localhost:8741/...")` depuis le frontend

### 2. Fichier `src/components/TeoHubConfig.tsx` - Logique de simulation

```text
Etape 1: POST /pending-report (avec X-API-Key depuis la BDD locale)
   -> Stocke le rapport de test dans SQLite
   
Etape 2: POST /open-report?tid=TEST-XXX  
   -> Declenche l'evenement Tauri pour naviguer l'iframe
   
Etape 3: Affiche le resultat final
   -> URL de navigation, statut de chaque etape
```

### 3. Nouvelle commande Tauri (optionnelle, dans `src-tauri/src/main.rs`)

- Ajouter une commande `simulate_ris_call` qui :
  - Insere directement le rapport dans SQLite (sans passer par HTTP)
  - Emet l'evenement `airadcr:navigate_to_report`
  - Retourne le resultat complet

Cette approche est plus fiable car elle contourne les problemes d'authentification HTTP.

### 4. Alternative plus simple (approche recommandee)

Plutot que de passer par HTTP (qui necessite une cle API), utiliser `invoke` pour appeler une commande Tauri dediee :

```text
invoke('simulate_ris_call', { 
  patientId: '11601',
  studyUid: '5.1.600.598386.1.618.1.905951.1.945557.148'
})
```

Cette commande :
- Insere le rapport directement en SQLite
- Emet l'evenement de navigation
- Retourne le resultat sans authentification HTTP

## Resultat attendu

Depuis l'onglet TEO Hub du panneau debug (Ctrl+Alt+D) :
1. Choisir un patient de test dans le menu deroulant
2. Cliquer sur "Simuler appel RIS"
3. Voir le log des etapes s'afficher en temps reel
4. L'iframe AIRADCR navigue automatiquement vers le rapport pre-rempli

