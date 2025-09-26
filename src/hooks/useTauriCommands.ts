import { useCallback, useState, useEffect } from 'react';

// Mock Tauri API for development - will be replaced by actual Tauri imports
const mockTauriAPI = {
  invoke: async (command: string, payload?: any) => {
    console.log(`[Mock Tauri] Invoking command: ${command}`, payload);
    
    // Simulate async operations
    await new Promise(resolve => setTimeout(resolve, 100));
    
    switch (command) {
      case 'set_always_on_top':
        return { success: true };
      case 'inject_text_to_active_window':
        return { success: true, injected: true };
      case 'get_active_window_info':
        return {
          appName: 'Microsoft Word',
          windowTitle: 'Document1 - Word',
          processId: 1234,
          isCompatible: true
        };
      case 'focus_airadcr_window':
        return { success: true };
      default:
        throw new Error(`Unknown command: ${command}`);
    }
  }
};

// Check if running in Tauri environment
const isTauriApp = typeof window !== 'undefined' && (window as any).__TAURI__;

export interface ActiveWindowInfo {
  appName: string;
  windowTitle: string;
  processId: number;
  isCompatible: boolean;
}

export const useTauriCommands = () => {
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(false);
  const [activeWindow, setActiveWindow] = useState<ActiveWindowInfo | null>(null);
  const [isOnline, setIsOnline] = useState(false);

  // Get the appropriate API (real Tauri or mock)
  const api = isTauriApp ? (window as any).__TAURI__ : mockTauriAPI;

  const toggleAlwaysOnTop = useCallback(async () => {
    try {
      const newValue = !isAlwaysOnTop;
      await api.invoke('set_always_on_top', { onTop: newValue });
      setIsAlwaysOnTop(newValue);
      return { success: true };
    } catch (error) {
      console.error('Erreur always-on-top:', error);
      return { success: false, error };
    }
  }, [isAlwaysOnTop, api]);

  const injectTextToSystem = useCallback(async (text: string) => {
    try {
      const result = await api.invoke('inject_text_to_active_window', { text });
      return result.success;
    } catch (error) {
      console.error('Erreur injection:', error);
      return false;
    }
  }, [api]);

  const getActiveWindowInfo = useCallback(async (): Promise<ActiveWindowInfo | null> => {
    try {
      const windowInfo = await api.invoke('get_active_window_info');
      return windowInfo;
    } catch (error) {
      console.error('Erreur fenêtre active:', error);
      return null;
    }
  }, [api]);

  const focusAirADCRWindow = useCallback(async () => {
    try {
      await api.invoke('focus_airadcr_window');
      return { success: true };
    } catch (error) {
      console.error('Erreur focus fenêtre:', error);
      return { success: false, error };
    }
  }, [api]);

  // Simulate online status checking
  useEffect(() => {
    const checkOnlineStatus = () => {
      setIsOnline(navigator.onLine);
    };

    checkOnlineStatus();
    window.addEventListener('online', checkOnlineStatus);
    window.addEventListener('offline', checkOnlineStatus);

    return () => {
      window.removeEventListener('online', checkOnlineStatus);
      window.removeEventListener('offline', checkOnlineStatus);
    };
  }, []);

  // Periodically check active window info
  useEffect(() => {
    const updateActiveWindow = async () => {
      const windowInfo = await getActiveWindowInfo();
      setActiveWindow(windowInfo);
    };

    updateActiveWindow();
    const interval = setInterval(updateActiveWindow, 2000);

    return () => clearInterval(interval);
  }, [getActiveWindowInfo]);

  return {
    isAlwaysOnTop,
    activeWindow,
    isOnline,
    isTauriApp,
    toggleAlwaysOnTop,
    injectTextToSystem,
    getActiveWindowInfo,
    focusAirADCRWindow,
  };
};