import { useEffect, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { logger } from '@/utils/logger';

interface UseGlobalShortcutsProps {
  onTestInjection: () => void;
  isTauriApp: boolean;
}

export const useGlobalShortcuts = ({ onTestInjection, isTauriApp }: UseGlobalShortcutsProps) => {
  const [isDebugVisible, setIsDebugVisible] = useState(false);
  const [isLogWindowVisible, setIsLogWindowVisible] = useState(false);

  const handleKeyPress = useCallback((event: KeyboardEvent) => {
    // Ctrl+Shift+D: Toggle Debug Panel (F12 réservé au SpeechMike)
    if (event.ctrlKey && event.shiftKey && event.key === 'D') {
      event.preventDefault();
      setIsDebugVisible(prev => !prev);
      logger.debug('[Shortcuts] Debug Panel toggled');
      return;
    }

    // Ctrl+Shift+L: Toggle Log Window
    if (event.ctrlKey && event.shiftKey && event.key === 'L') {
      event.preventDefault();
      setIsLogWindowVisible(prev => !prev);
      logger.debug('[Shortcuts] Log Window toggled');
      return;
    }

    // F9: Force désactivation click-through (anti-fantôme)
    if (event.key === 'F9' && isTauriApp) {
      event.preventDefault();
      invoke('set_ignore_cursor_events', { ignore: false })
        .then(() => {
          logger.info('[Shortcuts] 🔓 Click-through DÉSACTIVÉ (force)');
        })
        .catch(err => {
          logger.error('[Shortcuts] Erreur désactivation click-through:', err);
        });
      return;
    }

    // Ctrl+Shift+T: Test injection rapide
    if (event.ctrlKey && event.shiftKey && event.key === 'T' && isTauriApp) {
      event.preventDefault();
      onTestInjection();
      logger.debug('[Shortcuts] Test injection déclenché');
      return;
    }
  }, [onTestInjection, isTauriApp]);

  useEffect(() => {
    document.addEventListener('keydown', handleKeyPress);
    logger.debug('[Shortcuts] Raccourcis clavier activés: Ctrl+Shift+D (Debug), Ctrl+Shift+L (Logs), F9 (Anti-fantôme), Ctrl+Shift+T (Test)');
    
    return () => {
      document.removeEventListener('keydown', handleKeyPress);
    };
  }, [handleKeyPress]);

  return {
    isDebugVisible,
    setIsDebugVisible,
    isLogWindowVisible,
    setIsLogWindowVisible,
  };
};
