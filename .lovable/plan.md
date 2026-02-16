

# Correction complete du format URL pour le mode portable/exe

## Probleme

Deux causes font que l'exe portable peut encore ouvrir l'ancien lien :

### Cause 1 : config.toml local persistant

Au premier lancement, l'application cree automatiquement un fichier `%APPDATA%/airadcr-desktop/config.toml` avec les valeurs par defaut **du moment**. Si l'exe a ete lance une seule fois avec l'ancienne version, ce fichier contient `iframe_url = "https://airadcr.com/app"` (sans `?tori=true`). Or la logique dans `config.rs` charge ce fichier en priorite sur les valeurs par defaut du code. Meme apres mise a jour de l'exe, l'ancien fichier config est toujours lu.

### Cause 2 : URL manquante dans handlers.rs ligne 830

Le endpoint `POST /open-report` retourne encore l'ancien format dans sa reponse :

```text
navigated_to: "https://airadcr.com/app?tid=XXX"   (manque tori=true)
```

## Corrections

### Fichier 1 : `src-tauri/src/http_server/handlers.rs` (ligne 830)

Corriger l'URL de reponse du endpoint `/open-report` :

```rust
// Avant
navigated_to: Some(format!("https://airadcr.com/app?tid={}", tid)),

// Apres
navigated_to: Some(format!("https://airadcr.com/app?tori=true&tid={}", tid)),
```

### Fichier 2 : `src-tauri/src/config.rs` (methode `load`)

Ajouter une logique de migration automatique : si le fichier `config.toml` existant contient l'ancienne URL (sans `?tori=true`), la mettre a jour automatiquement et sauvegarder. Cela evite que les utilisateurs existants doivent intervenir manuellement.

Concretement, apres le chargement du fichier config, verifier si `iframe_url` est l'ancienne valeur (`https://airadcr.com/app` sans `?tori=true`) et la corriger :

```rust
let mut config = loaded_config;
if config.iframe_url == "https://airadcr.com/app" || config.iframe_url == "https://airadcr.com" {
    config.iframe_url = "https://airadcr.com/app?tori=true".to_string();
    let _ = config.save(); // Sauvegarder la correction
}
```

## Fichiers a modifier

| Fichier | Modification |
|---|---|
| `src-tauri/src/http_server/handlers.rs` | Ajouter `tori=true` dans l'URL `navigated_to` (ligne 830) |
| `src-tauri/src/config.rs` | Ajouter migration automatique de l'ancienne URL dans la methode `load()` |

## Resultat

Apres ces corrections :
- Les exe existants se corrigeront automatiquement au prochain lancement (migration config)
- Les nouveaux builds auront la bonne URL partout
- Le endpoint `/open-report` retournera la bonne URL dans sa reponse

