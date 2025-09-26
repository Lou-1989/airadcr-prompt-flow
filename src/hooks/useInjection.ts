import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

// Hook spécialisé pour la gestion de l'injection sécurisée
export const useInjection = () => {
  
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
  
  // Fonction principale d'injection - WORKFLOW UTILISATEUR CORRIGÉ
  const performInjection = useCallback(async (text: string): Promise<boolean> => {
    if (!text || text.trim().length === 0) {
      console.warn('[Injection Frontend] Texte vide, injection annulée');
      return false;
    }
    
    try {
      console.log('[Injection Frontend] 🎯 DÉMARRAGE injection sécurisée...');
      console.log('[Injection Frontend] Texte:', text.substring(0, 50) + (text.length > 50 ? '...' : ''));
      
      // DÉLAI ÉTENDU pour permettre à l'utilisateur de REPOSITIONNER le curseur dans l'app cible
      console.log('[Injection Frontend] ⏳ REPOSITIONNEZ votre curseur dans l\'application cible (RIS/Word/etc.) - 3 secondes...');
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      // Effectuer l'injection via Tauri (position détectée côté Rust juste avant injection)
      const [x, y] = await invoke<[number, number]>('perform_injection', { text });
      
      console.log(`[Injection Frontend] ✅ Injection réussie à la position (${x}, ${y})`);
      console.log('[Injection Frontend] 🎉 INJECTION TERMINÉE');
      return true;
      
    } catch (error) {
      console.error('[Injection Frontend] ❌ Erreur lors de l\'injection:', error);
      return false;
    }
  }, []);
  
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
  };
};