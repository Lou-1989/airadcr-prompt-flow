import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { toast } from "sonner";
import Index from "./pages/Index";
import NotFound from "./pages/NotFound";
import { useTauriWindow } from "@/hooks/useTauriWindow";
import { useInjection } from "@/hooks/useInjection";
import { useClipboardBridge } from "@/hooks/useClipboardBridge";
import { DebugPanel } from "@/components/DebugPanel";
import { logger } from "@/utils/logger";

const queryClient = new QueryClient();

const App = () => {
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
  } = useInjection();

  // Clipboard Bridge avec auto-injection
  const clipboardBridge = useClipboardBridge(async (reportContent: string) => {
    try {
      logger.info('Auto-injection déclenchée par clipboard bridge');
      await performInjection(reportContent);
      toast.success('Rapport injecté automatiquement!', {
        description: `${reportContent.substring(0, 50)}...`
      });
    } catch (error) {
      logger.error('Erreur auto-injection:', error);
      toast.error('Erreur lors de l\'auto-injection');
    }
  });

  // Fonctions de test pour le debug panel
  const handleTestInjection = async () => {
    const testText = "Test d'injection AirADCR - " + new Date().toLocaleTimeString();
    await performInjection(testText);
    toast.info('Test d\'injection lancé');
  };

  const handleTestClipboard = async () => {
    const result = await clipboardBridge.testCurrentClipboard();
    if (result) {
      toast.success('Rapport AirADCR détecté dans le clipboard!');
    } else {
      toast.info('Aucun rapport AirADCR détecté');
    }
  };

  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <Toaster />
        <Sonner />
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
          isClipboardMonitoring={clipboardBridge.isMonitoring}
          detectedReports={clipboardBridge.detectedReports}
          onToggleAlwaysOnTop={toggleAlwaysOnTop}
          onTestInjection={handleTestInjection}
          onLockPosition={lockCurrentPosition}
          onUnlockPosition={unlockPosition}
          onTestClipboard={handleTestClipboard}
        />
      </TooltipProvider>
    </QueryClientProvider>
  );
};

export default App;
