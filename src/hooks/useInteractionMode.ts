import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { logger } from '@/utils/logger';
import { toast } from 'sonner';

interface CornerDetection {
  inCorner: boolean;
  enteredAt: number | null;
}

// Hook pour détecter l'intention d'interaction utilisateur via coin d'écran
export const useInteractionMode = (isInjecting: boolean) => {
  const [isInteractionMode, setIsInteractionMode] = useState(false);
  const [isTauriApp, setIsTauriApp] = useState(false);
  const cornerDetectionRef = useRef<CornerDetection>({ inCorner: false, enteredAt: null });
  const interactionTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Vérifier si on est dans Tauri au montage
  useEffect(() => {
    const checkTauri = () => {
      const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__;
      setIsTauriApp(isTauri);
    };
    checkTauri();
  }, []);

  // Détection si le curseur est dans le coin top-right
  const checkCornerPosition = useCallback(async () => {
    try {
      const position = await invoke<{ x: number; y: number }>('get_cursor_position');
      
      // Obtenir les dimensions de l'écran (estimation, ajuster si nécessaire)
      const screenWidth = window.screen.width;
      const screenHeight = window.screen.height;
      
      // Zone de détection: coin top-right (20px de marge)
      const isInCorner = position.x > screenWidth - 20 && position.y < 20;
      
      const now = Date.now();
      
      if (isInCorner) {
        if (!cornerDetectionRef.current.inCorner) {
          // Entrée dans le coin
          cornerDetectionRef.current = { inCorner: true, enteredAt: now };
          logger.debug('[InteractionMode] Curseur entré dans le coin');
        } else {
          // Vérifier si le curseur est resté 600ms dans le coin
          const timeInCorner = now - (cornerDetectionRef.current.enteredAt || 0);
          
          if (timeInCorner >= 600 && !isInteractionMode && !isInjecting) {
            // Activer le mode interaction
            activateInteractionMode();
          }
        }
      } else {
        // Sortie du coin
        if (cornerDetectionRef.current.inCorner) {
          cornerDetectionRef.current = { inCorner: false, enteredAt: null };
          logger.debug('[InteractionMode] Curseur sorti du coin');
        }
      }
    } catch (error) {
      logger.warn('[InteractionMode] Erreur détection position:', error);
    }
  }, [isInteractionMode, isInjecting]);

  // Activer le mode interaction pour 5 secondes
  const activateInteractionMode = useCallback(async () => {
    if (isInjecting) {
      logger.debug('[InteractionMode] Activation bloquée: injection en cours');
      return;
    }

    try {
      // Désactiver click-through
      await invoke('set_ignore_cursor_events', { ignore: false });
      setIsInteractionMode(true);
      
      logger.debug('[InteractionMode] Mode interaction ACTIVÉ (5s)');
      toast.info('Mode interaction activé (5s)', {
        duration: 5000,
        position: 'top-center',
      });

      // Réinitialiser le coin
      cornerDetectionRef.current = { inCorner: false, enteredAt: null };

      // Clear timeout existant si présent
      if (interactionTimeoutRef.current) {
        clearTimeout(interactionTimeoutRef.current);
      }

      // Désactiver après 5 secondes
      interactionTimeoutRef.current = setTimeout(() => {
        deactivateInteractionMode();
      }, 5000);
    } catch (error) {
      logger.error('[InteractionMode] Erreur activation:', error);
    }
  }, [isInjecting]);

  // Désactiver le mode interaction
  const deactivateInteractionMode = useCallback(async () => {
    try {
      // Réactiver click-through
      await invoke('set_ignore_cursor_events', { ignore: true });
      setIsInteractionMode(false);
      
      logger.debug('[InteractionMode] Mode interaction DÉSACTIVÉ');
      
      if (interactionTimeoutRef.current) {
        clearTimeout(interactionTimeoutRef.current);
        interactionTimeoutRef.current = null;
      }
    } catch (error) {
      logger.error('[InteractionMode] Erreur désactivation:', error);
    }
  }, []);

  // Polling de la position du curseur toutes les 150ms (seulement dans Tauri)
  useEffect(() => {
    if (!isTauriApp) {
      logger.debug('[InteractionMode] Mode non disponible hors Tauri');
      return;
    }

    pollingIntervalRef.current = setInterval(checkCornerPosition, 150);

    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
      if (interactionTimeoutRef.current) {
        clearTimeout(interactionTimeoutRef.current);
      }
    };
  }, [checkCornerPosition, isTauriApp]);

  // Désactiver automatiquement si une injection démarre
  useEffect(() => {
    if (isInjecting && isInteractionMode) {
      logger.debug('[InteractionMode] Injection détectée, désactivation auto');
      deactivateInteractionMode();
    }
  }, [isInjecting, isInteractionMode, deactivateInteractionMode]);

  // Listener pour blur window (perte de focus)
  useEffect(() => {
    const handleBlur = () => {
      if (isInteractionMode) {
        logger.debug('[InteractionMode] Blur détecté, désactivation');
        deactivateInteractionMode();
      }
    };

    window.addEventListener('blur', handleBlur);
    return () => window.removeEventListener('blur', handleBlur);
  }, [isInteractionMode, deactivateInteractionMode]);

  return {
    isInteractionMode,
    activateInteractionMode,
    deactivateInteractionMode,
  };
};
