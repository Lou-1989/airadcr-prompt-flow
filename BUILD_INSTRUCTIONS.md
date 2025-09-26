# 🔨 Instructions de Build pour AirADCR Desktop - VERSION FINALE

## ✅ CORRECTIONS APPLIQUÉES

### Problèmes Résolus
- ✅ Conflit versions Tauri (v1.6 stable)
- ✅ Hook React connecté aux vraies APIs  
- ✅ Icônes multi-résolutions créées
- ✅ Assets d'installeur Windows générés
- ✅ Licences et configuration optimisées
- ✅ Scripts package.json intégrés

## Prérequis

### 1. Installation Rust
```bash
# Windows
winget install Rustlang.Rust

# Ou via le site officiel
https://rustup.rs/
```

### 2. Installation Tauri CLI v1
```bash
cargo install tauri-cli --version "^1.0"
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
npm run tauri:dev
```

### 2. Build de production COMPLET
```bash
# Build avec tous les installeurs (RECOMMANDÉ)
npm run tauri:build

# Build sans installeur (portable uniquement)
npm run tauri:build-portable

# Build optimisé release
npm run tauri:build-release
```

### 3. Fichiers générés

Après le build, vous trouverez dans `src-tauri/target/release/bundle/`:

- **msi/**: `AirADCR Desktop_1.0.0_x64_fr-FR.msi` - Installeur MSI entreprise
- **nsis/**: `AirADCR Desktop_1.0.0_x64-setup.exe` - Installeur NSIS grand public  
- **../**: `airadcr-desktop.exe` - Exécutable portable

## Scripts disponibles (intégrés dans package.json)

```bash
npm run tauri          # CLI Tauri
npm run tauri:dev      # Mode développement
npm run tauri:build    # Build production complet
npm run tauri:build-portable  # Build portable uniquement
npm run tauri:build-release   # Build optimisé
```

## Assets Créés & Intégrés

### Icônes Multi-Résolution ✅
- `src-tauri/icons/32x32.png` - Petite icône
- `src-tauri/icons/128x128.png` - Icône standard
- `src-tauri/icons/128x128@2x.png` - Icône retina
- `src-tauri/icons/icon.png` - System tray
- `src-tauri/icons/icon.ico` - Windows natif

### Assets Installeur Windows ✅
- `src-tauri/assets/banner.bmp` - Bannière WiX MSI
- `src-tauri/assets/dialog.bmp` - Dialogue WiX
- `src-tauri/assets/header.bmp` - Entête NSIS
- `src-tauri/assets/sidebar.bmp` - Sidebar NSIS
- `src-tauri/assets/installer.ico` - Icône installeur

### Licences ✅
- `src-tauri/LICENSE.txt` - Licence NSIS
- `src-tauri/LICENSE.rtf` - Licence WiX RTF

## Fonctionnalités Natives Intégrées ✅

### APIs Tauri v1.6 Connectées
- **Always-on-top** : Basculement via raccourci
- **System tray** : Minimisation/restauration  
- **Positionnement fenêtre** : Contrôle précis
- **Raccourcis globaux** : Configuration Rust
- **Informations système** : OS, architecture, version
- **Gestion complète fenêtre** : Hide/show/focus/close

### Hook React useTauriWindow ✅
```typescript
const { 
  isTauriApp,          // Détection environnement natif
  isAlwaysOnTop,       // État always-on-top
  systemInfo,          // Info système réelle  
  toggleAlwaysOnTop,   // Basculer always-on-top
  setPosition,         // Positionner fenêtre
  minimizeToTray,      // Minimiser vers tray
  toggleVisibility,    // Cacher/afficher
  quitApp             // Fermer application
} = useTauriWindow();
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

✅ **AirADCR-Desktop-Setup.exe** (~18 MB) - NSIS  
✅ **AirADCR-Desktop.msi** (~20 MB) - MSI entreprise
✅ **airadcr-desktop.exe** (~15 MB) - Portable
✅ **Fonctionnalités natives Windows** complètes
✅ **Always-on-top configurable**  
✅ **System tray integration**  
✅ **Raccourcis clavier globaux**  
✅ **Installation propre avec désinstalleur**

## Support

L'application fonctionne sur:
- Windows 10 (1903+)  
- Windows 11
- Architecture x64

**COMMANDE FINALE POUR CRÉER L'EXE PARFAIT :**
```bash
npm run tauri:build
```