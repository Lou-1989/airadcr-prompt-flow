import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Index from "./pages/Index";
import NotFound from "./pages/NotFound";
import { useTauriWindow } from "@/hooks/useTauriWindow";
import { InjectionProvider, useInjectionContext } from "@/contexts/InjectionContext";

import { useGlobalShortcuts } from "@/hooks/useGlobalShortcuts";
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

// Communication sécurisée gérée dans WebViewContainer uniquement (pour éviter les doublons)


  // Fonction de test pour le debug panel
  const handleTestInjection = async () => {
    const testText = "Test d'injection AirADCR - " + new Date().toLocaleTimeString();
    await performInjection(testText);
    logger.debug('Test d\'injection lancé');
  };

  // Raccourcis clavier globaux
  const { isDebugVisible, setIsDebugVisible, isLogWindowVisible, setIsLogWindowVisible } = useGlobalShortcuts({
    onTestInjection: handleTestInjection,
    isTauriApp,
  });

  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Index />} />
          {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
      
      {/* Debug Panel - Accessible par Ctrl+Shift+D */}
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

      {/* Fenêtre de logs flottante - Accessible par Ctrl+Shift+L */}
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
