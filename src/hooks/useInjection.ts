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
  hwnd?: number; // 🆕 Handle Windows pour identification exacte de la fenêtre
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
      const position = await getCursorPosition();
      if (!position) return;
      
      // 🆕 CAPTURE ROBUSTE: Utiliser get_window_at_point pour récupérer la fenêtre sous le curseur
      // (même si AirADCR a le focus)
      let windowInfo: WindowInfo | null = null;
      
      try {
        windowInfo = await invoke<WindowInfo>('get_window_at_point', { 
          x: position.x, 
          y: position.y 
        });
      } catch (error) {
        // Fallback: utiliser checkAppFocus classique
        const hasFocus = await checkAppFocus();
        if (!hasFocus) {
          windowInfo = await getActiveWindowInfo();
        }
      }
      
      // 🔒 FILTRAGE STRICT: Ignorer AirADCR pour capturer uniquement les fenêtres externes
      if (windowInfo && windowInfo.app_name !== 'AIRADCR') {
        const newPosition: CursorPosition = {
          ...position,
          timestamp: Date.now()
        };
        
        // 🆕 Stocker la dernière fenêtre externe capturée
        setLastExternalWindow(windowInfo);
        
        setExternalPositions(prev => {
          const updated = [newPosition, ...prev.slice(0, 2)]; // Garder les 3 dernières
          
          // Log TOUTES les captures pour diagnostic complet
          logger.debug(`📍 Position capturée: (${position.x}, ${position.y}) - ${windowInfo.app_name}`);
          
          return updated;
        });
      } else if (windowInfo && windowInfo.app_name === 'AIRADCR') {
        logger.debug('[Monitoring] ⏭️ Position ignorée (AirADCR détecté)');
      }
    } catch (error) {
      logger.warn('[Monitoring] Erreur capture position:', error);
    }
  }, [getCursorPosition, checkAppFocus, getActiveWindowInfo]);
  
  // Démarrer/arrêter la surveillance
  const startMonitoring = useCallback(() => {
    if (intervalRef.current) return;
    
    logger.debug('[Monitoring] 🚀 Démarrage surveillance externe (300ms) - Filtre AirADCR actif');
    setIsMonitoring(true);
    
    // Intervalle réduit pour capture plus réactive
    intervalRef.current = setInterval(captureExternalPosition, 300);
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
  const performInjection = useCallback(async (text: string, injectionType?: string, html?: string): Promise<boolean> => {
    // 🔒 PROTECTION: Bloquer si injection en cours (la queue est gérée par useSecureMessaging)
    if (isInjecting) {
      logger.warn('[Injection] Injection déjà en cours, retour immédiat');
      return false; // La sérialisation est gérée dans useSecureMessaging
    }
    
    if (!text || text.trim().length === 0) {
      logger.warn('[Injection] Texte vide, injection annulée');
      return false;
    }
    
    setIsInjecting(true);
    
    // 📡 Notifier les autres hooks que l'injection commence
    window.dispatchEvent(new CustomEvent('airadcr-injection-start'));
    
    stopMonitoring(); // ✅ Arrêter la capture pendant l'injection
    
    const startTime = Date.now();
    logger.info(`🎯 INJECTION ${injectionType || 'default'} - ${text.length} caractères - Verrouillée: ${!!lockedPosition}`);
    logger.debug('[Injection] Vérification priorités - Verrouillée:', !!lockedPosition, '| Externe disponible:', externalPositions.length > 0);
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
        
        // 🆕 Récupérer les infos du bureau virtuel pour clamper les coordonnées (pour tous les cas)
        let virtualDesktop: { x: number; y: number; width: number; height: number } | null = null;
        try {
          virtualDesktop = await invoke('get_virtual_desktop_info');
          logger.debug('[Injection] Bureau virtuel:', virtualDesktop);
        } catch (error) {
          logger.warn('[Injection] get_virtual_desktop_info non supporté (non-Windows):', error);
        }
        
        // 💡 Afficher l'info au premier usage
        const firstUsageKey = 'injection_tip_shown';
        if (!localStorage.getItem(firstUsageKey)) {
          setTimeout(() => {
            logger.info('💡 Pour remplacer du texte : sélectionnez-le manuellement dans votre logiciel avant l\'injection');
          }, 500);
          localStorage.setItem(firstUsageKey, 'true');
        }
        
        // PRIORITÉ 1: Position verrouillée avec conversion relative → absolue
        if (isLocked && lockedPosition) {
          const age = Date.now() - lockedPosition.timestamp;
          
          // Si on a une position relative, convertir en absolue en fonction de la fenêtre actuelle
          let targetX = lockedPosition.x;
          let targetY = lockedPosition.y;
          
          if (lockedPosition.relativePosition && lockedPosition.windowInfo) {
            // 🆕 AMÉLIORATION 1: Synchronisation active de lastExternalWindow
            logger.debug('[Injection] 🔄 Capture fraîche de la fenêtre cible...');
            
            const previousExternalWindow = lastExternalWindow;
            await captureExternalPosition();
            
            // 🆕 Polling actif: Attendre que lastExternalWindow soit mis à jour (max 1 seconde)
            const maxWaitMs = 1000;
            const startWait = Date.now();
            while (
              lastExternalWindow === previousExternalWindow && 
              Date.now() - startWait < maxWaitMs
            ) {
              await new Promise(resolve => setTimeout(resolve, 50));
            }
            
            if (lastExternalWindow === previousExternalWindow) {
              logger.warn('[Injection] ⚠️ lastExternalWindow non mis à jour après 1s, utilisation ancienne valeur');
            } else {
              logger.debug(`[Injection] ✅ lastExternalWindow synchronisé après ${Date.now() - startWait}ms`);
            }
            
            let windowToUse: WindowInfo | null = null;
            
            // PRIORITÉ: Utiliser lastExternalWindow si correspond
            if (lastExternalWindow?.app_name === lockedPosition.windowInfo.app_name) {
              windowToUse = lastExternalWindow;
              logger.debug(`[Injection] 🎯 Utilisation lastExternalWindow: ${windowToUse.app_name}`);
            } else {
              windowToUse = lockedPosition.windowInfo;
              logger.debug(`[Injection] ⚠️ Fallback windowInfo verrouillé: ${windowToUse.app_name}`);
            }
            
            // 🆕 MODE CLIENT RECT avec MULTI-POINT SCAN + HWND validation
            let injectionSuccess = false;
            const lockedAny = lockedPosition as any;
            
            if (lockedAny.relativeTo === 'client' && lockedAny.clientRectInfo) {
              try {
                // 🆕 AMÉLIORATION 3: MULTI-POINT SCAN étendu (centre + 4 coins + 2 mi-centres)
                const probePoints = [
                  { x: windowToUse.x + Math.floor(windowToUse.width / 2), y: windowToUse.y + Math.floor(windowToUse.height / 2), name: 'centre' },
                  { x: windowToUse.x + 50, y: windowToUse.y + 50, name: 'coin haut-gauche' },
                  { x: windowToUse.x + windowToUse.width - 50, y: windowToUse.y + 50, name: 'coin haut-droit' },
                  { x: windowToUse.x + 50, y: windowToUse.y + windowToUse.height - 50, name: 'coin bas-gauche' },
                  { x: windowToUse.x + windowToUse.width - 50, y: windowToUse.y + windowToUse.height - 50, name: 'coin bas-droit' },
                  { x: windowToUse.x + Math.floor(windowToUse.width / 4), y: windowToUse.y + Math.floor(windowToUse.height / 2), name: 'mi-centre gauche' },
                  { x: windowToUse.x + Math.floor(3 * windowToUse.width / 4), y: windowToUse.y + Math.floor(windowToUse.height / 2), name: 'mi-centre droit' },
                ];
                
                let currentClientRect: any = null;
                
                for (const probe of probePoints) {
                  try {
                    logger.debug(`[Injection] 📍 Test probe ${probe.name} à (${probe.x}, ${probe.y})`);
                    const rect = await invoke('get_window_client_rect_at_point', { x: probe.x, y: probe.y });
                    
                    // 🆕 AMÉLIORATION 3: Vérifier le HWND si disponible
                    const rectAny = rect as any;
                    const hwndMatch = !lockedAny.hwnd || !rectAny.hwnd || rectAny.hwnd === lockedAny.hwnd;
                    const appMatch = rectAny.app_name === lockedPosition.windowInfo.app_name;
                    
                    if (hwndMatch && appMatch) {
                      currentClientRect = rect;
                      logger.debug(`[Injection] ✅ Probe ${probe.name} réussie: app="${rectAny.app_name}" hwnd=${rectAny.hwnd} (attendu: ${lockedAny.hwnd})`);
                      break;
                    } else {
                      logger.debug(`[Injection] ⚠️ Probe ${probe.name} mismatch: app="${rectAny.app_name}" (attendu: "${lockedPosition.windowInfo.app_name}") hwnd=${rectAny.hwnd} (attendu: ${lockedAny.hwnd})`);
                    }
                  } catch (error) {
                    logger.debug(`[Injection] ⚠️ Probe ${probe.name} échouée:`, error);
                  }
                }
                
                if (!currentClientRect) {
                  throw new Error('Aucun probe multi-point valide trouvé');
                }
                
                // 2️⃣ Recalculer la position absolue depuis les ratios CLIENT
                const currentAny = currentClientRect as any;
                targetX = Math.round(
                  currentAny.client_left + 
                  (lockedPosition.relativePosition.ratio_x * currentAny.client_width)
                );
                targetY = Math.round(
                  currentAny.client_top + 
                  (lockedPosition.relativePosition.ratio_y * currentAny.client_height)
                );
                
                logger.debug(`[Injection] 🎯 CLIENT RECT MODE (multi-point + HWND):`);
                logger.debug(`[Injection]    Client zone: (${currentAny.client_left}, ${currentAny.client_top}) ${currentAny.client_width}x${currentAny.client_height}`);
                logger.debug(`[Injection]    Ratios: (${lockedPosition.relativePosition.ratio_x.toFixed(3)}, ${lockedPosition.relativePosition.ratio_y.toFixed(3)})`);
                logger.debug(`[Injection]    HWND: ${currentAny.hwnd}`);
                logger.debug(`[Injection]    → Position calculée: (${targetX}, ${targetY})`);
                
                injectionSuccess = true;
              } catch (error) {
                logger.warn('[Injection] ⚠️ Échec multi-point scan, fallback window rect:', error);
              }
            }
            
            // FALLBACK: Mode WINDOW RECT (ancien comportement)
            if (!injectionSuccess) {
              const usedWidth = windowToUse.width > 0 ? windowToUse.width : lockedPosition.windowInfo.width;
              const usedHeight = windowToUse.height > 0 ? windowToUse.height : lockedPosition.windowInfo.height;
              
              targetX = windowToUse.x + Math.round(lockedPosition.relativePosition.ratio_x * usedWidth);
              targetY = windowToUse.y + Math.round(lockedPosition.relativePosition.ratio_y * usedHeight);
              
              logger.debug(`[Injection] 📍 WINDOW RECT MODE (fallback):`);
              logger.debug(`[Injection]    Fenêtre: (${windowToUse.x}, ${windowToUse.y}) ${usedWidth}x${usedHeight}`);
              logger.debug(`[Injection]    → Position calculée: (${targetX}, ${targetY})`);
            }
          } else {
            // Fallback position absolue
            logger.warn(`[Injection] Position verrouillée incomplète, utilisation position actuelle`);
            const pos = await getCursorPosition();
            if (!pos) {
              throw new Error('Impossible d\'obtenir la position du curseur');
            }
            targetX = pos.x;
            targetY = pos.y;
          }
          
          // 🆕 Clamper les coordonnées dans les bornes du bureau virtuel
          if (virtualDesktop) {
            const vdMaxX = virtualDesktop.x + virtualDesktop.width - 1;
            const vdMaxY = virtualDesktop.y + virtualDesktop.height - 1;
            const originalX = targetX;
            const originalY = targetY;
            
            targetX = Math.max(virtualDesktop.x, Math.min(targetX, vdMaxX));
            targetY = Math.max(virtualDesktop.y, Math.min(targetY, vdMaxY));
            
            if (targetX !== originalX || targetY !== originalY) {
              logger.warn(`[Injection] Coordonnées clampées: (${originalX}, ${originalY}) → (${targetX}, ${targetY})`);
            }
            
            logger.debug(`[Injection] Coordonnées finales (verrouillée): (${targetX}, ${targetY}) dans bureau [${virtualDesktop.x}, ${virtualDesktop.y}, ${vdMaxX}, ${vdMaxY}]`);
          } else {
            logger.debug(`[Injection] Position verrouillée: (${targetX}, ${targetY}) - Âge: ${age}ms`);
          }
          
          await invoke('perform_injection_at_position_direct', {
            text,
            html: html || null,
            x: targetX,
            y: targetY
          });
          
          const duration = Date.now() - startTime;
          logger.info(`✅ INJECTION RÉUSSIE (verrouillée) à (${targetX}, ${targetY}) - ${lockedPosition.application} - Durée: ${duration}ms`);
          return true;
        }
        
        // PRIORITÉ 2: Dernière position externe si récente
        const lastExternalPosition = externalPositions[0];
        
        if (lastExternalPosition) {
          const age = Date.now() - lastExternalPosition.timestamp;
          const isPositionRecent = age < 60000; // Max 60 secondes (étendu)
          
          logger.debug(`[Injection] Position externe: (${lastExternalPosition.x}, ${lastExternalPosition.y}) - Âge: ${age}ms`);
          
          if (isPositionRecent) {
            const appName = lastExternalWindow?.app_name || 'Application inconnue';
            
            // 🔒 BLOCAGE CRITIQUE: Si la position externe pointe vers AirADCR, annuler
            if (appName === 'AIRADCR') {
              failureReason = 'TARGET_IS_AIRADCR';
              logger.error('[Injection] ❌ INJECTION ANNULÉE: Position externe pointe vers AirADCR (bug filtrage)');
              logger.error('[Injection] Cause probable: captureExternalPosition() a capturé AirADCR par erreur');
              return false;
            }
            
            // 🆕 Clamper les coordonnées pour position externe aussi
            let extX = lastExternalPosition.x;
            let extY = lastExternalPosition.y;
            
            if (virtualDesktop) {
              const vdMaxX = virtualDesktop.x + virtualDesktop.width - 1;
              const vdMaxY = virtualDesktop.y + virtualDesktop.height - 1;
              const originalExtX = extX;
              const originalExtY = extY;
              
              extX = Math.max(virtualDesktop.x, Math.min(extX, vdMaxX));
              extY = Math.max(virtualDesktop.y, Math.min(extY, vdMaxY));
              
              if (extX !== originalExtX || extY !== originalExtY) {
                logger.warn(`[Injection] Coordonnées externes clampées: (${originalExtX}, ${originalExtY}) → (${extX}, ${extY})`);
              }
              
              logger.debug(`[Injection] Coordonnées finales (externe): (${extX}, ${extY}) dans bureau [${virtualDesktop.x}, ${virtualDesktop.y}, ${vdMaxX}, ${vdMaxY}]`);
            }
            
            await invoke('perform_injection_at_position_direct', {
              text,
              html: html || null,
              x: extX,
              y: extY
            });
            
            const duration = Date.now() - startTime;
            
            // 🆕 Avertir si fenêtre non éditable (Explorer, shell, etc.)
            const nonEditableApps = ['explorer.exe', 'Explorer', 'Explorateur Windows', 'Shell', 'Taskbar'];
            if (nonEditableApps.some(app => appName.toLowerCase().includes(app.toLowerCase()))) {
              logger.warn(`⚠️ INJECTION dans fenêtre NON ÉDITABLE: ${appName} - Le texte peut ne pas apparaître. Ouvrez Word/RIS ou verrouillez une zone texte.`);
            }
            
            logger.info(`✅ INJECTION RÉUSSIE (externe) à (${extX}, ${extY}) - ${appName} - Durée: ${duration}ms`);
            return true;
          }
        }
        
        // PRIORITÉ 3: Capture automatique IMMÉDIATE (nouvelle méthode robuste)
        logger.info('⏱️ Aucune position récente, capture immédiate du curseur...');
        
        // 🆕 Capture directe avec get_window_at_point (sans attendre focus externe)
        const currentPos = await getCursorPosition();
        if (!currentPos) {
          failureReason = 'NO_CURSOR_POSITION';
          logger.error('❌ Impossible de récupérer la position du curseur');
          return false;
        }
        
        // Récupérer la fenêtre sous le curseur actuel
        let targetWindow: WindowInfo | null = null;
        try {
          targetWindow = await invoke<WindowInfo>('get_window_at_point', {
            x: currentPos.x,
            y: currentPos.y
          });
        } catch (error) {
          logger.warn('[Injection] get_window_at_point échoué, utilisation position brute:', error);
        }
        
        const appName = targetWindow?.app_name || 'Application inconnue';
        
        // 🔒 BLOCAGE CRITIQUE: Si le curseur est dans AirADCR, annuler l'injection
        if (appName === 'AIRADCR') {
          failureReason = 'CURSOR_IN_AIRADCR';
          logger.error('[Injection] ❌ INJECTION ANNULÉE: Curseur dans AirADCR. Placez le curseur dans Word/RIS avant d\'injecter.');
          return false;
        }
        
        let finalX = currentPos.x;
        let finalY = currentPos.y;
        
        if (virtualDesktop) {
          const vdMaxX = virtualDesktop.x + virtualDesktop.width - 1;
          const vdMaxY = virtualDesktop.y + virtualDesktop.height - 1;
          
          finalX = Math.max(virtualDesktop.x, Math.min(finalX, vdMaxX));
          finalY = Math.max(virtualDesktop.y, Math.min(finalY, vdMaxY));
        }
        
        await invoke('perform_injection_at_position_direct', {
          text,
          html: html || null,
          x: finalX,
          y: finalY
        });
        
        const duration = Date.now() - startTime;
        
        // 🆕 Avertir si fenêtre non éditable
        const nonEditableApps = ['explorer.exe', 'Explorer', 'Explorateur Windows', 'Shell', 'Taskbar'];
        if (nonEditableApps.some(app => appName.toLowerCase().includes(app.toLowerCase()))) {
          logger.warn(`⚠️ INJECTION dans fenêtre NON ÉDITABLE: ${appName} - Le texte peut ne pas apparaître. Ouvrez Word/RIS ou verrouillez une zone texte.`);
        }
        
        logger.info(`✅ INJECTION RÉUSSIE (auto-capture immédiate) à (${finalX}, ${finalY}) - ${appName} - Durée: ${duration}ms`);
        return true;
      })();
      
      // Race entre injection et timeout
      const success = await Promise.race([injectionPromise, timeoutPromise]);
      return success;
      
    } catch (error) {
      const duration = Date.now() - startTime;
      logger.error('=== ERREUR INJECTION ===', error);
      logger.error(`❌ INJECTION ÉCHOUÉE - Type: ${injectionType} - Verrouillée: ${!!lockedPosition} - Durée: ${duration}ms`);
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
      
      // 🆕 OBTENIR LE CLIENT RECT (zone cliquable sans bordures/titre) + HWND
      let clientRectInfo: any = null;
      try {
        clientRectInfo = await invoke('get_window_client_rect_at_point', { 
          x: position.x, 
          y: position.y 
        });
        logger.debug(`[Lock] ClientRect obtenu: client (${clientRectInfo.client_left}, ${clientRectInfo.client_top}) ${clientRectInfo.client_width}x${clientRectInfo.client_height} HWND=${clientRectInfo.hwnd}`);
      } catch (error) {
        logger.warn('[Lock] ⚠️ Impossible d\'obtenir ClientRect, fallback sur WindowRect:', error);
      }
      
      // 🆕 Calculer les RATIOS par rapport au CLIENT RECT (zone cliquable)
      let ratioX: number;
      let ratioY: number;
      let relativeTo = 'window'; // 'client' ou 'window'
      
      if (clientRectInfo) {
        // PRIORITÉ 1: Ratios par rapport au CLIENT RECT (précis multi-écrans + DPI)
        const relativeX = position.x - clientRectInfo.client_left;
        const relativeY = position.y - clientRectInfo.client_top;
        ratioX = clientRectInfo.client_width > 0 ? relativeX / clientRectInfo.client_width : 0;
        ratioY = clientRectInfo.client_height > 0 ? relativeY / clientRectInfo.client_height : 0;
        relativeTo = 'client';
        
        logger.debug(`[Lock] 🎯 Ratios CLIENT: (${ratioX.toFixed(3)}, ${ratioY.toFixed(3)}) depuis position (${relativeX}, ${relativeY})`);
      } else {
        // FALLBACK: Ratios par rapport au WINDOW RECT (moins précis mais fonctionne)
        const relativeX = position.x - windowInfo.x;
        const relativeY = position.y - windowInfo.y;
        ratioX = windowInfo.width > 0 ? relativeX / windowInfo.width : 0;
        ratioY = windowInfo.height > 0 ? relativeY / windowInfo.height : 0;
        
        logger.debug(`[Lock] ⚠️ Ratios WINDOW (fallback): (${ratioX.toFixed(3)}, ${ratioY.toFixed(3)})`);
      }
      
      const relativePosition: RelativePosition = {
        relative_x: position.x - windowInfo.x, // Garder pour rétrocompatibilité
        relative_y: position.y - windowInfo.y,
        ratio_x: ratioX,
        ratio_y: ratioY,
        timestamp: Date.now()
      };
      
      const newLockedPosition: LockedPosition = {
        ...position,
        timestamp: Date.now(),
        application: windowInfo.app_name,
        windowInfo,
        relativePosition,
        clientRectInfo, // 🆕 Stocker les infos client rect
        relativeTo, // 🆕 'client' ou 'window'
        hwnd: clientRectInfo?.hwnd // 🆕 Stocker le HWND pour identification exacte
      } as any;
      
      setLockedPosition(newLockedPosition);
      setIsLocked(true);
      
      // Sauvegarder dans localStorage
      saveLockedPositions(windowInfo.app_name, newLockedPosition);
      
      logger.info(`[Lock] ✅ Verrouillage accepté: ${windowInfo.app_name} à (${position.x}, ${position.y})`);
      logger.debug(`[Lock] Position verrouillée (relativeTo: ${relativeTo})`);
      logger.debug(`[Lock] Ratios: (${ratioX.toFixed(3)}, ${ratioY.toFixed(3)}) dans ${windowInfo.app_name}`);
      logger.debug(`[Lock] Position absolue: (${position.x}, ${position.y})`);
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