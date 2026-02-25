

## Corriger l'installation multi-sessions Windows

### Diagnostic

Le probleme vient de la combinaison de deux reglages dans `src-tauri/tauri.conf.json` :

1. **NSIS `installMode: "perMachine"`** (ligne 102) : installe l'application dans `C:\Program Files`, accessible a toutes les sessions Windows
2. **`webviewInstallMode: "downloadBootstrapper"`** (ligne 80-82) : tente de telecharger et installer WebView2 pendant l'installation

Sur la **session principale**, le bootstrapper WebView2 fonctionne car il a les permissions administrateur initiales. Sur une **session secondaire**, le bootstrapper echoue car :
- WebView2 est deja installe au niveau systeme par la premiere session
- Le bootstrapper detecte un conflit ou n'a pas les permissions pour reinstaller
- L'echec du bootstrapper fait planter tout l'installeur MSI et NSIS

### Lien avec TÉO Hub
Si l'application ne s'installe pas sur la session secondaire, le serveur HTTP local (port 8741) ne demarre jamais. TÉO Hub/Orthanc envoie ses requetes dans le vide -- ce qui explique l'absence de connexion.

### Modifications

**Fichier : `src-tauri/tauri.conf.json`**

1. Changer `webviewInstallMode` de `"downloadBootstrapper"` a `"skip"` (ligne 80-82)
   - WebView2 est deja integre a Windows 10 (1803+) et Windows 11
   - Plus besoin de le telecharger, ce qui elimine la source de l'echec

2. Egalement mettre `skipWebviewInstall: true` dans la section wix (ligne 91) pour coherence avec le MSI

### Impact
- L'installation fonctionnera sur toutes les sessions Windows sans conflit
- Le serveur HTTP demarrera correctement sur la session secondaire
- TÉO Hub pourra atteindre le port 8741
- Aucun impact sur les machines modernes (WebView2 deja present)
- Le portable EXE continue de fonctionner comme avant

