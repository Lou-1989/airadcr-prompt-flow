import { useEffect, useState, useCallback } from 'react';

interface WindowPosition {
  x: number;
  y: number;
}

interface SystemInfo {
  platform: string;
  arch: string;
  version: string;
}

declare global {
  interface Window {
    __TAURI__?: any;
  }
}

export const useTauriWindow = () => {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(false);
  const [windowPosition, setWindowPosition] = useState<WindowPosition>({ x: 0, y: 0 });
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [isTauriApp, setIsTauriApp] = useState(false);

  // Détection si on est dans l'app Tauri
  useEffect(() => {
    const checkTauri = () => {
      try {
        if (window.__TAURI__) {
          setIsTauriApp(true);
          setSystemInfo({
            platform: 'Windows',
            arch: 'x64',
            version: '1.0.0'
          });
          console.log('Environnement Tauri détecté');
        }
      } catch (error) {
        console.log('Environnement web standard détecté');
      }
    };

    checkTauri();
  }, []);

  // Basculer always-on-top (simulé pour l'instant)
  const toggleAlwaysOnTop = useCallback(async () => {
    if (!isTauriApp) {
      console.log('Always-on-top non disponible dans le navigateur');
      return false;
    }

    try {
      // Ici on appellerait l'API Tauri
      const newState = !isAlwaysOnTop;
      setIsAlwaysOnTop(newState);
      console.log(`Always-on-top ${newState ? 'activé' : 'désactivé'}`);
      return newState;
    } catch (error) {
      console.error('Erreur toggle always-on-top:', error);
      return isAlwaysOnTop;
    }
  }, [isTauriApp, isAlwaysOnTop]);

  // Définir la position de la fenêtre
  const setPosition = useCallback(async (x: number, y: number) => {
    if (!isTauriApp) return false;

    try {
      setWindowPosition({ x, y });
      console.log(`Position fenêtre: ${x}, ${y}`);
      return true;
    } catch (error) {
      console.error('Erreur positionnement fenêtre:', error);
      return false;
    }
  }, [isTauriApp]);

  // Minimiser vers le system tray
  const minimizeToTray = useCallback(async () => {
    if (!isTauriApp) {
      console.log('System tray non disponible dans le navigateur');
      return false;
    }

    try {
      console.log('Minimisation vers system tray');
      return true;
    } catch (error) {
      console.error('Erreur minimisation:', error);
      return false;
    }
  }, [isTauriApp]);

  // Restaurer depuis le system tray
  const restoreFromTray = useCallback(async () => {
    if (!isTauriApp) return false;

    try {
      console.log('Restauration depuis system tray');
      return true;
    } catch (error) {
      console.error('Erreur restauration:', error);
      return false;
    }
  }, [isTauriApp]);

  // Basculer la visibilité de la fenêtre
  const toggleVisibility = useCallback(async () => {
    if (!isTauriApp) return false;

    try {
      console.log('Basculement visibilité fenêtre');
      return true;
    } catch (error) {
      console.error('Erreur toggle visibilité:', error);
      return false;
    }
  }, [isTauriApp]);

  // Fermer complètement l'application
  const quitApp = useCallback(async () => {
    if (!isTauriApp) {
      console.log('Fermeture app non disponible dans le navigateur');
      return;
    }

    try {
      console.log('Fermeture application Tauri');
    } catch (error) {
      console.error('Erreur fermeture app:', error);
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