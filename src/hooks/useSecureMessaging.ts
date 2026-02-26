import { useEffect, useCallback, useRef } from 'react';
import { isValidMessage, SECURITY_CONFIG } from '@/security/SecurityConfig';
import { useInjectionContext } from '@/contexts/InjectionContext';
import { logger } from '@/utils/logger';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

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
  const injectionQueueRef = useRef<Array<{ id: string; text: string; type: string; html?: string }>>([]);
  const isProcessingRef = useRef<boolean>(false);

  // 🎤 FONCTION: Notifier Tauri de l'état d'enregistrement + contrôle LED SpeechMike
  const notifyRecordingState = useCallback((state: 'started' | 'paused' | 'finished') => {
    logger.debug(`[useSecureMessaging] 🎤 État enregistrement: ${state}`);
    
    // Contrôle LED natif du SpeechMike
    const ledMap: Record<string, string> = {
      started: 'recording',  // Rouge fixe
      paused: 'pause',       // Rouge clignotant
      finished: 'idle',      // Vert fixe
    };
    invoke('speechmike_set_led', { ledState: ledMap[state] }).catch(() => {
      // Silencieux si pas de SpeechMike connecté
    });
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
      const timestamp = new Date().toISOString();
      iframe.contentWindow.postMessage(
        { type, payload },
        'https://airadcr.com'
      );
      logger.debug(`[Sécurisé] ✉️ postMessage envoyé @ ${timestamp}: ${type}`, payload || '(pas de payload)');
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
    
    performInjection(item.text, item.type, item.html)
      .then(success => {
        sendSecureMessage('airadcr:injection_status', {
          id: item.id,
          success,
          reason: success ? 'SUCCESS' : 'UNKNOWN_ERROR',
          timestamp: Date.now()
        });
        
        // 🧹 Nettoyage SQLite post-injection réussie: libérer le pipeline
        if (success) {
          const iframe = document.querySelector('iframe[title="AirADCR"]') as HTMLIFrameElement;
          const tidMatch = iframe?.src?.match(/[?&]tid=([^&]+)/);
          const tid = tidMatch ? decodeURIComponent(tidMatch[1]) : null;
          if (tid) {
            logger.debug(`[Pipeline] 🧹 Suppression rapport ${tid} de SQLite après injection réussie`);
            invoke('delete_pending_report_cmd', { technicalId: tid }).catch(err => {
              logger.warn('[Pipeline] Échec suppression post-injection (cleanup auto prendra le relais):', err);
            });
          }
        }
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
            type: injectionType,
            html: payload.html || undefined
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
      
      // 🎤 LED SpeechMike: feedback visuel selon l'état d'enregistrement
      case 'airadcr:recording_started':
        logger.debug('[Sécurisé] 🔴 Enregistrement démarré → LED rouge fixe');
        notifyRecordingState('started');
        break;
        
      case 'airadcr:recording_paused':
        logger.debug('[Sécurisé] ⏸️ Enregistrement en pause → LED rouge clignotant');
        notifyRecordingState('paused');
        break;
        
      case 'airadcr:recording_finished':
        logger.debug('[Sécurisé] ✅ Enregistrement terminé → LED vert fixe');
        notifyRecordingState('finished');
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
  
  // 🎤 ÉCOUTE DES ÉVÉNEMENTS TAURI (raccourcis clavier globaux + SpeechMike natif)
  useEffect(() => {
    const listeners: UnlistenFn[] = [];
    
    // 🎤 DICTATION: Ctrl+Shift+D (Start/Stop dictée) — aussi déclenché par SpeechMike natif (bouton Record)
    listen('airadcr:dictation_startstop', () => {
      logger.debug('[Tauri Event] 🔴 Start/Stop dictée (shortcut ou SpeechMike natif)');
      sendSecureMessage('airadcr:toggle_recording');
    }).then(unlisten => listeners.push(unlisten));
    
    // 🎤 DICTATION: Ctrl+Shift+P (Pause/Resume dictée) — aussi déclenché par SpeechMike natif (bouton Stop/Play)
    listen('airadcr:dictation_pause', () => {
      logger.debug('[Tauri Event] ⏯️ Pause/Resume dictée (shortcut ou SpeechMike natif)');
      sendSecureMessage('airadcr:toggle_pause');
    }).then(unlisten => listeners.push(unlisten));
    
    // 💉 INJECTION: Ctrl+Shift+T (Inject texte brut) — aussi déclenché par SpeechMike natif (bouton Instruction)
    listen('airadcr:inject_raw', () => {
      logger.debug('[Tauri Event] 💉 Inject texte brut (shortcut ou SpeechMike natif)');
      sendSecureMessage('airadcr:request_injection', { type: 'brut' });
    }).then(unlisten => listeners.push(unlisten));
    
    // 💉 INJECTION: Ctrl+Shift+S (Inject rapport structuré) — aussi déclenché par SpeechMike natif (bouton F1/EOL)
    listen('airadcr:inject_structured', () => {
      logger.debug('[Tauri Event] 📋 Inject rapport structuré (shortcut ou SpeechMike natif)');
      sendSecureMessage('airadcr:request_injection', { type: 'structuré' });
    }).then(unlisten => listeners.push(unlisten));
    
    // 🎤 SPEECHMIKE NATIF: Périphérique connecté
    listen('airadcr:speechmike_connected', (event) => {
      logger.debug('[SpeechMike Natif] ✅ Périphérique connecté:', event.payload);
    }).then(unlisten => listeners.push(unlisten));
    
    // 🎤 SPEECHMIKE NATIF: Périphérique déconnecté
    listen('airadcr:speechmike_disconnected', () => {
      logger.debug('[SpeechMike Natif] ❌ Périphérique déconnecté');
    }).then(unlisten => listeners.push(unlisten));
    
    return () => {
      listeners.forEach(unlisten => unlisten());
    };
  }, [sendSecureMessage]);
  
  return {
    sendSecureMessage,
    notifyRecordingState, // Exposer pour utilisation externe
    isLocked, // Exposer l'état de verrouillage pour l'interface
  };
};