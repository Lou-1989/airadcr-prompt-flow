import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Index from "./pages/Index";
import NotFound from "./pages/NotFound";
import { useTauriWindow } from "@/hooks/useTauriWindow";
import { InjectionProvider, useInjectionContext } from "@/contexts/InjectionContext";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

import { DebugPanel } from "@/components/DebugPanel";
import { DevLogWindow } from "@/components/DevLogWindow";
import { logger } from "@/utils/logger";

const queryClient = new QueryClient();

// Composant interne qui utilise le contexte
const AppContent = () => {
  const { 
    isTauriApp, 
    isAlwaysOnTop, 
    toggleAlwaysOnTop 
  } = useTauriWindow();
  
  const { 
    performInjection,
    testInjectionAvailability,
    lockCurrentPosition,
    unlockPosition,
    isLocked,
    isMonitoring
  } = useInjectionContext();

  // État pour les panneaux
  const [isDebugVisible, setIsDebugVisible] = useState(false);
  const [isLogWindowVisible, setIsLogWindowVisible] = useState(false);

  // Fonction de test pour le debug panel
  const handleTestInjection = async () => {
    const testText = "Test d'injection AirADCR - " + new Date().toLocaleTimeString();
    await performInjection(testText);
    logger.debug('Test d\'injection lancé');
  };

  // ✅ Écoute des événements Tauri (raccourcis globaux)
  useEffect(() => {
    if (!isTauriApp) return;

    const sendToIframe = (type: string, payload?: any) => {
      const iframe = document.querySelector('iframe[title="AirADCR"]') as HTMLIFrameElement;
      if (iframe?.contentWindow) {
        iframe.contentWindow.postMessage({ type, payload: payload || null }, 'https://airadcr.com');
        logger.debug(`[Shortcuts] Message envoyé à iframe: ${type}`, payload);
      } else {
        logger.error('[Shortcuts] Iframe AirADCR non trouvée');
      }
    };

    const unlistenPromises = [
      // Debug Panel: Ctrl+Alt+D (modifié de Ctrl+Shift+D)
      listen('airadcr:toggle_debug', () => {
        setIsDebugVisible(prev => !prev);
        logger.debug('[Shortcuts] Debug Panel toggled');
      }),
      
      // Log Window: Ctrl+Alt+L (modifié de Ctrl+Shift+L)
      listen('airadcr:toggle_logs', () => {
        setIsLogWindowVisible(prev => !prev);
        logger.debug('[Shortcuts] Log Window toggled');
      }),
      
      // Test Injection: Ctrl+Alt+I (modifié de Ctrl+Shift+T)
      listen('airadcr:test_injection', () => {
        handleTestInjection();
        logger.debug('[Shortcuts] Test injection déclenché');
      }),
      
      // Anti-Ghost: F9
      listen('airadcr:force_clickable', () => {
        invoke('set_ignore_cursor_events', { ignore: false })
          .then(() => {
            logger.info('[Shortcuts] 🔓 Click-through DÉSACTIVÉ (force)');
          })
          .catch(err => {
            logger.error('[Shortcuts] Erreur désactivation click-through:', err);
          });
      }),
      
      // NOTE: Les événements de dictée/injection (dictation_startstop, dictation_pause,
      // inject_raw, inject_structured) sont gérés UNIQUEMENT dans useSecureMessaging.ts
      // pour éviter les doublons de postMessage vers l'iframe.
    ];

    // Cleanup
    return () => {
      Promise.all(unlistenPromises).then(unlisteners => {
        unlisteners.forEach(unlisten => unlisten());
      });
    };
  }, [isTauriApp]);

  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Index />} />
          {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
      
      {/* Debug Panel - Accessible par Ctrl+Alt+D */}
      <DebugPanel
        isTauriApp={isTauriApp}
        isAlwaysOnTop={isAlwaysOnTop}
        isLocked={isLocked}
        isMonitoring={isMonitoring}
        isVisible={isDebugVisible}
        onToggleAlwaysOnTop={toggleAlwaysOnTop}
        onTestInjection={handleTestInjection}
        onLockPosition={lockCurrentPosition}
        onUnlockPosition={unlockPosition}
        onClose={() => setIsDebugVisible(false)}
      />

      {/* Fenêtre de logs flottante - Accessible par Ctrl+Alt+L */}
      <DevLogWindow 
        isVisible={isLogWindowVisible}
        onClose={() => setIsLogWindowVisible(false)}
      />
    </>
  );
};

const App = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <Toaster />
        <Sonner />
        <InjectionProvider>
          <AppContent />
        </InjectionProvider>
      </TooltipProvider>
    </QueryClientProvider>
  );
};

export default App;