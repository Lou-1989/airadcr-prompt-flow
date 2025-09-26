import { useEffect, useCallback } from 'react';
import { isValidMessage, SECURITY_CONFIG } from '@/security/SecurityConfig';

type MessageHandler = (data: any) => void;

// Hook pour la communication sécurisée avec l'iframe AirADCR
export const useSecureMessaging = () => {
  // Gestionnaire de messages sécurisé
  const handleSecureMessage = useCallback((event: MessageEvent) => {
    // Validation stricte du message
    if (!isValidMessage(event)) {
      return;
    }
    
    const { type, payload } = event.data;
    
    switch (type) {
      case 'airadcr:ready':
        console.log('[Sécurisé] AirADCR iframe prête');
        break;
        
      case 'airadcr:inject':
        console.log('[Sécurisé] Demande d\'injection reçue:', payload);
        // Ici, on traiterait la demande d'injection de manière sécurisée
        break;
        
      case 'airadcr:status':
        console.log('[Sécurisé] Statut AirADCR:', payload);
        break;
        
      default:
        console.warn('[Sécurisé] Type de message non géré:', type);
    }
  }, []);
  
  // Envoi de message sécurisé vers l'iframe
  const sendSecureMessage = useCallback((type: string, payload?: any) => {
    const iframe = document.querySelector('iframe[title="AirADCR"]') as HTMLIFrameElement;
    
    if (!iframe || !iframe.contentWindow) {
      console.error('[Sécurisé] Iframe AirADCR non trouvée');
      return false;
    }
    
    // Validation du type de message
    if (!SECURITY_CONFIG.ALLOWED_MESSAGE_TYPES.includes(type as any)) {
      console.error('[Sécurisé] Type de message non autorisé:', type);
      return false;
    }
    
    try {
      iframe.contentWindow.postMessage(
        { type, payload },
        'https://airadcr.com'
      );
      return true;
    } catch (error) {
      console.error('[Sécurisé] Erreur envoi message:', error);
      return false;
    }
  }, []);
  
  // Configuration des écouteurs d'événements
  useEffect(() => {
    window.addEventListener('message', handleSecureMessage);
    
    return () => {
      window.removeEventListener('message', handleSecureMessage);
    };
  }, [handleSecureMessage]);
  
  return {
    sendSecureMessage,
  };
};