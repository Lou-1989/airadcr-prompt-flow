# 📋 Documentation Technique - Communication airadcr.com → Desktop Tauri

> **⚠️ RÉFÉRENCE OBLIGATOIRE** : Consulter ce document avant toute modification du système de communication

## 🎯 Vue d'ensemble du système

L'application **airadcr.com** utilise un système de communication centralisé via `postMessage` pour interagir avec une application desktop parent (future application Tauri). L'architecture suit le pattern iframe enfant → parent desktop.

---

## 🏗️ Architecture du système de communication

### **DesktopCommunicator** - Classe singleton
**Fichier :** `src/lib/desktop-communicator.ts`

```typescript
export class DesktopCommunicator {
  private static instance: DesktopCommunicator | null = null;
  private listeners: Set<(isLocked: boolean) => void> = new Set();
  private isLocked: boolean = false;
  private isInitialized: boolean = false;
}
```

**Méthodes principales :**
- `getInstance()` - Pattern singleton
- `initialize()` - Initialisation automatique au montage
- `sendToDesktop(type, payload)` - Envoi générique via `parent.postMessage()`
- `setupMessageListener()` - Écoute des réponses du desktop

---

## 📡 Messages envoyés vers le desktop parent

### 1. **Messages d'injection de rapport**
```javascript
// Type: 'airadcr:inject'
{
  type: 'airadcr:inject',
  payload: { 
    text: "**Titre :** Scanner thoracique\n\n**Indication :** Dyspnée..." 
  }
}
```

### 2. **Messages de verrouillage**
```javascript
// Verrouiller la position
{ type: 'airadcr:lock' }

// Déverrouiller la position  
{ type: 'airadcr:unlock' }

// Demander le statut actuel
{ type: 'airadcr:request_status' }
```

### 3. **Message de test**
```javascript
// Pour debug/test de communication
{
  type: 'airadcr:test',
  payload: { timestamp: 1634567890123 }
}
```

---

## 📥 Messages reçus du desktop parent

### **Statut de verrouillage**
```javascript
// Le desktop envoie ce message en réponse
{
  type: 'airadcr:lock_status',
  payload: { locked: true } // ou false
}
```

---

## 🔘 Analyse détaillée des boutons

### **1. Bouton de verrouillage**
**Localisation :** `src/components/structured-editor/StructuredEditorHeader.tsx`

```typescript
// Hook utilisé
const { isLocked, toggleLock } = useDesktopCommunicator();

// Gestionnaire de clic
<button onClick={toggleLock}>
  {isLocked ? <Lock /> : <Unlock />}
</button>
```

**Comportement :**
- **Clic sur verrouiller** → Envoie `{ type: 'airadcr:lock' }`
- **Clic sur déverrouiller** → Envoie `{ type: 'airadcr:unlock' }`
- **État visuel** : Rouge quand verrouillé, gris quand déverrouillé
- **Animations** : `animate-scale-in` lors du changement d'état

### **2. Bouton d'injection - Dictée**
**Localisation :** `src/components/dictation-recorder/DictationInput.tsx` (ligne ~202)

```typescript
const handleInjectReport = useCallback(() => {
  // Formatage du texte pour l'injection
  const formattedForInject = dictationText
    .replace(/\*\*(.*?):\*\*/g, '$1 :') // Titres sans markdown
    .replace(/\*\*(.*?)\*\*/g, '$1')    // Texte en gras sans markdown
    .replace(/\n\n/g, '\n\n')           // Conserver les double retours
    .replace(/^\s*-\s+/gm, '• ')        // Convertir les listes en puces
    .trim();

  injectReport(formattedForInject);
}, [dictationText, injectReport, toast]);
```

**Caractéristiques :**
- **Contenu** : Texte de dictée brut formaté
- **Formatage** : Suppression du markdown, conversion des listes
- **Message envoyé** : `{ type: 'airadcr:inject', payload: { text: formattedText } }`

### **3. Bouton d'injection - Rapport structuré**
**Localisation :** `src/components/ExportPanel.tsx` (ligne ~97)

```typescript
const handleInjectReport = useCallback(async () => {
  const reportData = {
    title,
    indication: structuredReport.indication,
    technique: structuredReport.technique,
    results: structuredReport.results,
    conclusion: structuredReport.conclusion,
    references: structuredReport.references || [],
    translation: translation || structuredReport.translation,
    explanation: explanation || structuredReport.explanation
  };

  const customContent = generateCustomContent(reportData);
  injectReport(customContent);
}, [isReportValid, title, structuredReport, ...]);
```

**Caractéristiques :**
- **Contenu** : Rapport structuré selon les préférences utilisateur
- **Personnalisation** : Utilise `useCopyPreferences()` pour sélectionner les sections
- **Contenu intelligent** : Inclut titre, indication, technique, résultats, conclusion, références, traduction, explication selon les préférences

---

## 🔗 Intégration avec React

### **Hook useDesktopCommunicator**
**Fichier :** `src/hooks/useDesktopCommunicator.tsx`

