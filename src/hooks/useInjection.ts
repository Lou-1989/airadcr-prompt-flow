import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { logger } from '@/utils/logger';

interface CursorPosition {
  x: number;
  y: number;
  timestamp: number;
}

interface LockedPosition extends CursorPosition {
  application: string;
}

// Hook spécialisé pour la gestion de l'injection sécurisée avec capture préventive
export const useInjection = () => {
  const [externalPositions, setExternalPositions] = useState<CursorPosition[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [lockedPosition, setLockedPosition] = useState<LockedPosition | null>(null);
  const [isLocked, setIsLocked] = useState(false);
  const [isInjecting, setIsInjecting] = useState(false);
  const [injectionQueue, setInjectionQueue] = useState<string[]>([]);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  
  // Fonction pour vérifier si l'app a le focus
  const checkAppFocus = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>('check_app_focus');
    } catch (error) {
      logger.warn('[Focus] Erreur vérification focus:', error);
      return true; // Par défaut, considérer qu'on a le focus en cas d'erreur
    }
  }, []);

  // Fonction pour obtenir la position actuelle du curseur
  const getCursorPosition = useCallback(async (): Promise<{ x: number; y: number } | null> => {
    try {
      const position = await invoke<{ x: number; y: number; timestamp: number }>('get_cursor_position');
      return { x: position.x, y: position.y };
    } catch (error) {
      logger.error('[Injection] Erreur récupération position curseur:', error);
      return null;
    }
  }, []);
  
  // Fonction pour capturer la position externe
  const captureExternalPosition = useCallback(async () => {
    try {
      // ✅ SOLUTION: Capturer uniquement quand l'app N'A PAS le focus
      const hasFocus = await checkAppFocus();
      
      if (hasFocus) {
        // Ne pas capturer si on est dans AirADCR
        return;
      }
      
      const position = await getCursorPosition();
      if (position) {
        const newPosition: CursorPosition = {
          ...position,
          timestamp: Date.now()
        };
        
        setExternalPositions(prev => {
          const updated = [newPosition, ...prev.slice(0, 2)]; // Garder les 3 dernières
          logger.debug('[Monitoring] Position EXTERNE capturée:', newPosition);
          return updated;
        });
      }
    } catch (error) {
      logger.warn('[Monitoring] Erreur capture position:', error);
    }
  }, [getCursorPosition, checkAppFocus]);
  
  // Démarrer/arrêter la surveillance
  const startMonitoring = useCallback(() => {
    if (intervalRef.current) return;
    
    logger.debug('[Monitoring] Démarrage surveillance positions externes...');
    setIsMonitoring(true);
    
    intervalRef.current = setInterval(captureExternalPosition, 500);
  }, [captureExternalPosition]);
  
  const stopMonitoring = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
      setIsMonitoring(false);
      logger.debug('[Monitoring] Arrêt surveillance positions externes');
    }
  }, []);
  
  // Démarrer automatiquement la surveillance
  useEffect(() => {
    startMonitoring();
    return () => stopMonitoring();
  }, [startMonitoring, stopMonitoring]);
  
  // getCursorPosition déjà défini plus haut
  
  // 🔒 FONCTION PRINCIPALE: Injection sécurisée avec protections critiques
  const performInjection = useCallback(async (text: string): Promise<boolean> => {
    // 🔒 PROTECTION: Bloquer si injection en cours
    if (isInjecting) {
      logger.warn('[Injection] Injection en cours, ajout à la queue');
      setInjectionQueue(prev => [...prev, text]);
      return false;
    }
    
    if (!text || text.trim().length === 0) {
      logger.warn('[Injection] Texte vide, injection annulée');
      return false;
    }
    
    setIsInjecting(true);
    stopMonitoring(); // ✅ Arrêter la capture pendant l'injection
    
    logger.debug('=== DÉBUT INJECTION ===');
    logger.debug('[Injection] Texte à injecter:', text.substring(0, 50) + '...');
    logger.debug('[Injection] Position verrouillée disponible:', !!lockedPosition);
    logger.debug('[Injection] Positions externes disponibles:', externalPositions.length);
    logger.debug('[Injection] Statut verrouillage:', isLocked);
    
    // Variable pour tracer la raison d'échec
    let failureReason = 'UNKNOWN_ERROR';
    
    try {
      // ⏱️ TIMEOUT: Maximum 5 secondes pour l'injection
      const timeoutPromise = new Promise<boolean>((_, reject) => 
        setTimeout(() => {
          failureReason = 'TIMEOUT';
          reject(new Error('Injection timeout (5s)'));
        }, 5000)
      );
      
      const injectionPromise = (async () => {
        // 🔓 ALWAYS-ON-TOP: Récupérer l'état actuel et désactiver temporairement
        let previousAlwaysOnTop = false;
        try {
          previousAlwaysOnTop = await invoke<boolean>('get_always_on_top_status');
          logger.debug(`[Injection] État always-on-top actuel: ${previousAlwaysOnTop}`);
          
          if (previousAlwaysOnTop) {
            await invoke('set_always_on_top', { alwaysOnTop: false });
            logger.debug('[Injection] Always-on-top DÉSACTIVÉ temporairement');
          }
        } catch (error) {
          logger.warn('[Injection] Impossible de gérer always-on-top:', error);
        }
        
        try {
          // PRIORITÉ 1: Position verrouillée si active
          if (isLocked && lockedPosition) {
            const age = Date.now() - lockedPosition.timestamp;
            logger.debug(`[Injection] Utilisation position verrouillée: (${lockedPosition.x}, ${lockedPosition.y}) - App: ${lockedPosition.application} - Âge: ${age}ms`);
            
            await invoke('perform_injection_at_position', {
              text,
              x: lockedPosition.x,
              y: lockedPosition.y
            });
            
            logger.debug(`=== INJECTION RÉUSSIE (verrouillée) à (${lockedPosition.x}, ${lockedPosition.y}) ===`);
            return true;
          }
          
          // PRIORITÉ 2: Utiliser la dernière position externe si disponible ET récente
          const lastExternalPosition = externalPositions[0];
          
          if (lastExternalPosition) {
            const age = Date.now() - lastExternalPosition.timestamp;
            const isPositionRecent = age < 30000; // Max 30 secondes
            
            logger.debug(`[Injection] Dernière position externe: (${lastExternalPosition.x}, ${lastExternalPosition.y}) - Âge: ${age}ms - Valide: ${isPositionRecent}`);
            
            if (isPositionRecent) {
              await invoke('perform_injection_at_position', {
                text,
                x: lastExternalPosition.x,
                y: lastExternalPosition.y
              });
              
              logger.debug(`=== INJECTION RÉUSSIE (externe) à (${lastExternalPosition.x}, ${lastExternalPosition.y}) ===`);
              return true;
            } else {
              failureReason = 'POSITION_TOO_OLD';
              logger.error(`[Injection] ❌ Position externe trop ancienne (${age}ms). Cliquez d'abord dans RIS/Word puis réessayez.`);
              return false;
            }
          } else {
            failureReason = 'NO_EXTERNAL_POSITION';
            logger.error('[Injection] ❌ Aucune position externe capturée. Cliquez d\'abord dans RIS/Word puis réessayez.');
            return false;
          }
        } finally {
          // 🔒 ALWAYS-ON-TOP: Restaurer l'état précédent
          if (previousAlwaysOnTop) {
            try {
              await invoke('set_always_on_top', { alwaysOnTop: true });
              logger.debug('[Injection] Always-on-top RESTAURÉ');
            } catch (error) {
              logger.warn('[Injection] Impossible de restaurer always-on-top:', error);
            }
          }
        }
      })();
      
      // Race entre injection et timeout
      const success = await Promise.race([injectionPromise, timeoutPromise]);
      return success;
      
    } catch (error) {
      logger.error('=== ERREUR INJECTION ===', error);
      if (failureReason === 'TIMEOUT') {
        logger.error('[Injection] L\'injection a expiré après 5 secondes');
      }
      return false;
      
    } finally {
      // 🔄 REDÉMARRAGE: Après 500ms, redémarrer monitoring et traiter queue
      setTimeout(() => {
        startMonitoring();
        setIsInjecting(false);
        
        // Traiter la queue si des injections sont en attente
        if (injectionQueue.length > 0) {
          const nextText = injectionQueue[0];
          setInjectionQueue(prev => prev.slice(1));
          logger.debug('[Injection] Traitement queue, reste:', injectionQueue.length - 1);
          performInjection(nextText);
        }
      }, 500);
    }
  }, [isInjecting, injectionQueue, externalPositions, isLocked, lockedPosition, stopMonitoring, startMonitoring]);
  
  // Fonction pour tester la disponibilité de l'injection
  const testInjectionAvailability = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('get_cursor_position');
      return true;
    } catch (error) {
      logger.warn('[Injection] Fonctionnalité d\'injection non disponible:', error);
      return false;
    }
  }, []);
  
  // Fonctions de gestion de l'ancrage verrouillé
  const lockCurrentPosition = useCallback(async (): Promise<boolean> => {
    try {
      // CORRECTION: Permettre le verrouillage même quand l'app a le focus
      const position = await getCursorPosition();
      if (!position) {
        logger.error('[Lock] Impossible d\'obtenir la position actuelle');
        return false;
      }
      
      // Obtenir le nom de l'application active (simulation pour l'instant)
      const application = 'Application'; // TODO: Récupérer le nom réel de l'app
      
      const newLockedPosition: LockedPosition = {
        ...position,
        timestamp: Date.now(),
        application
      };
      
      setLockedPosition(newLockedPosition);
      setIsLocked(true);
      
      logger.debug(`[Lock] Position verrouillée: (${position.x}, ${position.y}) - App: ${application}`);
      return true;
    } catch (error) {
      logger.error('[Lock] Erreur verrouillage position:', error);
      return false;
    }
  }, [getCursorPosition]);
  
  const unlockPosition = useCallback(() => {
    setIsLocked(false);
    setLockedPosition(null);
    logger.debug('[Lock] Position déverrouillée');
  }, []);
  
  const updateLockedPosition = useCallback(async (): Promise<boolean> => {
    if (!isLocked) {
      logger.warn('[Lock] Aucune position à mettre à jour');
      return false;
    }
    
    const position = await getCursorPosition();
    if (!position || !lockedPosition) {
      return false;
    }
    
    const updatedPosition: LockedPosition = {
      ...position,
      timestamp: Date.now(),
      application: lockedPosition.application
    };
    
    setLockedPosition(updatedPosition);
    logger.debug(`[Lock] Position verrouillée mise à jour: (${position.x}, ${position.y})`);
    return true;
  }, [isLocked, lockedPosition, getCursorPosition]);

  return {
    getCursorPosition,
    performInjection,
    testInjectionAvailability,
    externalPositions,
    isMonitoring,
    startMonitoring,
    stopMonitoring,
    // Nouvelles fonctions d'ancrage
    lockCurrentPosition,
    unlockPosition,
    updateLockedPosition,
    isLocked,
    lockedPosition,
  };
};