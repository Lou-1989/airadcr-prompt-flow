# 🛡️ Guide d'Installation AIRADCR Desktop

## À propos d'AIRADCR

**AIRADCR** (https://airadcr.com) est une application médicale professionnelle développée pour les radiologues par **Dr Lounes BENSID**. Elle permet d'améliorer la qualité et la structure des comptes rendus radiologiques grâce à une interface de dictée intelligente.

---

## ⚠️ Avertissement Windows Defender (Faux Positif)

### Pourquoi Windows bloque-t-il AIRADCR ?

AIRADCR utilise des **API Windows légitimes** pour injecter du texte médical dans vos logiciels RIS/PACS/Word. Ces mêmes techniques sont parfois utilisées par des malwares, d'où la détection par Windows Defender.

**C'est un FAUX POSITIF confirmé** : notre code est open-source et auditable sur GitHub.

---

## ✅ Solution 1 : Installer via le fichier .msi (RECOMMANDÉ)

Le fichier `.msi` est moins suspect que le `.exe` portable :

1. Télécharger **AIRADCR-1.0.0-setup.msi** depuis https://airadcr.com/download
2. **Clic droit** sur le fichier → **Installer**
3. Si Windows SmartScreen bloque :
   - Cliquer sur **"Informations complémentaires"**
   - Puis **"Exécuter quand même"**
4. Suivre l'assistant d'installation

---

## ✅ Solution 2 : Autoriser manuellement dans Windows Defender

### Étape 1 : Ajouter une exclusion de dossier

1. Ouvrir **Windows Security** (icône bouclier dans la barre des tâches)
2. Aller dans **Protection contre virus et menaces**
3. Cliquer sur **Gérer les paramètres**
4. Descendre jusqu'à **Exclusions**
5. Cliquer sur **Ajouter ou supprimer des exclusions**
6. Cliquer sur **Ajouter une exclusion** → **Dossier**
7. Sélectionner le dossier d'installation d'AIRADCR (généralement `C:\Program Files\AIRADCR\`)

### Étape 2 : Vérifier l'origine du fichier

1. **Clic droit** sur le fichier AIRADCR.exe
2. Sélectionner **Propriétés**
3. Onglet **Détails** : Vérifier que le **copyright** mentionne "AIRADCR - https://airadcr.com"
4. Onglet **Signatures numériques** : Vérifier la signature (disponible prochainement)

---

## 🔒 Pourquoi AIRADCR a besoin de ces permissions ?

AIRADCR utilise des API Windows de bas niveau pour :

- ✅ **Injecter du texte** dans vos logiciels médicaux (RIS, PACS, Word)
- ✅ **Gérer les multi-écrans** pour les workflows de radiologie
- ✅ **Restaurer les fenêtres minimisées** automatiquement
- ✅ **Maintenir la fenêtre au premier plan** (always-on-top)

**Ces fonctionnalités sont essentielles** pour un workflow médical optimisé.

---

## 🆘 Support

- **Site officiel** : https://airadcr.com
- **Documentation** : https://docs.airadcr.com
- **GitHub** : https://github.com/airadcr/desktop
- **Contact** : contact@airadcr.com

---

## 🔐 Sécurité et Confidentialité

- ✅ Code open-source auditable sur GitHub
- ✅ Aucune donnée médicale n'est envoyée sur Internet
- ✅ Tout le traitement se fait localement sur votre machine
- ✅ Conforme RGPD pour une utilisation médicale
- ✅ Certification de code à venir (en cours d'acquisition)

---

**Développé par Dr Lounes BENSID - AIRADCR © 2025**
