# 📥 Installation Windows - AirADCR Desktop

## 🎯 Options d'Installation Disponibles

### ✅ **Option 1 : Application Web (Prête Maintenant)**
L'application actuelle est **prête à utiliser** comme application web progressive :

#### Installation Immédiate
1. **Accès direct** : Ouvrir dans un navigateur moderne
2. **PWA** : Peut être "installée" comme app depuis Chrome/Edge
3. **Raccourci bureau** : Créable via le navigateur

#### Avantages
- ✅ **Disponible immédiatement**
- ✅ **Aucune installation complexe**
- ✅ **Mises à jour automatiques**
- ✅ **Compatible tous Windows**

---

### 🚧 **Option 2 : Application Desktop Native (Tauri)**
Pour créer un vrai `.exe` Windows, il faudrait configurer Tauri :

#### Étapes Nécessaires
```bash
# 1. Installer Rust
# Depuis https://rustup.rs/

# 2. Installer Tauri CLI
npm install -g @tauri-apps/cli

# 3. Initialiser Tauri
npm run tauri init

# 4. Configuration Windows
# Éditer src-tauri/tauri.conf.json

# 5. Build Windows
npm run tauri build
```

#### Fonctionnalités Desktop Supplémentaires
- 🔝 **Always-on-top** natif
- 💉 **Injection système** directe
- 🖥️ **Détection fenêtres** avancée
- 📋 **Accès clipboard** natif
- 🔧 **Intégration Windows** complète

---

## 🚀 **Déploiement Recommandé (Option 1)**

### Installation Web Progressive

#### 1. **Via Navigateur (Recommandé)**
```
1. Ouvrir Chrome/Edge
2. Aller sur : [URL de l'app après déploiement]
3. Cliquer sur l'icône "Installer" dans la barre d'adresse
4. L'app apparaît comme une vraie application Windows
```

#### 2. **Raccourci Bureau Manuel**
```
1. Créer un raccourci vers :
   chrome.exe --app="[URL]" --window-size=1200,800
2. Icône personnalisée AirADCR
3. Démarrage automatique possible
```

#### 3. **Distribution Entreprise**
```
Script PowerShell pour installation automatique :
- Création raccourci bureau
- Configuration always-on-top (via navigateur)
- Démarrage automatique Windows
```

---

## 📦 **Package d'Installation Prêt**

### Contenu du Package
- ✅ **Executable/Raccourci** : Lance l'application web
- ✅ **Icône AirADCR** : Design médical professionnel
- ✅ **Installation silencieuse** : Pour déploiement IT
- ✅ **Documentation** : Guide utilisateur

### Configuration Système
```ini
[AirADCR Desktop]
URL=https://[votre-domaine]/
WindowSize=1200x800
AlwaysOnTop=true
StartWithWindows=true
```

---

## 🛠️ **Instructions IT/Admin**

### Déploiement Réseau
1. **Group Policy** : Distribution automatique raccourci
2. **SCCM/Intune** : Installation centralisée
3. **Script Batch** : Configuration poste par poste

### Prérequis Système
- ✅ **Windows 10/11** (toutes versions)
- ✅ **Chrome 90+** ou **Edge 90+**
- ✅ **Connexion Internet** (pour airadcr.com)
- ✅ **Permissions utilisateur** standard

### Sécurité IT
- 🔒 **Domaines autorisés** : Uniquement airadcr.com
- 🛡️ **Sandbox navigateur** : Isolation complète
- 📊 **Logs audit** : Traçabilité accès
- 🔐 **SSO compatible** : Intégration AD possible

---

## 📋 **Checklist Déploiement**

### ✅ **Application Actuelle**
- [x] Code optimisé et sécurisé
- [x] Interface simplifiée (iframe airadcr.com)
- [x] Design system médical AirADCR
- [x] Responsive toutes résolutions
- [x] Gestion erreurs réseau

### 🚧 **Pour Version Desktop (.exe)**
- [ ] Configuration Tauri
- [ ] Compilation Rust/Windows
- [ ] Signature code certificat
- [ ] Installeur MSI/NSIS
- [ ] Tests compatibilité Windows

---

## 🎯 **Recommandation Finale**

### **Démarrage Immédiat : Option 1 (Web)**
L'application est **prête maintenant** pour déploiement web avec toutes les fonctionnalités AirADCR intégrées.

### **Évolution Future : Option 2 (Desktop)**
Si besoin de fonctionnalités Windows natives avancées, configuration Tauri possible en 2-3 jours supplémentaires.

---

## 📞 **Support Installation**

### Questions Fréquentes
- **"Peut-on l'installer sans admin ?"** → Oui (option web)
- **"Fonctionne-t-il hors ligne ?"** → Non (nécessite airadcr.com)
- **"Compatible serveur terminal ?"** → Oui parfaitement
- **"Intégration EMR possible ?"** → Via airadcr.com existant