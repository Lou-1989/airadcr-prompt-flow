import { useEffect, useCallback, useRef } from 'react';
import { isValidMessage, SECURITY_CONFIG } from '@/security/SecurityConfig';
import { useInjection } from './useInjection';
import { logger } from '@/utils/logger';

type MessageHandler = (data: any) => void;

// Hook pour la communication sécurisée avec l'iframe AirADCR
export const useSecureMessaging = () => {
  const { 
    performInjection, 
    lockCurrentPosition, 
    unlockPosition, 
    updateLockedPosition,
    isLocked 
  } = useInjection();
  
  // 🔒 DEBOUNCE: Protection contre injections multiples
  const lastInjectionTimeRef = useRef<number>(0);
  const INJECTION_COOLDOWN = 1000; // 1 seconde entre injections
  
  // 🔒 DEDUPLICATION: Éviter les doublons de requêtes
  const recentRequestsRef = useRef<Map<string, number>>(new Map());
  const REQUEST_DEDUP_WINDOW = 2000; // 2 secondes

  // Envoi de message sécurisé vers l'iframe (déclaré AVANT handleSecureMessage)
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
  }, []); // Pas de dépendances car utilise seulement des APIs natives
  
  // Gestionnaire de messages sécurisé
  const handleSecureMessage = useCallback((event: MessageEvent) => {
    // Validation stricte du message
    if (!isValidMessage(event)) {
      return;
    }
    
    const { type, payload } = event.data;
    logger.debug(`[Sécurisé] Message reçu: ${type}`, { origin: event.origin, payload });
    
    switch (type) {
      case 'airadcr:ready':
        logger.debug('[Sécurisé] AirADCR iframe prête');
        break;
        
      case 'airadcr:inject':
        const now = Date.now();
        
        // 🔒 DEDUPLICATION: Générer un ID unique pour cette requête
        const requestId = payload?.id || `${payload?.text?.substring(0, 20)}_${Math.floor(now / 1000)}`;
        
        // Nettoyer les anciennes entrées (> 2s)
        recentRequestsRef.current.forEach((timestamp, id) => {
          if (now - timestamp > REQUEST_DEDUP_WINDOW) {
            recentRequestsRef.current.delete(id);
          }
        });
        
        // Vérifier si c'est un doublon
        if (recentRequestsRef.current.has(requestId)) {
          const timeSinceDuplicate = now - (recentRequestsRef.current.get(requestId) || 0);
          logger.warn('[Sécurisé] Injection DUPLIQUÉE ignorée', {
            requestId,
            timeSinceDuplicate
          });
          return;
        }
        
        // Enregistrer cette requête
        recentRequestsRef.current.set(requestId, now);
        
        // 🔒 DEBOUNCE: Vérifier si cooldown actif (filet de sécurité)
        const timeSinceLastInjection = now - lastInjectionTimeRef.current;
        
        if (timeSinceLastInjection < INJECTION_COOLDOWN) {
          logger.warn('[Sécurisé] Injection ignorée (cooldown actif)', {
            timeSinceLastInjection,
            cooldown: INJECTION_COOLDOWN
          });
          // Envoyer ACK négatif
          sendSecureMessage('airadcr:injection_ack', { 
            id: requestId, 
            accepted: false, 
            reason: 'COOLDOWN_ACTIVE' 
          });
          return;
        }
        
        // ✅ ACK IMMÉDIAT: Confirmer réception pour stopper les retries
        logger.debug(`[Sécurisé] Envoi ACK pour requête ${requestId}`);
        sendSecureMessage('airadcr:injection_ack', { 
          id: requestId, 
          accepted: true 
        });
        
        if (payload && payload.text) {
          lastInjectionTimeRef.current = now;
          
          performInjection(payload.text).then(success => {
            // 📊 STATUT FINAL: Envoyer le résultat de l'injection
            const status = {
              id: requestId,
              success,
              reason: success ? 'SUCCESS' : 'UNKNOWN_ERROR',
              timestamp: Date.now()
            };
            
            logger.debug(`[Sécurisé] Envoi statut final pour ${requestId}:`, status);
            sendSecureMessage('airadcr:injection_status', status);
            
            if (success) {
              logger.debug('[Sécurisé] Injection réalisée avec succès');
            } else {
              logger.error('[Sécurisé] Échec de l\'injection');
            }
          }).catch(error => {
            // Envoyer statut d'erreur
            logger.error('[Sécurisé] Erreur lors de l\'injection:', error);
            sendSecureMessage('airadcr:injection_status', {
              id: requestId,
              success: false,
              reason: 'INJECTION_ERROR',
              error: error.message,
              timestamp: Date.now()
            });
          });
        } else {
          logger.warn('[Sécurisé] Payload d\'injection invalide');
          sendSecureMessage('airadcr:injection_status', {
            id: requestId,
            success: false,
            reason: 'INVALID_PAYLOAD',
            timestamp: Date.now()
          });
        }
        break;
        
      case 'airadcr:status':
        logger.debug('[Sécurisé] Statut AirADCR:', payload);
        break;
        
      case 'airadcr:lock':
        logger.debug('[Sécurisé] Demande de verrouillage reçue');
        lockCurrentPosition().then(success => {
          sendSecureMessage('airadcr:lock_status', { locked: success });
          if (success) {
            logger.debug('[Sécurisé] Position verrouillée avec succès');
          } else {
            logger.error('[Sécurisé] Échec du verrouillage');
          }
        });
        break;
        
      case 'airadcr:unlock':
        logger.debug('[Sécurisé] Demande de déverrouillage reçue');
        unlockPosition();
        sendSecureMessage('airadcr:lock_status', { locked: false });
        break;
        
      case 'airadcr:update_lock':
        logger.debug('[Sécurisé] Demande de mise à jour position verrouillée');
        updateLockedPosition().then(success => {
          sendSecureMessage('airadcr:lock_status', { locked: success });
        });
        break;
        
      default:
        logger.warn('[Sécurisé] Type de message non géré:', type);
    }
  }, [performInjection, lockCurrentPosition, unlockPosition, updateLockedPosition, sendSecureMessage]);
  
  // Configuration des écouteurs d'événements
  useEffect(() => {
    window.addEventListener('message', handleSecureMessage);
    
    return () => {
      window.removeEventListener('message', handleSecureMessage);
    };
  }, [handleSecureMessage]);
  
  return {
    sendSecureMessage,
    isLocked, // Exposer l'état de verrouillage pour l'interface
  };
};