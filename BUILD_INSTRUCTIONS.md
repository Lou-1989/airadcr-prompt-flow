# 🔨 Instructions de Build pour AirADCR Desktop

## Prérequis

### 1. Installation Rust
```bash
# Windows
winget install Rustlang.Rust

# Ou via le site officiel
https://rustup.rs/
```

### 2. Installation Tauri CLI
```bash
cargo install tauri-cli
```

### 3. Installation WiX Toolset (pour MSI)
```bash
# Via winget
winget install wixtoolset.wix

# Ou télécharger depuis
https://wixtoolset.org/
```

## Build de l'application

### 1. Build de développement
```bash
# Démarrer en mode développement
npm run tauri dev
```

### 2. Build de production (.exe)
```bash
# Build complet avec installeur
npm run tauri build

# Build sans installeur (portable)
npm run tauri build -- --config '{"bundle":{"active":false}}'
```

### 3. Fichiers générés

Après le build, vous trouverez dans `src-tauri/target/release/bundle/`:

- **msi/**: `AirADCR Desktop_1.0.0_x64_en-US.msi` - Installeur MSI
- **nsis/**: `AirADCR Desktop_1.0.0_x64-setup.exe` - Installeur NSIS
- **../**: `airadcr-desktop.exe` - Exécutable portable

## Scripts de build personnalisés

### Package.json à mettre à jour
```json
{
  "scripts": {
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "tauri:build-portable": "tauri build --config '{\"bundle\":{\"active\":false}}'",
    "tauri:build-release": "tauri build --config '{\"bundle\":{\"windows\":{\"certificateThumbprint\":null,\"timestampUrl\":\"\"}}}'"
  }
}
```

## Signature du code (optionnel)

Pour éviter les avertissements Windows Defender:

1. Obtenir un certificat de signature de code
2. Configurer dans `tauri.conf.json`:
```json
"windows": {
  "certificateThumbprint": "VOTRE_THUMBPRINT",
  "timestampUrl": "http://timestamp.comodoca.com"
}
```

## 🎯 Résultat Final

✅ **AirADCR-Desktop-Setup.exe** (~15-20 MB)  
✅ **Fonctionnalités natives Windows**  
✅ **Always-on-top configurable**  
✅ **System tray integration**  
✅ **Raccourcis clavier globaux**  
✅ **Installation propre avec désinstalleur**

## Support

L'application fonctionne sur:
- Windows 10 (1903+)  
- Windows 11
- Architecture x64