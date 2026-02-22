

# Correction des erreurs de build Rust -- variables `x` et `y` introuvables

## Probleme

Deux fonctions Tauri utilisent des parametres prefixes par un underscore (`_x`, `_y`) mais les referencent sans underscore (`x`, `y`) dans le corps de la fonction, ce qui provoque l'erreur `cannot find value x/y in this scope`.

## Corrections

### 1. Fichier `src-tauri/src/main.rs`, ligne 581

Renommer les parametres de `_x, _y` en `x, y` :

```rust
// Avant
async fn get_window_at_point(_x: i32, _y: i32)

// Apres
async fn get_window_at_point(x: i32, y: i32)
```

### 2. Fichier `src-tauri/src/main.rs`, ligne 774

Meme correction :

```rust
// Avant
async fn get_window_client_rect_at_point(_x: i32, _y: i32)

// Apres
async fn get_window_client_rect_at_point(x: i32, y: i32)
```

## Explication

En Rust, le prefixe `_` sur un parametre indique au compilateur que la variable est intentionnellement inutilisee. Ici les parametres sont bien utilises (lignes 587 et 779), donc le prefixe doit etre retire.

## Impact

- Corrige les 4 erreurs de compilation (`E0425`)
- Aucun changement fonctionnel
- Deux lignes a modifier

