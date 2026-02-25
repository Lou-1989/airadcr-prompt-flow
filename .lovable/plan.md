

## Ouvrir le serveur HTTP au reseau local

### Pourquoi c'est necessaire
Le serveur HTTP ecoute actuellement sur `127.0.0.1` (localhost uniquement). Orthanc/TEO tourne sur une machine differente du reseau, donc il ne peut pas atteindre le port 8741. Il faut changer le binding vers `0.0.0.0` pour accepter les connexions reseau.

### Impact
- **Windows** : aucun changement de comportement visible, le serveur acceptera simplement aussi les connexions reseau
- **Mac** : idem, transparent
- **Securite** : le rate limiting et l'authentification par cle API restent en place

### Modifications

**Fichier 1 : `src-tauri/src/config.rs`**
- Ajouter un champ `http_bind_address` avec valeur par defaut `"0.0.0.0"`
- Permet de reconfigurer via `config.toml` si besoin de restreindre plus tard

**Fichier 2 : `src-tauri/src/http_server/mod.rs`**
- Modifier la signature de `start_server` pour accepter un parametre `bind_address`
- Remplacer le `"127.0.0.1"` en dur par cette valeur dynamique

**Fichier 3 : `src-tauri/src/main.rs`**
- Passer `config.http_bind_address` lors de l'appel a `start_server`

### Apres deploiement
Dominique pourra tester avec :
```text
curl http://[IP-DU-POSTE]:8741/health
```

Note : le pare-feu Windows devra autoriser le port 8741 en entrant (commande PowerShell fournie dans le guide precedent).

