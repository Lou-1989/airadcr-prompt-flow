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
          {/* Content Navigation */}
          <div className="flex items-center justify-between p-3 bg-secondary/20 border-b border-border">
            <div className="flex items-center gap-4">
              <div className="flex bg-secondary/50 rounded-lg p-1">
                <Button
                  size="sm"
                  variant={activeView === 'webview' ? 'default' : 'ghost'}
                  onClick={() => setActiveView('webview')}
                  className="text-xs h-7"
                >
                  <Globe className="w-3 h-3 mr-1" />
                  AirADCR Web
                </Button>
                <Button
                  size="sm"
                  variant={activeView === 'system' ? 'default' : 'ghost'}
                  onClick={() => setActiveView('system')}
                  className="text-xs h-7"
                >
                  <Zap className="w-3 h-3 mr-1" />
                  Injection Système
                </Button>
              </div>
              
              {activeView === 'webview' && (
                <Badge variant="outline" className="text-xs">
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
                className="text-xs"
              >
                {isPanelOpen ? (
                  <>
                    <ChevronRight className="w-4 h-4 mr-1" />
                    Masquer le panneau
                  </>
                ) : (
                  <>
                    <ChevronLeft className="w-4 h-4 mr-1" />
                    Panneau de contrôle
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

        {/* Side Panel */}
        {isPanelOpen && (
          <>
            <Separator orientation="vertical" />
            <div className="w-80 bg-secondary/10 border-l border-border">
              <div className="p-4">
                <div className="flex items-center gap-2 mb-4">
                  <Settings className="w-4 h-4" />
                  <h3 className="font-semibold">Panneau de contrôle</h3>
                </div>
                
                <div className="space-y-4">
                  <div className="space-y-2">
                    <h4 className="text-sm font-medium text-muted-foreground">STATUT DE L'APPLICATION</h4>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span>Mode:</span>
                        <Badge variant="secondary">
                          {window.location.hostname === 'localhost' ? 'Développement' : 'Production'}
                        </Badge>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span>Environnement:</span>
                        <Badge variant="outline">React + Vite</Badge>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span>Tauri:</span>
                        <Badge variant="secondary">Simulé</Badge>
                      </div>
                    </div>
                  </div>

                  <Separator />

                  <div className="space-y-2">
                    <h4 className="text-sm font-medium text-muted-foreground">FONCTIONNALITÉS</h4>
                    <div className="space-y-2 text-xs text-muted-foreground">
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Interface utilisateur native</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Détection de fenêtre active</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-warning rounded-full"></div>
                        <span>Injection système (simulée)</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Gestion presse-papiers</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-2 h-2 bg-success rounded-full"></div>
                        <span>Always-on-top</span>
                      </div>
                    </div>
                  </div>

                  <Separator />

                  <div className="space-y-2">
                    <h4 className="text-sm font-medium text-muted-foreground">PROCHAINES ÉTAPES</h4>
                    <div className="space-y-2 text-xs text-muted-foreground">
                      <p>1. Intégrer avec Tauri Rust backend</p>
                      <p>2. Implémenter l'API Windows native</p>
                      <p>3. Configurer la WebView pour airadcr.com</p>
                      <p>4. Tester l'injection système réelle</p>
                      <p>5. Package et distribution</p>
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