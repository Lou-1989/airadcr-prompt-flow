

# Correction du build CI -- SQLCipher sans OpenSSL externe

## Probleme

La feature `bundled-sqlcipher` de rusqlite necessite qu'OpenSSL soit installe sur la machine de build (variable `OPENSSL_DIR`). Le runner CI Windows (GitHub Actions) ne l'a pas pre-installe, d'ou l'echec.

## Solution

Remplacer la feature `bundled-sqlcipher` par `bundled-sqlcipher-vendored-openssl`. Cette variante compile OpenSSL directement dans le binaire -- aucune dependance externe requise. Le chiffrement AES-256 reste identique.

## Modification

### Fichier : `src-tauri/Cargo.toml`, ligne 41

Remplacer :
```toml
rusqlite = { version = "0.31", features = ["bundled-sqlcipher"] }
```

Par :
```toml
rusqlite = { version = "0.31", features = ["bundled-sqlcipher-vendored-openssl"] }
```

## Impact

- Aucun changement fonctionnel (meme chiffrement AES-256)
- Build CI Windows : corrige l'erreur `Missing environment variable OPENSSL_DIR`
- Build CI macOS : fonctionne egalement (plus besoin de `brew install openssl`)
- Taille du binaire : augmentation negligeable (~1-2 MB pour OpenSSL statique)
- Une seule ligne a modifier

