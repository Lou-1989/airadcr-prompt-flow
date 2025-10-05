# 🚀 Guide de déploiement - Profil SpeechMike pour AIRADCR

**Version:** 1.0  
**Date:** 2025-10-05  
**Public cible:** Radiologues et techniciens IT

---

## 📋 Vue d'ensemble

Ce guide explique comment installer et configurer le profil SpeechMike pour l'application AIRADCR Desktop. Le profil permet au micro Philips SpeechMike de contrôler la dictée radiologique via des touches F-keys détectées globalement par Tauri.

---

## 🎯 Prérequis

### Logiciels requis

- ✅ **Philips SpeechControl** (version 9.0 ou supérieure)
- ✅ **AIRADCR Desktop Application** installée (`airadcr-desktop.exe`)
- ✅ **Philips SpeechMike** connecté et reconnu par Windows
- ⚠️ **Dragon NaturallySpeaking** (optionnel, mais doit coexister si installé)

### Fichiers nécessaires

- 📄 `airadcr_speechmike_profile.xml` (fourni dans le package d'installation)

---

## 📥 Installation du profil SpeechMike

### Étape 1 : Ouvrir Philips SpeechControl

1. Lancer **Philips SpeechControl** depuis le menu Démarrer
2. Si le logiciel n'est pas installé, télécharger depuis : [https://www.speech.philips.com/support](https://www.speech.philips.com/support)

### Étape 2 : Accéder aux paramètres d'Application Control

1. Dans SpeechControl, cliquer sur l'icône **⚙️ Settings** (en haut à droite)
2. Aller dans l'onglet **Application Control**
3. Vérifier que la section "Application Profiles" est visible

### Étape 3 : Importer le profil AIRADCR

1. Cliquer sur le bouton **Import Profile** (ou **Importer un profil**)
2. Sélectionner le fichier `airadcr_speechmike_profile.xml`
3. Confirmer l'import
4. Un message de succès devrait apparaître : ✅ "Profile imported successfully"

### Étape 4 : Vérifier le profil

1. Dans la liste des profils, vérifier la présence de :
   - **Profil par défaut** : "Dragon NaturallySpeaking par défaut" (TargetApplication: vide)
   - **Profil AIRADCR** : "AIRADCR Desktop - Dictée radiologique" (TargetApplication: `airadcr-desktop.exe`)
2. S'assurer que les deux profils ont le statut **Active: True** (case cochée)

### Étape 5 : Redémarrer SpeechControl

1. Fermer complètement Philips SpeechControl
2. Relancer SpeechControl
3. Le profil AIRADCR devrait maintenant être actif et prêt à basculer automatiquement

---

## 🧪 Tests de validation

### Test 1 : Détection automatique de l'application

**Objectif** : Vérifier que SpeechControl bascule automatiquement sur le profil AIRADCR

**Procédure** :
1. Lancer **Dragon NaturallySpeaking** (si installé)
2. Appuyer sur le **bouton rouge** du SpeechMike
3. Vérifier que Dragon démarre/arrête la reconnaissance vocale ✅
4. Lancer **AIRADCR Desktop**
5. Donner le focus à AIRADCR (cliquer dans la fenêtre)
6. Appuyer sur le **bouton rouge** du SpeechMike
7. Vérifier que l'enregistrement démarre dans AIRADCR (bouton bleu devient "🔴 Arrêter") ✅

**Résultat attendu** : Le SpeechMike bascule automatiquement entre Dragon et AIRADCR selon l'application active.

---

### Test 2 : Workflow complet de dictée

**Objectif** : Valider le cycle complet Record → Pause → Resume → Finish

**Procédure** :
1. **Démarrer l'enregistrement** :
   - Appuyer sur le **bouton rouge** du SpeechMike (ou touche **F10** du clavier)
   - Vérifier que le bouton bleu AIRADCR affiche "🔴 Arrêter"
   - Dicter un texte de test : *"Examen IRM cérébrale sans injection..."*

2. **Mettre en pause** :
   - Appuyer **1 fois** sur le **bouton rouge** du SpeechMike
   - Vérifier que le bouton AIRADCR affiche "⏸️ En pause"

3. **Reprendre l'enregistrement** :
   - Appuyer **1 fois** sur le **bouton rouge** (ou bouton **Play**)
   - Vérifier que le bouton AIRADCR affiche "🔴 Arrêter"
   - Continuer la dictée : *"...sans anomalie détectée"*

4. **Terminer et transcrire** :
   - Appuyer **1 fois** sur le **bouton rouge** (pour pause)
   - Appuyer **2ème fois** sur le **bouton rouge** (pour finish)
   - Vérifier que la transcription démarre et le rapport structuré apparaît

**Résultat attendu** : ✅ L'enregistrement se comporte comme attendu à chaque étape.

---

### Test 3 : Capture globale (sans focus AIRADCR)

**Objectif** : Vérifier que F10 fonctionne même quand AIRADCR n'a pas le focus

**Procédure** :
1. Lancer AIRADCR Desktop
2. Ouvrir une autre application (ex: Word, RIS)
3. Donner le focus à cette autre application (cliquer dedans)
4. Appuyer sur le **bouton rouge** du SpeechMike (ou **F10** du clavier)
5. Vérifier que l'enregistrement démarre dans AIRADCR (vérifier visuellement le bouton dans AIRADCR)

**Résultat attendu** : ✅ AIRADCR réagit même sans avoir le focus (capture globale via GlobalShortcut Tauri).

---

## 🔧 Dépannage

### Problème 1 : Le SpeechMike ne répond pas dans AIRADCR

**Symptômes** :
- Appuyer sur le bouton rouge ne démarre pas l'enregistrement
- Aucune réaction dans AIRADCR

**Solutions** :

1. **Vérifier le profil actif** :
   - Ouvrir SpeechControl → Application Control
   - S'assurer que "AIRADCR Desktop" est coché (Active: True)
   - Vérifier que `TargetApplication` est bien `airadcr-desktop.exe`

2. **Vérifier le nom de l'exécutable** :
   - Ouvrir le Gestionnaire des tâches Windows (Ctrl+Shift+Esc)
   - Chercher le processus AIRADCR
   - Si le nom est différent de `airadcr-desktop.exe`, éditer le profil XML :
     ```xml
     <ApplicationProfile TargetApplication="VOTRE_NOM_REEL.exe" Active="True">
     ```

3. **Tester avec les touches clavier** :
   - Appuyer manuellement sur **F10** du clavier
   - Si F10 fonctionne mais pas le SpeechMike, le problème vient du profil SpeechControl
   - Si F10 ne fonctionne pas non plus, vérifier les logs Tauri (voir section Logs ci-dessous)

4. **Redémarrer SpeechControl et AIRADCR** :
   ```
   1. Fermer SpeechControl complètement (vérifier dans la barre système)
   2. Fermer AIRADCR Desktop
   3. Relancer SpeechControl
   4. Relancer AIRADCR Desktop
   ```

---

### Problème 2 : Le bouton rouge ne met pas en pause correctement

**Symptômes** :
- Le premier appui sur le bouton rouge ne met pas en pause
- Il faut appuyer plusieurs fois

**Solutions** :

1. **Vérifier la synchronisation d'état** :
   - Ouvrir la Console DevTools dans AIRADCR (F12)
   - Vérifier les logs : `[useSecureMessaging] Commande Record reçue`
   - Vérifier les logs Tauri : `[DictationState] Recording - Appui #1`

2. **Vérifier que le web notifie Tauri** :
   - Après démarrage enregistrement, vérifier log : `[Tauri←Web] airadcr:recording_started`
   - Si ce message n'apparaît pas, le problème vient de `useSecureMessaging.ts`

---

### Problème 3 : Conflit avec Dragon NaturallySpeaking

**Symptômes** :
- Dragon et AIRADCR réagissent tous les deux au SpeechMike
- Ou au contraire, aucun des deux ne réagit

**Solutions** :

1. **Vérifier l'ordre des profils** :
   - Le profil AIRADCR doit être **plus spécifique** (avec TargetApplication)
   - Le profil Dragon doit être **générique** (TargetApplication vide)
   - SpeechControl choisit automatiquement le profil le plus spécifique

2. **Forcer le profil AIRADCR** :
   - Dans SpeechControl → Application Control
   - Décocher temporairement le profil Dragon par défaut
   - Tester uniquement avec AIRADCR
   - Recocher le profil Dragon ensuite

---

## 📊 Vérification des logs

### Logs Tauri (Backend)

**Comment y accéder** :
1. Lancer AIRADCR Desktop depuis un terminal :
   ```bash
   cd "C:\Program Files\AIRADCR"
   .\airadcr-desktop.exe
   ```
2. Les logs Rust s'affichent dans le terminal

**Logs à rechercher** :
```
✅ [SpeechMike] Shortcuts F10/F11/F12 enregistrés
🎤 [SpeechMike] F10 (Record) pressé
⏸️ [SpeechMike] F11 (Pause) pressé
✅ [SpeechMike] F12 (Finish) pressé
🟢 [SpeechMike] Enregistrement démarré (notifié par le web)
```

---

### Logs Web (Frontend)

**Comment y accéder** :
1. Dans AIRADCR Desktop, appuyer sur **F12** pour ouvrir DevTools
2. Aller dans l'onglet **Console**

**Logs à rechercher** :
```
[Tauri→Web] Envoi commande: airadcr:speechmike_record
🎤 [SpeechMike] Commande Record reçue
[useSecureMessaging] Notification Tauri: recording_started
```

---

## 🎛️ Personnalisation du profil

### Changer le mapping des boutons

Si vous souhaitez modifier le comportement du SpeechMike, éditer le fichier XML :

**Exemple** : Mapper le bouton **Stop** pour terminer directement (au lieu de pause) :

```xml
<Operation Event="spmStopPressed" ModifierKey="None" Name="AIRADCR Finish">
  <Steps>
    <OperationElement Type="Hotkey">
      <Hotkey Key="F12" Modifiers="None" />
    </OperationElement>
  </Steps>
</Operation>
```

Puis réimporter le profil dans SpeechControl.

---

### Activer les boutons F1-F4 pour des fonctions personnalisées

Les boutons **F1, F2, F3, F4** du SpeechMike peuvent être mappés sur des actions personnalisées :

**Exemple** : F1 = Copier le rapport dans le presse-papiers

```xml
<Operation Event="spmF1Pressed" ModifierKey="None" Name="Copy Report">
  <Steps>
    <OperationElement Type="Hotkey">
      <Hotkey Key="C" Modifiers="Control" />
    </OperationElement>
  </Steps>
</Operation>
```

---

## 📞 Support

### En cas de problème persistant

1. **Vérifier la version de SpeechControl** :
   - Version minimale requise : 9.0
   - Télécharger la dernière version sur [https://www.speech.philips.com](https://www.speech.philips.com)

2. **Consulter la documentation technique** :
   - `SPEECHMIKE_TAURI_SPECIFICATION.md` - Spécification complète du protocole
   - `AIRADCR_COMMUNICATION_REFERENCE.md` - Référence de communication web/desktop

3. **Contacter le support AIRADCR** :
   - Email : support@airadcr.com
   - Forum : [https://forum.airadcr.com](https://forum.airadcr.com)

---

## ✅ Checklist de validation post-installation

- [ ] Profil AIRADCR importé et actif dans SpeechControl
- [ ] Test 1 réussi : Détection automatique de l'application ✅
- [ ] Test 2 réussi : Workflow complet Record → Pause → Resume → Finish ✅
- [ ] Test 3 réussi : Capture globale sans focus AIRADCR ✅
- [ ] Logs Tauri affichent les messages `[SpeechMike]` correctement
- [ ] Logs Web affichent les messages `[useSecureMessaging]` correctement
- [ ] Compatibilité Dragon vérifiée (si installé)

---

**FIN DU GUIDE DE DÉPLOIEMENT**

*Dernière mise à jour : 2025-10-05*  
*Version : 1.0*  
*Équipe AIRADCR*