```typescript
export const useDesktopCommunicator = () => {
  const [isLocked, setIsLocked] = useState(false);
  const [communicator] = useState(() => DesktopCommunicator.getInstance());

  return {
    isLocked,           // État de verrouillage
    injectReport,       // Fonction d'injection
    lockPosition,       // Verrouillage
    unlockPosition,     // Déverrouillage  
    toggleLock,         // Basculer l'état
    requestLockStatus,  // Demander le statut
    testCommunication   // Test debug
  };
};
```

### **Initialisation automatique**
**Fichier :** `src/App.tsx`
```typescript
useEffect(() => {
  // Initialiser le communicateur desktop au montage de l'app
  const communicator = DesktopCommunicator.getInstance();
  communicator.initialize();
}, []);
```

---

## 🛠️ Implémentation recommandée côté Tauri

### **Structure suggérée pour l'application desktop**
```javascript
class AIRadcrDesktopHandler {
  constructor() {
    this.isLocked = false;
    this.setupIframeListener();
  }

  setupIframeListener() {
    window.addEventListener('message', (event) => {
      const { type, payload } = event.data;

      switch(type) {
        case 'airadcr:inject':
          this.handleInjectReport(payload.text);
          break;
        case 'airadcr:lock':
          this.handleLockPosition();
          break;
        case 'airadcr:unlock':
          this.handleUnlockPosition();
          break;
        case 'airadcr:request_status':
          this.sendLockStatus();
          break;
        case 'airadcr:test':
          console.log('Test communication reçu:', payload);
          break;
      }
    });
  }

  handleInjectReport(text) {
    // Injecter le texte dans le champ actif ou target
    const activeElement = document.activeElement;
    if (activeElement && (activeElement.tagName === 'TEXTAREA' || activeElement.tagName === 'INPUT')) {
      activeElement.value = text;
      activeElement.dispatchEvent(new Event('input', { bubbles: true }));
    }
  }

  handleLockPosition() {
    this.isLocked = true;
    // Logique de verrouillage de la fenêtre (always on top, etc.)
    this.sendLockStatus();
  }

  handleUnlockPosition() {
    this.isLocked = false;
    // Logique de déverrouillage
    this.sendLockStatus();
  }

  sendLockStatus() {
    const iframe = document.querySelector('#airadcr-iframe');
    if (iframe && iframe.contentWindow) {
      iframe.contentWindow.postMessage({
        type: 'airadcr:lock_status',
        payload: { locked: this.isLocked }
      }, '*');
    }
  }
}
```

---

## 🔍 Debug et monitoring

### **Logs automatiques**
Tous les messages sont automatiquement loggés :
```javascript
console.log('[DesktopCommunicator] Sending to desktop:', message);
console.log('[DesktopCommunicator] Received lock status:', locked);
```

### **Fonction de test**
```javascript
// Depuis la console du navigateur
DesktopCommunicator.getInstance().testCommunication();
```

---

## 🚀 Flux de communication type

### **Scénario 1 : Injection d'un rapport**
1. **Utilisateur** clique sur "Injecter le rapport" 
2. **airadcr.com** → `{ type: 'airadcr:inject', payload: { text: "..." } }`
3. **Desktop Tauri** reçoit le message et injecte dans le champ actif
4. **Feedback visuel** : Toast de confirmation dans airadcr.com

### **Scénario 2 : Verrouillage de position**
1. **Utilisateur** clique sur le cadenas
2. **airadcr.com** → `{ type: 'airadcr:lock' }`
3. **Desktop Tauri** verrouille la position et répond `{ type: 'airadcr:lock_status', payload: { locked: true } }`
4. **airadcr.com** met à jour l'icône (rouge + Lock icon)

---

## ✅ Points clés pour l'intégration Tauri

1. **Direction correcte** : `parent.postMessage()` depuis l'iframe enfant
2. **Types de messages standardisés** : `airadcr:inject`, `airadcr:lock`, `airadcr:unlock`
3. **Gestion d'état bidirectionnelle** : Tauri doit confirmer les changements d'état
4. **Injection intelligente** : Deux types de contenu (dictée brute vs rapport structuré)
5. **Always-on-top** : Le verrouillage doit maintenir la fenêtre au premier plan
6. **Cible d'injection** : Détecter le champ actif ou avoir un champ cible défini

---

## 📌 Architecture actuelle du projet Tauri

### **Fichiers clés de communication**
- `src/hooks/useSecureMessaging.ts` - Gestion sécurisée des messages iframe
- `src/hooks/useInjection.ts` - Logique d'injection de texte
- `src/hooks/useClipboardBridge.ts` - Pont clipboard pour contourner les limites postMessage
- `src/security/SecurityConfig.ts` - Configuration de sécurité (CSP, sandbox, origins)
- `src-tauri/src/main.rs` - Backend Rust avec commandes Tauri

### **Types de messages actuellement gérés**
```typescript
type AllowedMessageType = 
  | 'airadcr:ready'
  | 'airadcr:inject'
  | 'airadcr:lock'
  | 'airadcr:unlock'
  | 'airadcr:update_lock'
  | 'airadcr:lock_status'
  | 'airadcr:status'
  | 'airadcr:test';
```

---

Cette documentation fournit tous les éléments nécessaires pour implémenter et maintenir la communication entre airadcr.com et l'application desktop Tauri. Le système est robuste, centralisé et prêt pour la production.
