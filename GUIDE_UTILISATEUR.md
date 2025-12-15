# 📖 Guide Utilisateur AIRADCR Desktop

**Application de dictée radiologique professionnelle**

---

## 📋 Table des matières

1. [Installation](#-installation)
2. [Premier démarrage](#-premier-démarrage)
3. [Utilisation quotidienne](#-utilisation-quotidienne)
4. [Configuration du SpeechMike](#-configuration-du-speechmike)
5. [Raccourcis clavier](#-raccourcis-clavier)
6. [Injection des rapports](#-injection-des-rapports)
7. [Résolution des problèmes](#-résolution-des-problèmes)

---

## 🔧 Installation

### Prérequis
- Windows 10 ou 11 (64 bits)
- Connexion internet
- 100 Mo d'espace disque disponible

### Étapes d'installation

1. **Téléchargez l'installateur**
   - Rendez-vous sur [airadcr.com](https://airadcr.com)
   - Cliquez sur "Télécharger l'application Desktop"
   - Le fichier `AIRADCR_1.0.0_x64-setup.exe` se télécharge

2. **Lancez l'installation**
   - Double-cliquez sur le fichier téléchargé
   - Si Windows affiche un avertissement, cliquez sur "Plus d'infos" puis "Exécuter quand même"
   - L'application est signée numériquement par SSL.com

3. **Suivez l'assistant**
   - Choisissez la langue (Français ou Anglais)
   - Acceptez les conditions d'utilisation
   - Cliquez sur "Installer"
   - Patientez quelques secondes

4. **Terminez**
   - Cochez "Lancer AIRADCR" si vous souhaitez démarrer immédiatement
   - Cliquez sur "Terminer"

---

## 🚀 Premier démarrage

### Connexion à votre compte

1. L'application s'ouvre et affiche la page de connexion AIRADCR
2. Connectez-vous avec vos identifiants habituels
3. L'interface de dictée apparaît

### Comportement de l'application

- **Toujours visible** : L'application reste au premier plan pour un accès rapide
- **Redimensionnable** : Ajustez la taille selon vos préférences
- **Mémorise sa position** : L'application réapparaît là où vous l'avez laissée

### Premier test

1. Ouvrez votre RIS ou un document Word
2. Placez votre curseur à l'endroit où vous voulez injecter du texte
3. Dans AIRADCR, dictez un court texte de test
4. Appuyez sur `Ctrl+Shift+T` pour injecter le texte

---

## 📅 Utilisation quotidienne

### Workflow recommandé

```
1. Lancez AIRADCR Desktop au démarrage de votre session
2. Ouvrez votre RIS/PACS habituel
3. Sélectionnez un examen à interpréter
4. Dictez votre rapport dans AIRADCR
5. Injectez le rapport dans votre RIS
6. Passez à l'examen suivant
```

### Modes de dictée

| Mode | Description | Raccourci |
|------|-------------|-----------|
| **Dictée continue** | Dicte tant que vous parlez | `Ctrl+Shift+D` |
| **Pause/Reprise** | Met en pause la dictée | `Ctrl+Shift+P` |

### Types d'injection

| Type | Description | Raccourci |
|------|-------------|-----------|
| **Texte brut** | Injecte le texte simple | `Ctrl+Shift+T` |
| **Rapport structuré** | Injecte le rapport formaté | `Ctrl+Shift+S` |

---

## 🎤 Configuration du SpeechMike

### Matériel compatible
- Philips SpeechMike Premium (LFH3500, LFH3600)
- Philips SpeechMike III (LFH3200, LFH3300)
- Autres modèles Philips SpeechMike

### Installation du profil

1. **Installez Philips SpeechControl**
   - Téléchargez depuis le site Philips
   - Installez avec les options par défaut

2. **Importez le profil AIRADCR**
   - Ouvrez SpeechControl
   - Allez dans "Configuration" > "Importer profil"
   - Sélectionnez le fichier `airadcr_speechmike_profile.xml`
   - Ce fichier se trouve dans le dossier d'installation

3. **Activez le profil**
   - Dans SpeechControl, sélectionnez "AIRADCR"
   - Cliquez sur "Activer"

### Boutons du SpeechMike

Une fois le profil activé :

| Bouton | Fonction |
|--------|----------|
| **Record (●)** | Démarre/arrête la dictée |
| **Play (▶)** | Pause/reprise |
| **F1** | Injecte le texte brut |
| **F2** | Injecte le rapport structuré |
| **EOL/EOM** | Valide et passe au suivant |

---

## ⌨️ Raccourcis clavier

### Raccourcis principaux

| Raccourci | Action |
|-----------|--------|
| `Ctrl+Shift+D` | Démarrer/arrêter la dictée |
| `Ctrl+Shift+P` | Pause/reprise de la dictée |
| `Ctrl+Shift+T` | Injecter le texte brut |
| `Ctrl+Shift+S` | Injecter le rapport structuré |

### Raccourcis de débogage (avancé)

| Raccourci | Action |
|-----------|--------|
| `Ctrl+Alt+D` | Ouvrir le panneau de débogage |
| `Ctrl+Alt+L` | Verrouiller la cible d'injection |
| `Ctrl+Alt+I` | Afficher les informations système |
| `F9` | Anti-ghost (réinitialise les raccourcis) |

---

## 💉 Injection des rapports

### Principe de fonctionnement

1. **Préparez la cible**
   - Ouvrez votre RIS, Word, ou autre application
   - Placez le curseur là où vous voulez injecter le texte

2. **Dictez dans AIRADCR**
   - Utilisez la dictée vocale pour créer votre rapport
   - Relisez et corrigez si nécessaire

3. **Injectez**
   - Appuyez sur `Ctrl+Shift+T` (texte) ou `Ctrl+Shift+S` (structuré)
   - Le texte apparaît à l'emplacement du curseur

### Verrouillage de cible

Si vous travaillez avec plusieurs écrans ou fenêtres :

1. Placez votre curseur dans la fenêtre cible
2. Appuyez sur `Ctrl+Alt+L` pour verrouiller
3. Les injections iront toujours vers cette fenêtre
4. Appuyez à nouveau sur `Ctrl+Alt+L` pour déverrouiller

### Conseils pour une injection fiable

- ✅ Assurez-vous que la fenêtre cible est ouverte
- ✅ Le curseur doit être dans un champ de texte éditable
- ✅ Évitez de changer de fenêtre pendant l'injection
- ✅ Utilisez le verrouillage si vous avez plusieurs écrans

---

## 🔧 Résolution des problèmes

### L'application ne démarre pas

**Symptôme** : Rien ne se passe quand vous double-cliquez sur l'icône

**Solutions** :
1. Vérifiez que Windows est à jour
2. Redémarrez votre ordinateur
3. Réinstallez l'application
4. Contactez le support AIRADCR

### La dictée ne fonctionne pas

**Symptôme** : Le bouton de dictée ne réagit pas

**Solutions** :
1. Vérifiez votre connexion internet
2. Rafraîchissez la page (clic droit > Actualiser)
3. Vérifiez les permissions du microphone dans Windows
4. Reconnectez-vous à votre compte AIRADCR

### L'injection ne fonctionne pas

**Symptôme** : Le texte ne s'insère pas dans votre RIS

**Solutions** :
1. Vérifiez que le curseur est bien dans un champ de texte
2. Essayez `Ctrl+Alt+L` pour verrouiller la cible
3. Fermez et rouvrez l'application
4. Appuyez sur `F9` pour réinitialiser les raccourcis

### Le SpeechMike ne répond pas

**Symptôme** : Les boutons du SpeechMike ne font rien

**Solutions** :
1. Vérifiez que SpeechControl est ouvert et actif
2. Vérifiez que le profil "AIRADCR" est sélectionné
3. Débranchez et rebranchez le SpeechMike
4. Redémarrez SpeechControl

### L'application ne reste pas au premier plan

**Symptôme** : L'application passe derrière d'autres fenêtres

**Solutions** :
1. Vérifiez les paramètres de l'application
2. Fermez et rouvrez l'application
3. Contactez le support si le problème persiste

---

## 📞 Support

### Contacter l'équipe AIRADCR

- **Site web** : [airadcr.com](https://airadcr.com)
- **Email** : contact@airadcr.com
- **Documentation** : [docs.airadcr.com](https://docs.airadcr.com)

### Informations à fournir en cas de problème

Lors d'un contact avec le support, préparez :
- Votre version de Windows
- La version de l'application (visible dans "À propos")
- Une description précise du problème
- Les étapes pour reproduire le problème
- Une capture d'écran si possible

---

## 📝 Notes de version

### Version 1.0.0 (Décembre 2024)
- ✅ Première version stable
- ✅ Intégration SpeechMike Philips
- ✅ Injection de rapports radiologiques
- ✅ Always-on-top
- ✅ Raccourcis clavier globaux
- ✅ Mises à jour automatiques

---

*Guide rédigé pour AIRADCR Desktop v1.0.0*
*Dernière mise à jour : Décembre 2024*
