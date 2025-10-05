# 📋 Guide d'Injection AirADCR Desktop

## 🎯 Principe de fonctionnement

L'application AirADCR Desktop utilise le **comportement natif de Windows** pour injecter du texte dans vos logiciels (RIS, Word, etc.). Elle simule un utilisateur qui :
1. Clique à une position précise
2. Appuie sur `Ctrl+V` pour coller du texte

Cette approche est **simple, fiable et compatible avec tous les logiciels Windows**.

---

## 🔄 Comment remplacer du texte

### ✅ Méthode recommandée (Comportement Windows natif)

1. **Dans votre RIS/Word** : Sélectionnez manuellement le texte à remplacer
2. **Dans AirADCR** : Générez votre rapport radiologique
3. **Cliquez sur "Injecter"** ou utilisez le raccourci

➡️ **Résultat** : Le texte sélectionné est **automatiquement remplacé** par le nouveau rapport (comportement natif de `Ctrl+V` sur une sélection).

### 📍 Modes d'injection disponibles

#### Mode 1 : Position libre (par défaut)
- L'injection se fait à la **dernière position du curseur** capturée hors de l'application
- Idéal pour une utilisation rapide sans verrouillage

#### Mode 2 : Position verrouillée 🔒
- Cliquez sur "Verrouiller position" après avoir positionné votre curseur
- L'injection se fera **toujours au même endroit**, même si vous déplacez la fenêtre
- Idéal pour un workflow répétitif (ex: toujours injecter dans le champ "Conclusion")

---

## 🆚 Différences avec Dragon NaturallySpeaking

| Caractéristique | AirADCR Desktop | Dragon Professional |
|-----------------|-----------------|---------------------|
| **Mode correction** | ❌ Non supporté | ✅ Sélection automatique du dernier texte dicté |
| **Remplacement texte** | ✅ Sélection manuelle + Ctrl+V | ✅ Intelligence artificielle de contexte |
| **Compatibilité logiciels** | ✅ 100% des logiciels Windows | ⚠️ Dépend des hooks logiciels |
| **Simplicité** | ✅ Comportement natif Windows | ⚠️ Complexe (COM API) |
| **Coût** | ✅ Gratuit | 💰 ~300-500€ + maintenance |

### ⚡ Pourquoi pas de "mode correction" automatique ?

Dragon utilise une **API COM propriétaire** qui lui permet de :
- Suivre l'historique de dictée en temps réel
- Sélectionner automatiquement le dernier texte dicté
- Proposer des corrections contextuelles

**Implémenter cela nécessiterait** :
- License Dragon SDK (~10 000€+)
- Intégration COM complexe (6-12 mois de développement)
- Maintenance continue pour chaque nouvelle version de Dragon

➡️ **Notre choix** : Utiliser le comportement natif Windows (sélection manuelle + Ctrl+V) qui est :
- ✅ **Fiable** : Fonctionne dans 100% des logiciels
- ✅ **Simple** : Pas de dépendance externe
- ✅ **Rapide** : Injection en <100ms
- ✅ **Gratuit** : Pas de licence tierce

---

## 🐛 Résolution de problèmes courants

### ❌ "Aucune position capturée"
**Cause** : Le curseur n'a pas été détecté hors de l'application AirADCR.

**Solution** :
1. Cliquez dans votre RIS/Word pour placer le curseur
2. Attendez 1 seconde (capture automatique)
3. Revenez dans AirADCR et injectez

### ❌ "Injection à la mauvaise position"
**Cause** : La dernière position capturée ne correspond pas à l'endroit souhaité.

**Solution** :
1. Activez le **mode verrouillé** 🔒
2. Placez votre curseur exactement où vous voulez injecter
3. Cliquez sur "Verrouiller position"
4. Toutes les injections iront maintenant à cet endroit

### ❌ "Le texte s'ajoute au lieu de remplacer"
**Cause** : Aucun texte n'est sélectionné dans le logiciel cible.

**Solution** :
1. **Sélectionnez manuellement** le texte à remplacer dans votre RIS/Word
2. Puis cliquez sur "Injecter"
3. Le texte sélectionné sera remplacé automatiquement par `Ctrl+V`

### ❌ "L'injection ne fonctionne pas du tout"
**Cause** : Problème de focus fenêtre ou permissions Windows.

**Solution** :
1. Vérifiez que votre RIS/Word n'est **pas en mode administrateur** (conflit permissions)
2. Essayez de **cliquer manuellement** dans le champ avant d'injecter
3. Consultez les logs (bouton "Ouvrir logs" dans l'application)

---

## 🎓 Workflow recommandé

### Pour une utilisation occasionnelle :
```
1. Ouvrir RIS/Word
2. Cliquer dans le champ à remplir
3. Générer le rapport dans AirADCR
4. Cliquer sur "Injecter"
```

### Pour une utilisation intensive (répétitive) :
```
1. Ouvrir RIS/Word
2. Cliquer dans le champ "Conclusion" (ou autre champ cible)
3. Dans AirADCR : Cliquer "Verrouiller position" 🔒
4. Pour chaque patient :
   - Générer le rapport
   - Cliquer "Injecter" (toujours au même endroit)
```

### Pour remplacer un ancien rapport :
```
1. Dans RIS/Word : Sélectionner l'ancien texte (Ctrl+A ou souris)
2. Dans AirADCR : Générer le nouveau rapport
3. Cliquer "Injecter"
→ L'ancien texte est remplacé automatiquement
```

---

## 🚀 Optimisations futures possibles

### Phase 2 : Mode "Effacer avant injection" (Planifié)
- **Ajout d'un toggle** dans l'interface : "Effacer le champ avant injection"
- **Comportement** : Envoie automatiquement `Ctrl+A` avant `Ctrl+V`
- **Avantage** : Pas besoin de sélectionner manuellement si le champ entier doit être remplacé
- **Risque** : Plus destructif (efface TOUT le champ, même s'il contient plus que prévu)

### Phase 3 : Historique d'injection (Avancé)
- **Principe** : L'application garde en mémoire les 10 derniers textes injectés
- **Avantage** : Permet de "ré-injecter" ou "corriger" le dernier texte
- **Limite** : Ne fonctionne que si l'utilisateur ne modifie pas le texte entre temps

### Phase 4 : Intégration Dragon SDK (Professionnel)
- **Coût estimé** : 15 000€+ (licence SDK + développement)
- **Délai** : 6-12 mois
- **Avantage** : Mode correction automatique comme Dragon Professional
- **Recommandation** : Uniquement si budget >50K€ et demande explicite des clients

---

## 📞 Support

Pour toute question ou problème :
1. **Consulter les logs** : Bouton "Ouvrir logs" dans l'application
2. **Vérifier ce guide** : La plupart des problèmes ont une solution simple
3. **Contacter le support** : Avec les logs et une description du problème

---

**Version du guide** : 1.0 (2025-10-05)  
**Application** : AirADCR Desktop v1.0.0  
**Compatibilité** : Windows 10/11 (64-bit)
