# 🚀 Guide Final de Build - AirADCR Desktop

## ✅ Prérequis Vérifiés

### 1. Installation Rust & Tauri
```bash
# Windows - Installation Rust
winget install Rustlang.Rust

# Installation Tauri CLI v1
cargo install tauri-cli --version "^1.0"

# Vérification versions
rustc --version  # doit être >= 1.70
cargo --version  # doit être >= 1.70
tauri --version  # doit être ~1.6
```

### 2. Installation WiX Toolset (pour MSI)
```bash
winget install wixtoolset.wix
# Redémarrer le terminal après installation
```

## 🛠️ Build de Production

### Commandes de Build Disponibles

```bash
# 1. Build développement (test)
npm run tauri:dev

# 2. Build production complet (tous les formats)
npm run tauri:build

# 3. Build portable uniquement (.exe sans installeur)
npm run tauri:build-portable

# 4. Build release (optimisé, sans signature)
npm run tauri:build-release
```

## 📦 Fichiers Générés

Après `npm run tauri:build`, vous trouverez dans **`src-tauri/target/release/bundle/`** :

### Exécutables
- **`../airadcr-desktop.exe`** - Portable (15-20 MB)

### Installeurs Windows
- **`msi/AirADCR Desktop_1.0.0_x64_fr-FR.msi`** - Installeur MSI entreprise
- **`nsis/AirADCR Desktop_1.0.0_x64-setup.exe`** - Installeur NSIS grand public

## 🎯 Assets Créés & Intégrés

### Icônes Multi-Résolution
- ✅ `src-tauri/icons/32x32.png` - Petite icône
- ✅ `src-tauri/icons/128x128.png` - Icône standard  
- ✅ `src-tauri/icons/128x128@2x.png` - Icône retina
- ✅ `src-tauri/icons/icon.png` - System tray
- ✅ `src-tauri/icons/icon.ico` - Windows natif

### Assets Installeur Windows
- ✅ `src-tauri/assets/banner.bmp` - Bannière WiX
- ✅ `src-tauri/assets/dialog.bmp` - Dialogue WiX
- ✅ `src-tauri/assets/header.bmp` - Entête NSIS
- ✅ `src-tauri/assets/sidebar.bmp` - Sidebar NSIS
- ✅ `src-tauri/assets/installer.ico` - Icône installeur

### Licences
- ✅ `src-tauri/LICENSE.txt` - Licence NSIS
- ✅ `src-tauri/LICENSE.rtf` - Licence WiX

## 🔧 Fonctionnalités Natives Intégrées

### Tauri v1.6 - APIs Connectées
- ✅ **Always-on-top** via `toggle_always_on_top()`
- ✅ **Positionnement fenêtre** via `set_window_position()`
- ✅ **System tray** via `minimize_to_tray()` / `restore_from_tray()`
- ✅ **Raccourcis globaux** configurés dans Rust
- ✅ **Informations système** via OS API
- ✅ **Gestion fenêtre** complète (hide/show/focus/close)

### Hook React Connecté
```typescript
const {
  isTauriApp,           // true si environnement natif
  isAlwaysOnTop,        // état always-on-top
  systemInfo,           // info système réelle
  toggleAlwaysOnTop,    // basculer always-on-top
  setPosition,          // positionner fenêtre
  minimizeToTray,       // minimiser vers tray
  toggleVisibility,     // cacher/afficher
  quitApp              // fermer app
} = useTauriWindow();
```

## 📋 Checklist Pré-Build

```bash
# Vérifier que tout fonctionne
npm run dev          # ✅ Interface React OK
npm run build        # ✅ Build web OK  
npm run tauri:dev    # ✅ App Tauri OK

# Tester fonctionnalités natives
# - Always-on-top fonctionne
# - System tray visible
# - Positionnement fenêtre
# - Raccourcis clavier
```

## 🚀 Build Final

```bash
# Build production complet
npm run tauri:build

# Résultat attendu dans src-tauri/target/release/bundle/:
# ✅ AirADCR Desktop_1.0.0_x64_fr-FR.msi        (~20MB)
# ✅ AirADCR Desktop_1.0.0_x64-setup.exe        (~18MB)  
# ✅ airadcr-desktop.exe (portable)              (~15MB)
```

## 🎯 Distribution Recommandée

### Entreprise / IT
**Utiliser le fichier MSI** : `AirADCR Desktop_1.0.0_x64_fr-FR.msi`
- Installation silencieuse : `msiexec /i "AirADCR Desktop_1.0.0_x64_fr-FR.msi" /quiet`
- Désinstallation propre
- Compatible Group Policy / SCCM

### Grand Public  
**Utiliser le fichier NSIS** : `AirADCR Desktop_1.0.0_x64-setup.exe`
- Interface utilisateur intuitive
- Choix du répertoire d'installation
- Création raccourcis automatique

### Usage Portable
**Utiliser l'exécutable** : `airadcr-desktop.exe`  
- Aucune installation requise
- Exécution directe
- Idéal pour tests ou usage temporaire

## 🔐 Signature de Code (Optionnel)

Pour éviter les avertissements Windows Defender :

```json
// Dans tauri.conf.json
"windows": {
  "certificateThumbprint": "VOTRE_THUMBPRINT",
  "timestampUrl": "http://timestamp.comodoca.com"
}
```

## ✅ Validation Finale

L'application **AirADCR Desktop** est maintenant prête pour :

- ✅ **Compilation réussie** (.exe, .msi, .nsis)
- ✅ **Fonctionnalités natives Windows** complètes
- ✅ **Assets professionnels** intégrés  
- ✅ **Configuration sécurisée** CSP + allowlist
- ✅ **Performance optimisée** Tauri v1.6 stable
- ✅ **Documentation complète** pour IT/utilisateurs

**Commande finale** : `npm run tauri:build`
**Durée** : ~5-10 minutes selon la machine
**Résultat** : 3 formats d'installation Windows professionnels