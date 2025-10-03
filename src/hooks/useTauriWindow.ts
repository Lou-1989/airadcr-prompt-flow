import { useEffect, useState, useCallback } from 'react';
import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';
import { platform, arch, version } from '@tauri-apps/api/os';
import { logger } from '@/utils/logger';

interface WindowPosition {
  x: number;
  y: number;
}

interface SystemInfo {
  platform: string;
  arch: string;
  version: string;
}

export const useTauriWindow = () => {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(true);
  const [windowPosition, setWindowPosition] = useState<WindowPosition>({ x: 0, y: 0 });
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [isTauriApp, setIsTauriApp] = useState(false);
  const [isInjectionInProgress, setIsInjectionInProgress] = useState(false);

  // Détection Tauri et récupération info système
  useEffect(() => {
    const checkTauri = async () => {
      try {
        if (typeof window !== 'undefined' && (window as any).__TAURI__) {
          setIsTauriApp(true);
          
          try {
            const [platformName, archName, versionName] = await Promise.all([
              platform(),
              arch(),
              version()
            ]);
            
            setSystemInfo({
              platform: platformName,
              arch: archName,
              version: versionName
            });
          } catch (error) {
            logger.error('Erreur récupération info système:', error);
            setSystemInfo({
              platform: 'Windows',
              arch: 'x64',
              version: '1.0.0'
            });
          }
          
          logger.debug('Environnement Tauri détecté');
        }
      } catch (error) {
        logger.info('Environnement web standard détecté');
      }
    };

    checkTauri();
  }, []);

  // Initialisation always-on-top au démarrage UNIQUEMENT
  // Le click-through est désormais géré par useInteractionMode
  useEffect(() => {
    if (!isTauriApp) return;
    
    const initWindow = async () => {
      try {
        await invoke('set_always_on_top', { always_on_top: true });
        await new Promise(resolve => setTimeout(resolve, 200));
        const confirmedStatus = await invoke('get_always_on_top_status');
        setIsAlwaysOnTop(confirmedStatus as boolean);
        logger.debug(`✅ Always-on-top confirmé au démarrage: ${confirmedStatus}`);
      } catch (error) {
        logger.error('❌ Erreur initialisation always-on-top:', error);
      }
    };
    
    initWindow();
  }, [isTauriApp]);

  // 📡 Écouter les événements d'injection pour contrôle explicite always-on-top
  useEffect(() => {
    if (!isTauriApp) return;
    
    const handleInjectionStart = async () => {
      setIsInjectionInProgress(true);
      
      try {
        // 🎯 DÉSACTIVER always-on-top pour que l'application cible reste au premier plan
        await invoke('set_always_on_top', { always_on_top: false });
        logger.debug('⏸️ Always-on-top DÉSACTIVÉ (injection démarrée)');
      } catch (error) {
        logger.error('Erreur désactivation always-on-top:', error);
      }
    };
    
    const handleInjectionEnd = async () => {
      setIsInjectionInProgress(false);
      
      try {
        // ⏳ Attendre stabilisation de l'injection (300ms)
        await new Promise(resolve => setTimeout(resolve, 300));
        
        // 🎯 RÉACTIVER always-on-top après injection
        await invoke('set_always_on_top', { always_on_top: true });
        setIsAlwaysOnTop(true);
        logger.debug('▶️ Always-on-top RÉACTIVÉ (injection terminée)');
      } catch (error) {
        logger.error('Erreur réactivation always-on-top:', error);
      }
    };
    
    window.addEventListener('airadcr-injection-start', handleInjectionStart);
    window.addEventListener('airadcr-injection-end', handleInjectionEnd);
    
    return () => {
      window.removeEventListener('airadcr-injection-start', handleInjectionStart);
      window.removeEventListener('airadcr-injection-end', handleInjectionEnd);
    };
  }, [isTauriApp]);

  // Surveillance NON-INVASIVE du statut always-on-top
  // Vérifie toutes les 3 secondes et restaure UNIQUEMENT si perdu
  useEffect(() => {
    if (!isTauriApp) return;
    
    const intervalId = setInterval(async () => {
      // ⛔ SKIP si injection en cours pour éviter conflit Z-order
      if (isInjectionInProgress) {
        logger.debug('⏸️ Surveillance always-on-top suspendue (injection en cours)');
        return;
      }
      
      try {
        const currentState = await invoke('get_always_on_top_status');
        
        // Restaurer UNIQUEMENT si désactivé (manuel ou bug Windows)
        if (!currentState) {
          logger.warn('⚠️ Always-on-top perdu, restauration silencieuse...');
          await invoke('set_always_on_top', { always_on_top: true });
          
          // ❌ PAS de set_focus() pour éviter conflit injection
          setIsAlwaysOnTop(true);
        }
      } catch (error) {
        logger.warn('Erreur surveillance always-on-top:', error);
      }
    }, 3000); // 3 secondes - non-agressif
    
    return () => clearInterval(intervalId);
  }, [isTauriApp, isInjectionInProgress]);

  // Basculer always-on-top avec vraie API Tauri
  const toggleAlwaysOnTop = useCallback(async () => {
    if (!isTauriApp) {
      logger.warn('Always-on-top non disponible dans le navigateur');
      return false;
    }

    try {
      // Utiliser l'invoke command pour gérer always-on-top
      const newState = await invoke('toggle_always_on_top');
      setIsAlwaysOnTop(newState as boolean);
      logger.debug(`Always-on-top ${newState ? 'activé' : 'désactivé'}`);
      return newState as boolean;
    } catch (error) {
      logger.error('Erreur toggle always-on-top:', error);
      return isAlwaysOnTop;
    }
  }, [isTauriApp, isAlwaysOnTop]);

  // Définir la position de la fenêtre avec vraie API Tauri
  const setPosition = useCallback(async (x: number, y: number) => {
    if (!isTauriApp) return false;

    try {
      await invoke('set_window_position', { x, y });
      setWindowPosition({ x, y });
      logger.debug(`Position fenêtre: ${x}, ${y}`);
      return true;
    } catch (error) {
      logger.error('Erreur positionnement fenêtre:', error);
      return false;
    }
  }, [isTauriApp]);

  // Minimiser vers le system tray avec command Rust
  const minimizeToTray = useCallback(async () => {
    if (!isTauriApp) {
      logger.warn('System tray non disponible dans le navigateur');
      return false;
    }

    try {
      await invoke('minimize_to_tray');
      logger.debug('Minimisation vers system tray');
      return true;
    } catch (error) {
      logger.error('Erreur minimisation:', error);
      return false;
    }
  }, [isTauriApp]);

  // Restaurer depuis le system tray avec command Rust
  const restoreFromTray = useCallback(async () => {
    if (!isTauriApp) return false;

    try {
      await invoke('restore_from_tray');
      logger.debug('Restauration depuis system tray');
      return true;
    } catch (error) {
      logger.error('Erreur restauration:', error);
      return false;
    }
  }, [isTauriApp]);

  // Basculer la visibilité de la fenêtre
  const toggleVisibility = useCallback(async () => {
    if (!isTauriApp) return false;

    try {
      const isVisible = await appWindow.isVisible();
      if (isVisible) {
        await appWindow.hide();
      } else {
        await appWindow.show();
        await appWindow.setFocus();
      }
      logger.debug(`Fenêtre ${isVisible ? 'cachée' : 'affichée'}`);
      return !isVisible;
    } catch (error) {
      logger.error('Erreur toggle visibilité:', error);
      return false;
    }
  }, [isTauriApp]);

  // Fermer complètement l'application
  const quitApp = useCallback(async () => {
    if (!isTauriApp) {
      logger.warn('Fermeture app non disponible dans le navigateur');
      return;
    }

    try {
      await appWindow.close();
      logger.debug('Fermeture application Tauri');
    } catch (error) {
      logger.error('Erreur fermeture app:', error);
    }
  }, [isTauriApp]);

  return {
    isTauriApp,
    isAlwaysOnTop,
    windowPosition,
    systemInfo,
    toggleAlwaysOnTop,
    setPosition,
    minimizeToTray,
    restoreFromTray,
    toggleVisibility,
    quitApp,
  };
};