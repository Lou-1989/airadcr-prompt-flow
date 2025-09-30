import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { toast } from "sonner";
import Index from "./pages/Index";
import NotFound from "./pages/NotFound";
import { useTauriWindow } from "@/hooks/useTauriWindow";
import { InjectionProvider, useInjectionContext } from "@/contexts/InjectionContext";
import { useSecureMessaging } from "@/hooks/useSecureMessaging";
import { DebugPanel } from "@/components/DebugPanel";
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

  // Active le système de communication sécurisée postMessage
  useSecureMessaging();

  // Fonction de test pour le debug panel
  const handleTestInjection = async () => {
    const testText = "Test d'injection AirADCR - " + new Date().toLocaleTimeString();
    await performInjection(testText);
    toast.info('Test d\'injection lancé');
  };

  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Index />} />
          {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </BrowserRouter>
      
      {/* Debug Panel */}
      <DebugPanel
        isTauriApp={isTauriApp}
        isAlwaysOnTop={isAlwaysOnTop}
        isLocked={isLocked}
        isMonitoring={isMonitoring}
        onToggleAlwaysOnTop={toggleAlwaysOnTop}
        onTestInjection={handleTestInjection}
        onLockPosition={lockCurrentPosition}
        onUnlockPosition={unlockPosition}
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
