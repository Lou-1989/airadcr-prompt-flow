import { useEffect, useCallback } from 'react';
import { isValidMessage, SECURITY_CONFIG } from '@/security/SecurityConfig';
import { useInjection } from './useInjection';
import { logger } from '@/utils/logger';

type MessageHandler = (data: any) => void;

// Hook pour la communication sécurisée avec l'iframe AirADCR
export const useSecureMessaging = () => {
  const { performInjection } = useInjection();
  // Gestionnaire de messages sécurisé
  const handleSecureMessage = useCallback((event: MessageEvent) => {
    // Validation stricte du message
    if (!isValidMessage(event)) {
      return;
    }
    
    const { type, payload } = event.data;
    
    switch (type) {
      case 'airadcr:ready':
        logger.debug('[Sécurisé] AirADCR iframe prête');
        break;
        
      case 'airadcr:inject':
        logger.debug('[Sécurisé] Demande d\'injection reçue:', payload);
        if (payload && payload.text) {
          performInjection(payload.text).then(success => {
            if (success) {
              logger.debug('[Sécurisé] Injection réalisée avec succès');
            } else {
              logger.error('[Sécurisé] Échec de l\'injection');
            }
          });
        } else {
          logger.warn('[Sécurisé] Payload d\'injection invalide');
        }
        break;
        
      case 'airadcr:status':
        logger.debug('[Sécurisé] Statut AirADCR:', payload);
        break;
        
      default:
        logger.warn('[Sécurisé] Type de message non géré:', type);
    }
  }, []);
  
  // Envoi de message sécurisé vers l'iframe
  const sendSecureMessage = useCallback((type: string, payload?: any) => {
    const iframe = document.querySelector('iframe[title="AirADCR"]') as HTMLIFrameElement;
    
    if (!iframe || !iframe.contentWindow) {
      logger.error('[Sécurisé] Iframe AirADCR non trouvée');
      return false;
    }
    
    // Validation du type de message
    if (!SECURITY_CONFIG.ALLOWED_MESSAGE_TYPES.includes(type as any)) {
      logger.error('[Sécurisé] Type de message non autorisé:', type);
      return false;
    }
    
    try {
      iframe.contentWindow.postMessage(
        { type, payload },
        'https://airadcr.com'
      );
      return true;
    } catch (error) {
      logger.error('[Sécurisé] Erreur envoi message:', error);
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