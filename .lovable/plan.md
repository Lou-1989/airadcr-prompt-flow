

# Correction de l'erreur de build -- keychain.rs

## Diagnostic

Il y a **1 erreur** et **5 warnings**. Tout est trivial a corriger, zero raison de revenir a SQLite non chiffre.

### Erreur (bloquante)

| Fichier | Ligne | Probleme | Correction |
|---------|-------|----------|------------|
| `keychain.rs` | 94 | `delete_credential()` n'existe pas dans `keyring` v2 | Remplacer par `delete_password()` |

### Warnings (non bloquants mais a nettoyer)

| Fichier | Ligne | Probleme | Correction |
|---------|-------|----------|------------|
| `keychain.rs` | 10 | Import `error` inutilise | Retirer `error` de la ligne d'import |
| `main.rs` | 581 | Variables `x`, `y` inutilisees dans `get_window_at_point` | Prefixer avec `_` : `_x`, `_y` |
| `main.rs` | 774 | Variables `x`, `y` inutilisees dans `get_window_client_rect_at_point` | Prefixer avec `_` : `_x`, `_y` |

## Modifications

### Fichier 1 : `src-tauri/src/database/keychain.rs`

- Ligne 10 : changer `use log::{info, warn, error};` en `use log::{info, warn};`
- Ligne 94 : changer `entry.delete_credential()` en `entry.delete_password()`

### Fichier 2 : `src-tauri/src/main.rs`

- Ligne 581 : changer `x: i32, y: i32` en `_x: i32, _y: i32`
- Ligne 774 : changer `x: i32, y: i32` en `_x: i32, _y: i32`

## Verdict

4 lignes a modifier. SQLCipher + Keychain OS restent en place. Aucun impact fonctionnel.

