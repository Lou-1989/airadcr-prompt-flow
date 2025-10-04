import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { logger } from '@/utils/logger';

interface CursorPosition {
  x: number;
  y: number;
  timestamp: number;
}

interface WindowInfo {
  title: string;
  app_name: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface RelativePosition {
  relative_x: number;
  relative_y: number;
  ratio_x: number; // 🆕 Position en % de largeur fenêtre (0-1)
  ratio_y: number; // 🆕 Position en % de hauteur fenêtre (0-1)
  timestamp: number;
}

interface LockedPosition extends CursorPosition {
  application: string;
  windowInfo?: WindowInfo;
  relativePosition?: RelativePosition;
}

// Hook spécialisé pour la gestion de l'injection sécurisée avec capture préventive
export const useInjection = () => {
  const [externalPositions, setExternalPositions] = useState<CursorPosition[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [lockedPosition, setLockedPosition] = useState<LockedPosition | null>(null);
  const [isLocked, setIsLocked] = useState(false);
  const [isInjecting, setIsInjecting] = useState(false);
  const [injectionQueue, setInjectionQueue] = useState<string[]>([]);
  const [activeWindow, setActiveWindow] = useState<WindowInfo | null>(null);
  const [lastExternalWindow, setLastExternalWindow] = useState<WindowInfo | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  
  // Charger les positions sauvegardées depuis localStorage
  useEffect(() => {
    const saved = localStorage.getItem('airadcr_locked_positions');
    if (saved) {
      try {
        const positions = JSON.parse(saved);
        logger.debug('[Storage] Positions restaurées:', Object.keys(positions));
      } catch (error) {
        logger.warn('[Storage] Erreur chargement positions:', error);
      }
    }
  }, []);
  
  // Fonction pour obtenir les infos de la fenêtre active
  const getActiveWindowInfo = useCallback(async (): Promise<WindowInfo | null> => {
    try {
      const windowInfo = await invoke<WindowInfo>('get_active_window_info');
      setActiveWindow(windowInfo);
      return windowInfo;
    } catch (error) {
      logger.warn('[Window] Erreur récupération fenêtre active:', error);
      return null;
    }
  }, []);

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
  
  // Fonction pour capturer la position externe ET la fenêtre externe
  const captureExternalPosition = useCallback(async () => {
    try {
      // ✅ SOLUTION: Capturer uniquement quand l'app N'A PAS le focus
      const hasFocus = await checkAppFocus();
      
      if (hasFocus) {
        // Ne pas capturer si on est dans AirADCR
        return;
      }
      
      const position = await getCursorPosition();
      const windowInfo = await getActiveWindowInfo(); // 🆕 Capturer la fenêtre EXTERNE
      
      if (position && windowInfo) {
        const newPosition: CursorPosition = {
          ...position,
          timestamp: Date.now()
        };
        
        // 🆕 Stocker la dernière fenêtre externe capturée
        setLastExternalWindow(windowInfo);
        
        setExternalPositions(prev => {
          const updated = [newPosition, ...prev.slice(0, 2)]; // Garder les 3 dernières
          logger.debug('[Monitoring] Position + Fenêtre EXTERNE capturées:', {
            position: newPosition,
            window: windowInfo.app_name
          });
          return updated;
        });
      }
    } catch (error) {
      logger.warn('[Monitoring] Erreur capture position:', error);
    }
  }, [getCursorPosition, checkAppFocus, getActiveWindowInfo]);
  
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
  
  // 🔒 FONCTION PRINCIPALE: Injection sécurisée avec click-through professionnel
  const performInjection = useCallback(async (text: string, injectionType?: string): Promise<boolean> => {
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
    
    // 📡 Notifier les autres hooks que l'injection commence
    window.dispatchEvent(new CustomEvent('airadcr-injection-start'));
    
    stopMonitoring(); // ✅ Arrêter la capture pendant l'injection
    
    logger.debug('=== DÉBUT INJECTION PROFESSIONNELLE ===');
    logger.debug('[Injection] TYPE:', injectionType || 'default');
    logger.debug('[Injection] Texte à injecter:', text.substring(0, 50) + '...');
    logger.debug('[Injection] Longueur:', text.length, 'caractères');
    logger.debug('[Injection] Position verrouillée:', !!lockedPosition);
    logger.debug('[Injection] Positions externes:', externalPositions.length);
    
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
        // 🖱️ PHASE 1: MAINTENIR click-through activé pour injection externe
        await invoke('set_ignore_cursor_events', { ignore: true });
        logger.debug('[Injection] Click-through MAINTENU (injection externe)');
        
        // PRIORITÉ 1: Position verrouillée avec conversion relative → absolue
        if (isLocked && lockedPosition) {
          const age = Date.now() - lockedPosition.timestamp;
          
          // Si on a une position relative, convertir en absolue en fonction de la fenêtre actuelle
          let targetX = lockedPosition.x;
          let targetY = lockedPosition.y;
          
          if (lockedPosition.relativePosition && lockedPosition.windowInfo) {
            // Obtenir la fenêtre active actuelle
            const currentWindow = await getActiveWindowInfo();
            
            // 🆕 FALLBACK INTELLIGENT MULTI-ÉCRANS
            if (currentWindow && currentWindow.app_name === lockedPosition.windowInfo.app_name) {
              // ✅ CAS 1: Même application, utiliser ratio pour gérer redimensionnement
              const usedWidth = currentWindow.width > 0 ? currentWindow.width : lockedPosition.windowInfo.width;
              const usedHeight = currentWindow.height > 0 ? currentWindow.height : lockedPosition.windowInfo.height;
              
              targetX = currentWindow.x + Math.round(lockedPosition.relativePosition.ratio_x * usedWidth);
              targetY = currentWindow.y + Math.round(lockedPosition.relativePosition.ratio_y * usedHeight);
              
              logger.debug(`[Injection] ✅ Position ratio convertie: ratio(${lockedPosition.relativePosition.ratio_x.toFixed(2)}, ${lockedPosition.relativePosition.ratio_y.toFixed(2)}) → (${targetX}, ${targetY})`);
              logger.debug(`[Injection] Fenêtre: (${lockedPosition.windowInfo.x}, ${lockedPosition.windowInfo.y}) ${lockedPosition.windowInfo.width}x${lockedPosition.windowInfo.height} → (${currentWindow.x}, ${currentWindow.y}) ${currentWindow.width}x${currentWindow.height}`);
            } else if (lastExternalWindow && lastExternalWindow.app_name === lockedPosition.windowInfo.app_name) {
              // ⚠️ CAS 2: Application changée mais lastExternalWindow correspond → utiliser ratio
              const usedWidth = lastExternalWindow.width > 0 ? lastExternalWindow.width : lockedPosition.windowInfo.width;
              const usedHeight = lastExternalWindow.height > 0 ? lastExternalWindow.height : lockedPosition.windowInfo.height;
              
              targetX = lastExternalWindow.x + Math.round(lockedPosition.relativePosition.ratio_x * usedWidth);
              targetY = lastExternalWindow.y + Math.round(lockedPosition.relativePosition.ratio_y * usedHeight);
              
              logger.warn(`[Injection] ⚠️ Application changée (${currentWindow?.app_name} vs ${lockedPosition.windowInfo.app_name})`);
              logger.debug(`[Injection] 🔄 Utilisation lastExternalWindow: ratio(${lockedPosition.relativePosition.ratio_x.toFixed(2)}, ${lockedPosition.relativePosition.ratio_y.toFixed(2)}) → (${targetX}, ${targetY})`);
              logger.debug(`[Injection] Fenêtre externe: "${lastExternalWindow.title}" à (${lastExternalWindow.x}, ${lastExternalWindow.y}) ${lastExternalWindow.width}x${lastExternalWindow.height}`);
            } else {
              // ❌ CAS 3: Fallback position absolue
              logger.warn(`[Injection] ❌ Aucune fenêtre correspondante, utilisation position absolue: (${targetX}, ${targetY})`);
              logger.debug(`[Injection] locked.app=${lockedPosition.windowInfo.app_name}, current.app=${currentWindow?.app_name}, lastExt.app=${lastExternalWindow?.app_name}`);
            }
            
            // 📊 LOGS MULTI-ÉCRANS DÉTAILLÉS
            logger.debug('[Injection] État multi-écrans:', {
              isLocked: true,
              lockedApp: lockedPosition.windowInfo.app_name,
              currentApp: currentWindow?.app_name,
              lastExternalApp: lastExternalWindow?.app_name,
              targetCoords: `(${targetX}, ${targetY})`,
              hasNegativeCoords: targetX < 0 || targetY < 0,
              externalPositionAge: externalPositions[0] ? Date.now() - externalPositions[0].timestamp : 'N/A'
            });
          }
          
          logger.debug(`[Injection] Position verrouillée: (${targetX}, ${targetY}) - Âge: ${age}ms`);
          
          await invoke('perform_injection_at_position_direct', {
            text,
            x: targetX,
            y: targetY
          });
          
          logger.debug(`✅ INJECTION RÉUSSIE (${injectionType || 'default'}) verrouillée à (${targetX}, ${targetY})`);
          return true;
        }
        
        // PRIORITÉ 2: Dernière position externe si récente
        const lastExternalPosition = externalPositions[0];
        
        if (lastExternalPosition) {
          const age = Date.now() - lastExternalPosition.timestamp;
          const isPositionRecent = age < 30000; // Max 30 secondes
          
          logger.debug(`[Injection] Position externe: (${lastExternalPosition.x}, ${lastExternalPosition.y}) - Âge: ${age}ms`);
          
          if (isPositionRecent) {
            await invoke('perform_injection_at_position_direct', {
              text,
              x: lastExternalPosition.x,
              y: lastExternalPosition.y
            });
            
            logger.debug(`✅ INJECTION RÉUSSIE (${injectionType || 'default'}) externe à (${lastExternalPosition.x}, ${lastExternalPosition.y})`);
            return true;
          } else {
            failureReason = 'POSITION_TOO_OLD';
            logger.error(`❌ Position trop ancienne (${age}ms). Cliquez dans RIS/Word puis réessayez.`);
            return false;
          }
        } else {
          failureReason = 'NO_EXTERNAL_POSITION';
          logger.error('❌ Aucune position capturée. Cliquez dans RIS/Word puis réessayez.');
          return false;
        }
      })();
      
      // Race entre injection et timeout
      const success = await Promise.race([injectionPromise, timeoutPromise]);
      return success;
      
    } catch (error) {
      logger.error('=== ERREUR INJECTION ===', error);
      if (failureReason === 'TIMEOUT') {
        logger.error('[Injection] Timeout après 5 secondes');
      }
      return false;
      
    } finally {
      // 🔓 CRITIQUE: Désactiver click-through immédiatement pour rendre l'UI cliquable
      invoke('set_ignore_cursor_events', { ignore: false }).catch(() => {
        logger.warn('[Injection] Erreur désactivation click-through (ignorée)');
      });
      
      // 🔒 DOUBLE-SÉCURITÉ: Reforcer ignore:false après 300ms pour contrer les races conditions
      setTimeout(() => {
        invoke('set_ignore_cursor_events', { ignore: false }).catch(() => {
          logger.warn('[Injection] Erreur double-sécurité click-through (ignorée)');
        });
      }, 300);
      
      // 🔄 REDÉMARRAGE: Après 500ms, redémarrer monitoring et traiter queue
      setTimeout(() => {
        startMonitoring();
        setIsInjecting(false);
        
        // 📡 Notifier les autres hooks que l'injection est terminée
        window.dispatchEvent(new CustomEvent('airadcr-injection-end'));
        
        // ⚠️ NE PLUS RÉACTIVER LE CLICK-THROUGH - L'UI reste cliquable définitivement
        
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
  
  // Sauvegarder les positions dans localStorage
  const saveLockedPositions = useCallback((appName: string, position: LockedPosition) => {
    try {
      const saved = localStorage.getItem('airadcr_locked_positions');
      const positions = saved ? JSON.parse(saved) : {};
      positions[appName] = position;
      localStorage.setItem('airadcr_locked_positions', JSON.stringify(positions));
      logger.debug(`[Storage] Position sauvegardée pour ${appName}`);
    } catch (error) {
      logger.warn('[Storage] Erreur sauvegarde position:', error);
    }
  }, []);

  // Fonctions de gestion de l'ancrage verrouillé avec positionnement relatif
  const lockCurrentPosition = useCallback(async (): Promise<boolean> => {
    try {
      const position = await getCursorPosition();
      if (!position) {
        logger.error('[Lock] Impossible d\'obtenir la position actuelle');
        return false;
      }
      
      // 🔒 UTILISER LA DERNIÈRE FENÊTRE EXTERNE CAPTURÉE (pas getActiveWindowInfo!)
      const windowInfo = lastExternalWindow;
      
      if (!windowInfo) {
        logger.error('[Lock] ❌ Aucune fenêtre externe capturée. Cliquez dans RIS/Word d\'abord.');
        return false;
      }
      
      // ✅ VÉRIFIER que ce n'est PAS AirADCR
      const appNameLower = windowInfo.app_name.toLowerCase();
      if (appNameLower.includes('airadcr') || appNameLower.includes('tauri')) {
        logger.error('[Lock] ❌ Impossible de verrouiller AirADCR. Cliquez dans RIS/Word.');
        return false;
      }
      
      // Calculer la position RELATIVE par rapport à la fenêtre
      const relativeX = position.x - windowInfo.x;
      const relativeY = position.y - windowInfo.y;
      
      // 🆕 Calculer les RATIOS (robuste pour redimensionnement et multi-écrans)
      const ratioX = windowInfo.width > 0 ? relativeX / windowInfo.width : 0;
      const ratioY = windowInfo.height > 0 ? relativeY / windowInfo.height : 0;
      
      const relativePosition: RelativePosition = {
        relative_x: relativeX,
        relative_y: relativeY,
        ratio_x: ratioX,
        ratio_y: ratioY,
        timestamp: Date.now()
      };
      
      const newLockedPosition: LockedPosition = {
        ...position,
        timestamp: Date.now(),
        application: windowInfo.app_name,
        windowInfo,
        relativePosition
      };
      
      setLockedPosition(newLockedPosition);
      setIsLocked(true);
      
      // Sauvegarder dans localStorage
      saveLockedPositions(windowInfo.app_name, newLockedPosition);
      
      logger.debug(`[Lock] ✅ Position verrouillée RELATIVE: (${relativePosition.relative_x}, ${relativePosition.relative_y}) ratio(${relativePosition.ratio_x.toFixed(2)}, ${relativePosition.ratio_y.toFixed(2)}) dans ${windowInfo.app_name}`);
      logger.debug(`[Lock] Position absolue: (${position.x}, ${position.y})`);
      logger.debug(`[Lock] Fenêtre: "${windowInfo.title}" à (${windowInfo.x}, ${windowInfo.y}) ${windowInfo.width}x${windowInfo.height}`);
      return true;
    } catch (error) {
      logger.error('[Lock] Erreur verrouillage position:', error);
      return false;
    }
  }, [getCursorPosition, lastExternalWindow, saveLockedPositions]);
  
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
    lockCurrentPosition,
    unlockPosition,
    updateLockedPosition,
    isLocked,
    lockedPosition,
    isInjecting,
    activeWindow,
    lastExternalWindow,
    getActiveWindowInfo,
  };
};