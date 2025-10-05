import { useEffect, useCallback, useRef } from 'react';
import { isValidMessage, SECURITY_CONFIG } from '@/security/SecurityConfig';
import { useInjectionContext } from '@/contexts/InjectionContext';
import { logger } from '@/utils/logger';
import { invoke } from '@tauri-apps/api/tauri';

type MessageHandler = (data: any) => void;

// Hook pour la communication sécurisée avec l'iframe AirADCR
export const useSecureMessaging = () => {
  const { 
    performInjection, 
    lockCurrentPosition, 
    unlockPosition, 
    updateLockedPosition,
    isLocked 
  } = useInjectionContext();
  
  // 🔒 DEBOUNCE: Protection contre injections multiples
  const lastInjectionTimeRef = useRef<number>(0);
  const INJECTION_COOLDOWN = 1000; // 1 seconde entre injections
  
  // 🔒 DEDUPLICATION: Éviter les doublons de requêtes
  const recentRequestsRef = useRef<Map<string, number>>(new Map());
  const REQUEST_DEDUP_WINDOW = 2000; // 2 secondes
  
  // 🆕 QUEUE FIFO: Sérialisation des injections
  const injectionQueueRef = useRef<Array<{ id: string; text: string; type: string }>>([]);
  const isProcessingRef = useRef<boolean>(false);

  // 🎤 FONCTION: Notifier Tauri de l'état d'enregistrement (désormais simplifié - pas de synchro d'état)
  const notifyRecordingState = useCallback((state: 'started' | 'paused' | 'finished') => {
    const messageType = `airadcr:recording_${state}`;
    logger.debug(`[useSecureMessaging] 🎤 État enregistrement: ${messageType}`);
    
    // Note: Plus besoin d'appeler Tauri puisque DictationState est supprimé
    // La logique de dictation est 100% gérée par le frontend React
  }, []);

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
  
  // 🆕 FONCTION: Traitement séquentiel de la queue FIFO
  const processNextInjection = useCallback(() => {
    if (isProcessingRef.current || injectionQueueRef.current.length === 0) {
      return;
    }
    
    isProcessingRef.current = true;
    const item = injectionQueueRef.current.shift()!; // FIFO
    
    logger.debug(`[Queue] Traitement injection ${item.id} (reste: ${injectionQueueRef.current.length})`);
    
    performInjection(item.text, item.type)
      .then(success => {
        sendSecureMessage('airadcr:injection_status', {
          id: item.id,
          success,
          reason: success ? 'SUCCESS' : 'UNKNOWN_ERROR',
          timestamp: Date.now()
        });
      })
      .catch(error => {
        sendSecureMessage('airadcr:injection_status', {
          id: item.id,
          success: false,
          reason: 'INJECTION_ERROR',
          error: error.message,
          timestamp: Date.now()
        });
      })
      .finally(() => {
        isProcessingRef.current = false;
        logger.debug(`[Queue] Injection ${item.id} terminée, état: processing=${isProcessingRef.current}, queue=${injectionQueueRef.current.length}`);
        // Traiter le suivant après 200ms
        setTimeout(() => processNextInjection(), 200);
      });
  }, [performInjection, sendSecureMessage]);
  
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
        // Synchronisation initiale: demander le statut
        sendSecureMessage('airadcr:request_status');
        break;
      
      // 🎤 COMMANDES SPEECHMIKE: Record/Pause/Finish  
      case 'airadcr:speechmike_record':
        logger.debug('🎤 [SpeechMike] Commande Record reçue depuis Tauri');
        // Simuler le clic sur le bouton d'enregistrement AIRADCR
        // TODO: Implémenter la logique de démarrage/reprise dictée
        // Pour l'instant, juste logger
        notifyRecordingState('started');
        break;
        
      case 'airadcr:speechmike_pause':
        logger.debug('⏸️ [SpeechMike] Commande Pause reçue depuis Tauri');
        // TODO: Implémenter la logique de pause dictée
        notifyRecordingState('paused');
        break;
        
      case 'airadcr:speechmike_finish':
        logger.debug('✅ [SpeechMike] Commande Finish reçue depuis Tauri');
        // TODO: Implémenter la logique de finalisation dictée
        notifyRecordingState('finished');
        break;
        
      case 'airadcr:inject':
        const now = Date.now();
        
        // 🔒 DEDUPLICATION AMÉLIORÉE: ID unique robuste
        // Inclut type + hash du contenu + timestamp
        const contentHash = payload?.text ? 
          payload.text.substring(0, 30).replace(/\s/g, '') : '';
        const injectionType = payload?.type || 'default'; // 'brut' ou 'structuré'
        const requestId = payload?.id || 
          `${injectionType}_${contentHash}_${Math.floor(now / 100)}`; // 100ms de précision
        
        logger.debug(`[Sécurisé] 🎯 INJECTION DEMANDÉE - Type: "${injectionType}", ID: ${requestId}`);
        
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
            type: injectionType,
            timeSinceDuplicate
          });
          // Envoyer ACK négatif immédiat
          sendSecureMessage('airadcr:injection_ack', { 
            id: requestId, 
            accepted: false, 
            reason: 'DUPLICATE_REQUEST' 
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
          
          logger.debug(`[Sécurisé] 📝 Contenu à injecter (${injectionType}):`, {
            preview: payload.text.substring(0, 100) + '...',
            length: payload.text.length
          });
          
          // 🆕 EMPILER dans la queue FIFO au lieu d'appeler directement performInjection
          injectionQueueRef.current.push({
            id: requestId,
            text: payload.text,
            type: injectionType
          });
          
          logger.debug(`[Queue] Injection ${requestId} empilée (total: ${injectionQueueRef.current.length})`);
          
          // Démarrer le traitement si idle
          processNextInjection();
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
        
      case 'airadcr:request_status':
        logger.debug('[Sécurisé] Demande de statut reçue');
        sendSecureMessage('airadcr:lock_status', { locked: isLocked });
        break;
        
      default:
        logger.warn('[Sécurisé] Type de message non géré:', type);
    }
  }, [performInjection, lockCurrentPosition, unlockPosition, updateLockedPosition, sendSecureMessage, processNextInjection, notifyRecordingState]);
  
  // Configuration des écouteurs d'événements
  useEffect(() => {
    logger.debug('[Sécurité] Origines autorisées:', SECURITY_CONFIG.ALLOWED_ORIGINS);
    logger.debug('[Sécurité] Origin actuelle:', window.location.origin);
    
    window.addEventListener('message', handleSecureMessage);
    
    return () => {
      window.removeEventListener('message', handleSecureMessage);
    };
  }, [handleSecureMessage]);
  
  return {
    sendSecureMessage,
    notifyRecordingState, // Exposer pour utilisation externe
    isLocked, // Exposer l'état de verrouillage pour l'interface
  };
};