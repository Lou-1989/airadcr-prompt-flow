# 🚀 GUIDE DE BUILD COMPLET - AirADCR Desktop

## ✅ PRÉREQUIS PAR PLATEFORME

### Windows
```bash
# Rust + Tauri CLI
winget install Rustlang.Rust
cargo install tauri-cli --version "^1.0"

# WiX Toolset pour MSI
winget install WiXToolset.WiX
```

### macOS  
```bash
# Rust + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli --version "^1.0"

# Xcode Command Line Tools
xcode-select --install
```

### Linux
```bash
# Rust + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tauri-cli --version "^1.0"

# Dépendances système (Ubuntu/Debian)
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## 🔨 COMMANDES DE BUILD MULTI-PLATEFORMES

### Développement
```bash
npx tauri dev
```

### Production - Build complet (RECOMMANDÉ)
```bash
npx tauri build
```

### Builds spécifiques par plateforme
```bash
# Windows uniquement
npx tauri build --target x86_64-pc-windows-msvc

# macOS uniquement  
npx tauri build --target x86_64-apple-darwin
npx tauri build --target aarch64-apple-darwin  # Apple Silicon

# Linux uniquement
npx tauri build --target x86_64-unknown-linux-gnu
```

## 📦 FICHIERS GÉNÉRÉS PAR PLATEFORME

### 🪟 **Windows** - `src-tauri/target/release/bundle/`
- **📁 msi/** : `AirADCR Desktop_1.0.0_x64_en-US.msi` (entreprise)
- **📁 nsis/** : `AirADCR Desktop_1.0.0_x64-setup.exe` (grand public)  
- **📁 portable/** : `airadcr-desktop.exe` (portable 8-12 MB)

### 🍎 **macOS** - `src-tauri/target/release/bundle/`
- **📁 dmg/** : `AirADCR Desktop_1.0.0_x64.dmg` (installeur drag & drop)
- **📁 macos/** : `AirADCR Desktop.app` (application bundle)

### 🐧 **Linux** - `src-tauri/target/release/bundle/`
- **📁 deb/** : `airadcr-desktop_1.0.0_amd64.deb` (Debian/Ubuntu)
- **📁 rpm/** : `airadcr-desktop-1.0.0-1.x86_64.rpm` (RedHat/Fedora)
- **📁 appimage/** : `airadcr-desktop_1.0.0_amd64.AppImage` (portable)

## 🎯 FONCTIONNALITÉS NATIVES INTÉGRÉES

### ✅ **Always-on-top** (tous OS)
- Windows : Injection système native
- macOS : Window level management  
- Linux : X11/Wayland compatible

### ✅ **System Tray** (tous OS)
- Windows : Notification area
- macOS : Menu bar integration
- Linux : System indicator

### ✅ **Window Management** (tous OS)
- Maximize/minimize
- Positionnement précis
- Focus management
- Fullscreen support

### ✅ **Raccourcis globaux** (tous OS)
- Ctrl+Alt+T : Toggle always-on-top
- Ctrl+Alt+H : Hide/Show window
- Cmd sur macOS au lieu de Ctrl

## 🍎 CONFIGURATION DMG OPTIMISÉE

### Design DMG personnalisé :
- **Taille fenêtre** : 660x400px
- **Position app** : Optimisée pour drag & drop
- **Folder Applications** : Visible et accessible
- **Background** : Extensible (ajout futur d'une image)

### Paramètres macOS :
- **Version minimum** : macOS 10.13 (High Sierra)
- **Support Apple Silicon** : ✅ (ARM64)
- **Support Intel** : ✅ (x64)

## ⚡ ASSETS COMPLETS MULTI-OS

### ✅ **Icônes universelles** :
- `src-tauri/icons/32x32.png` (Windows/Linux)
- `src-tauri/icons/128x128.png` (toutes plateformes)  
- `src-tauri/icons/128x128@2x.png` (macOS Retina)
- `src-tauri/icons/icon.png` (Linux/system tray)
- `src-tauri/icons/icon.ico` (Windows)

### ✅ **Assets installeurs** :
- Windows : banner.bmp, dialog.bmp, header.bmp, sidebar.bmp
- macOS : DMG layout automatique
- Linux : Desktop entries et icônes système

## 🚀 COMMANDS FINALES PAR OS

### Windows
```bash
npx tauri build --target x86_64-pc-windows-msvc
# Génère : MSI + NSIS + Portable EXE
```

### macOS (Intel)
```bash  
npx tauri build --target x86_64-apple-darwin
# Génère : DMG + .app bundle
```

### macOS (Apple Silicon)
```bash
npx tauri build --target aarch64-apple-darwin  
# Génère : DMG + .app bundle (ARM64)
```

### Linux
```bash
npx tauri build --target x86_64-unknown-linux-gnu
# Génère : DEB + RPM + AppImage
```

### **Build universel (RECOMMANDÉ)**
```bash
npx tauri build
# Génère automatiquement pour l'OS courant
```

## 🎉 RÉSULTAT MULTI-PLATEFORME

✅ **Windows** : 3 formats (MSI, NSIS, EXE)  
✅ **macOS** : DMG professionnel + App bundle  
✅ **Linux** : 3 formats (DEB, RPM, AppImage)  

**🎯 Distribution universelle garantie pour AirADCR Desktop !**

---

### 📋 **Checklist Build Multi-OS**

- [x] **Assets générés** : Toutes icônes ✅
- [x] **Configuration DMG** : macOS ready ✅  
- [x] **Licences** : MIT .txt + .rtf ✅
- [x] **Code signé** : Configurable par OS
- [x] **Tests** : `npx tauri dev` fonctionnel ✅

### ⏱️ **Durées estimées par OS**
- Windows : 3-5 minutes
- macOS : 4-6 minutes  
- Linux : 3-4 minutes