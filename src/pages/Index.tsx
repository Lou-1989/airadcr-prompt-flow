import { useState } from 'react';
import { NativeToolbar } from '@/components/NativeToolbar';
import { WebViewContainer } from '@/components/WebViewContainer';
import { SystemIntegrationPanel } from '@/components/SystemIntegrationPanel';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { 
  Globe, 
  Zap, 
  Settings, 
  ChevronLeft, 
  ChevronRight,
  Monitor
} from 'lucide-react';

const Index = () => {
  const [activeView, setActiveView] = useState<'webview' | 'system'>('webview');
  const [isPanelOpen, setIsPanelOpen] = useState(false);

  return (
    <div className="flex flex-col h-screen bg-background text-foreground overflow-hidden">
      {/* Native Toolbar */}
      <NativeToolbar />
      
      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Primary Content Area */}
        <div className="flex-1 flex flex-col">
          {/* Content Navigation - Responsive */}
          <div className="flex items-center justify-between p-2 sm:p-3 bg-secondary/20 border-b border-border">
            <div className="flex items-center gap-2 sm:gap-4">
              <div className="flex bg-secondary/50 rounded-lg p-1">
                <Button
                  size="sm"
                  variant={activeView === 'webview' ? 'default' : 'ghost'}
                  onClick={() => setActiveView('webview')}
                  className="text-xs h-7 px-2 sm:px-3"
                >
                  <Globe className="w-3 h-3 sm:mr-1" />
                  <span className="hidden sm:inline">AirADCR Web</span>
                </Button>
                <Button
                  size="sm"
                  variant={activeView === 'system' ? 'default' : 'ghost'}
                  onClick={() => setActiveView('system')}
                  className="text-xs h-7 px-2 sm:px-3"
                >
                  <Zap className="w-3 h-3 sm:mr-1" />
                  <span className="hidden sm:inline">Debug</span>
                </Button>
              </div>
              
              {activeView === 'webview' && (
                <Badge variant="outline" className="text-xs hidden md:flex">
                  <Monitor className="w-3 h-3 mr-1" />
                  WebView Intégrée
                </Badge>
              )}
            </div>

            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setIsPanelOpen(!isPanelOpen)}
                className="text-xs px-2 sm:px-3"
              >
                {isPanelOpen ? (
                  <>
                    <ChevronRight className="w-4 h-4" />
                    <span className="hidden sm:inline ml-1">Masquer</span>
                  </>
                ) : (
                  <>
                    <ChevronLeft className="w-4 h-4" />
                    <span className="hidden sm:inline ml-1">Debug</span>
                  </>
                )}
              </Button>
            </div>
          </div>

          {/* Main View */}
          <div className="flex-1">
            {activeView === 'webview' ? (
              <WebViewContainer className="h-full" />
            ) : (
              <SystemIntegrationPanel />
            )}
          </div>
        </div>

        {/* Side Panel - Responsive */}
        {isPanelOpen && (
          <>
            <Separator orientation="vertical" className="hidden sm:block" />
            <div className={`${isPanelOpen ? 'w-full sm:w-80' : 'w-0'} bg-secondary/10 border-l border-border sm:relative absolute inset-y-0 right-0 z-10`}>
              <div className="p-4 h-full overflow-y-auto">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <Settings className="w-4 h-4" />
                    <h3 className="font-semibold">Debug</h3>
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => setIsPanelOpen(false)}
                    className="sm:hidden"
                  >
                    <ChevronRight className="w-4 h-4" />
                  </Button>
                </div>
                
                <div className="space-y-4">
                  <div className="space-y-2">
                    <h4 className="text-sm font-medium text-muted-foreground">STATUT</h4>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span>Mode:</span>
                        <Badge variant="secondary" className="text-xs">
                          {window.location.hostname === 'localhost' ? 'Dev' : 'Prod'}
                        </Badge>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span>WebView:</span>
                        <Badge variant="outline" className="text-xs">Connectée</Badge>
                      </div>
                    </div>
                  </div>

                  <Separator />

                  <div className="space-y-2">
                    <h4 className="text-sm font-medium text-muted-foreground">FONCTIONNALITÉS</h4>
                    <div className="space-y-2 text-xs">
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Always-on-top</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Détection fenêtre</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>AirADCR intégré</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-warning rounded-full"></div>
                        <span>Injection (à venir)</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default Index;