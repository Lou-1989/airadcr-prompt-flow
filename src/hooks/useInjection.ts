import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface CursorPosition {
  x: number;
  y: number;
  timestamp: number;
}

// Hook spécialisé pour la gestion de l'injection sécurisée avec capture préventive
export const useInjection = () => {
  const [externalPositions, setExternalPositions] = useState<CursorPosition[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  
  // Fonction pour vérifier si l'app a le focus
  const checkAppFocus = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>('check_app_focus');
    } catch (error) {
      console.warn('[Focus] Erreur vérification focus:', error);
      return true; // Par défaut, considérer qu'on a le focus en cas d'erreur
    }
  }, []);
  
  // Fonction pour capturer la position externe
  const captureExternalPosition = useCallback(async () => {
    try {
      const hasFocus = await checkAppFocus();
      
      // Ne capturer que si l'app n'a PAS le focus
      if (!hasFocus) {
        const position = await getCursorPosition();
        if (position) {
          const newPosition: CursorPosition = {
            ...position,
            timestamp: Date.now()
          };
          
          setExternalPositions(prev => {
            const updated = [newPosition, ...prev.slice(0, 2)]; // Garder les 3 dernières
            console.log('[Monitoring] Position externe capturée:', newPosition);
            return updated;
          });
        }
      }
    } catch (error) {
      console.warn('[Monitoring] Erreur capture position externe:', error);
    }
  }, []);
  
  // Démarrer/arrêter la surveillance
  const startMonitoring = useCallback(() => {
    if (intervalRef.current) return;
    
    console.log('[Monitoring] Démarrage surveillance positions externes...');
    setIsMonitoring(true);
    
    intervalRef.current = setInterval(captureExternalPosition, 500);
  }, [captureExternalPosition]);
  
  const stopMonitoring = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
      setIsMonitoring(false);
      console.log('[Monitoring] Arrêt surveillance positions externes');
    }
  }, []);
  
  // Démarrer automatiquement la surveillance
  useEffect(() => {
    startMonitoring();
    return () => stopMonitoring();
  }, [startMonitoring, stopMonitoring]);
  
  // Fonction pour obtenir la position actuelle du curseur
  const getCursorPosition = useCallback(async (): Promise<{ x: number; y: number } | null> => {
    try {
      const [x, y] = await invoke<[number, number]>('get_cursor_position');
      return { x, y };
    } catch (error) {
      console.error('[Injection] Erreur récupération position curseur:', error);
      return null;
    }
  }, []);
  
  // Fonction principale d'injection avec position préventive
  const performInjection = useCallback(async (text: string): Promise<boolean> => {
    if (!text || text.trim().length === 0) {
      console.warn('[Injection] Texte vide, injection annulée');
      return false;
    }
    
    try {
      console.log('[Injection] Démarrage injection sécurisée...');
      
      // Utiliser la dernière position externe si disponible
      const lastExternalPosition = externalPositions[0];
      
      if (lastExternalPosition) {
        // Vérifier que la position n'est pas trop ancienne (max 30 secondes)
        const isPositionRecent = (Date.now() - lastExternalPosition.timestamp) < 30000;
        
        if (isPositionRecent) {
          console.log(`[Injection] Utilisation position externe: (${lastExternalPosition.x}, ${lastExternalPosition.y})`);
          
          // Effectuer l'injection à la position sauvegardée
          const [x, y] = await invoke<[number, number]>('perform_injection_at_position', {
            text,
            x: lastExternalPosition.x,
            y: lastExternalPosition.y
          });
          
          console.log(`[Injection] Injection réussie à la position externe (${x}, ${y})`);
          return true;
        } else {
          console.warn('[Injection] Position externe trop ancienne, utilisation position actuelle');
        }
      } else {
        console.warn('[Injection] Aucune position externe disponible, utilisation position actuelle');
      }
      
      // Fallback: utiliser la méthode normale
      const [x, y] = await invoke<[number, number]>('perform_injection', { text });
      console.log(`[Injection] Injection réussie à la position actuelle (${x}, ${y})`);
      return true;
      
    } catch (error) {
      console.error('[Injection] Erreur lors de l\'injection:', error);
      return false;
    }
  }, [externalPositions]);
  
  // Fonction pour tester la disponibilité de l'injection
  const testInjectionAvailability = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('get_cursor_position');
      return true;
    } catch (error) {
      console.warn('[Injection] Fonctionnalité d\'injection non disponible:', error);
      return false;
    }
  }, []);
  
  return {
    getCursorPosition,
    performInjection,
    testInjectionAvailability,
    externalPositions,
    isMonitoring,
    startMonitoring,
    stopMonitoring,
  };
};