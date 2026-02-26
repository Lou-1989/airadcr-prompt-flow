import { useState, useCallback, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { logger, LogLevel } from '@/utils/logger';
import { useInjectionContext } from '@/contexts/InjectionContext';
import { Label } from '@/components/ui/label';
import { DatabaseTab } from './DatabaseTab';
import { TeoHubConfig } from './TeoHubConfig';
import { Settings, Database, Server, ShieldAlert, ShieldCheck } from 'lucide-react';

interface RuntimeInfo {
  disable_api_auth: boolean;
  http_port: number;
  config_path: string;
  teo_hub_enabled: boolean;
  build_version: string;
}

interface DebugPanelProps {
  isTauriApp: boolean;
  isAlwaysOnTop: boolean;
  isLocked: boolean;
  isMonitoring: boolean;
  isVisible: boolean;
  onToggleAlwaysOnTop: () => void;
  onTestInjection: () => void;
  onLockPosition: () => void;
  onUnlockPosition: () => void;
  onClose: () => void;
}

export const DebugPanel = ({
  isTauriApp,
  isAlwaysOnTop,
  isLocked,
  isMonitoring,
  isVisible,
  onToggleAlwaysOnTop,
  onTestInjection,
  onLockPosition,
  onUnlockPosition,
  onClose
}: DebugPanelProps) => {
  const { activeWindow, lockedPosition } = useInjectionContext();
  const [iframeTestResult, setIframeTestResult] = useState<'idle' | 'testing' | 'success' | 'fail'>('idle');
  const [runtimeInfo, setRuntimeInfo] = useState<RuntimeInfo | null>(null);

  useEffect(() => {
    if (!isTauriApp) return;
    const load = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/tauri');
        const info = await invoke<RuntimeInfo>('get_runtime_info');
        setRuntimeInfo(info);
      } catch (err) {
        logger.error('[Debug] Erreur chargement runtime info', err);
      }
    };
    load();
  }, [isTauriApp]);

  const testIframeCommunication = useCallback(() => {
    setIframeTestResult('testing');
    const iframe = document.querySelector('iframe[title="AirADCR"]') as HTMLIFrameElement;
    if (!iframe?.contentWindow) {
      setIframeTestResult('fail');
      logger.error('[Debug] Iframe AirADCR non trouvée');
      return;
    }

    const handler = (event: MessageEvent) => {
      if (event.origin === 'https://airadcr.com' && event.data?.type === 'airadcr:status') {
        setIframeTestResult('success');
        window.removeEventListener('message', handler);
        logger.info('[Debug] ✅ Iframe répond !', event.data.payload);
      }
    };
    window.addEventListener('message', handler);
    iframe.contentWindow.postMessage({ type: 'airadcr:request_status', payload: null }, 'https://airadcr.com');
    logger.debug('[Debug] postMessage airadcr:request_status envoyé');

    setTimeout(() => {
      window.removeEventListener('message', handler);
      setIframeTestResult(prev => prev === 'testing' ? 'fail' : prev);
    }, 3000);
  }, []);

  const getStatusBadge = (status: boolean, label: string) => (
    <div className="flex items-center justify-between p-2 bg-background/50 rounded">
      <span className="text-sm font-medium">{label}</span>
      <Badge variant={status ? "default" : "secondary"}>
        {status ? "ACTIF" : "INACTIF"}
      </Badge>
    </div>
  );

  const testFunction = async (name: string, testFn: () => void | Promise<void>) => {
    try {
      logger.info(`Test démarré: ${name}`);
      await testFn();
      logger.info(`Test réussi: ${name}`);
    } catch (error) {
      logger.error(`Test échoué: ${name}`, error);
    }
  };

  if (!isVisible) {
    return null;
  }

  return (
    <Card className="fixed top-4 right-4 w-80 z-50 bg-background/95 backdrop-blur border shadow-lg">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Panel Debug AirADCR (F12 pour fermer)</CardTitle>
          <Button
            onClick={onClose}
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
          >
            ×
          </Button>
        </div>
      </CardHeader>
      
      <CardContent className="p-0">
        <Tabs defaultValue="system" className="w-full">
          <TabsList className="w-full grid grid-cols-3 h-8 mx-3 mb-2" style={{ width: 'calc(100% - 24px)' }}>
            <TabsTrigger value="system" className="text-xs h-7 gap-1">
              <Settings className="w-3 h-3" />
              Système
            </TabsTrigger>
            <TabsTrigger value="database" className="text-xs h-7 gap-1">
              <Database className="w-3 h-3" />
              BDD
            </TabsTrigger>
            <TabsTrigger value="teo_hub" className="text-xs h-7 gap-1">
              <Server className="w-3 h-3" />
              TÉO Hub
            </TabsTrigger>
          </TabsList>
          
          {/* Onglet Système */}
          <TabsContent value="system" className="px-4 pb-4 mt-0 space-y-4">
            {/* État Runtime */}
            {runtimeInfo && (
              <div className="space-y-2">
                <h4 className="text-sm font-semibold">État Runtime</h4>
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between p-2 bg-background/50 rounded">
                    <span className="text-sm font-medium flex items-center gap-1.5">
                      {runtimeInfo.disable_api_auth ? (
                        <ShieldAlert className="w-3.5 h-3.5 text-destructive" />
                      ) : (
                        <ShieldCheck className="w-3.5 h-3.5 text-primary" />
                      )}
                      Auth API
                    </span>
                    <Badge 
                      variant={runtimeInfo.disable_api_auth ? "destructive" : "default"}
                      className={runtimeInfo.disable_api_auth ? "animate-pulse" : ""}
                    >
                      {runtimeInfo.disable_api_auth ? "DÉSACTIVÉE" : "ACTIVÉE"}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between p-2 bg-background/50 rounded">
                    <span className="text-sm font-medium">Port HTTP</span>
                    <span className="text-xs font-mono">{runtimeInfo.http_port}</span>
                  </div>
                  <div className="flex items-center justify-between p-2 bg-background/50 rounded">
                    <span className="text-sm font-medium">Config</span>
                    <span className="text-[10px] font-mono truncate max-w-[160px]" title={runtimeInfo.config_path}>
                      {runtimeInfo.config_path.split(/[/\\]/).slice(-2).join('/')}
                    </span>
                  </div>
                  <div className="flex items-center justify-between p-2 bg-background/50 rounded">
                    <span className="text-sm font-medium">Version</span>
                    <span className="text-xs font-mono">{runtimeInfo.build_version}</span>
                  </div>
                </div>
              </div>
            )}

            <Separator />

            {/* Status des fonctions */}
            <div className="space-y-2">
              <h4 className="text-sm font-semibold">État des Fonctions</h4>
              {getStatusBadge(isTauriApp, "Tauri App")}
              {getStatusBadge(isAlwaysOnTop, "Always-On-Top")}
              {getStatusBadge(isLocked, "Position Verrouillée")}
              {getStatusBadge(isMonitoring, "Surveillance Position")}
            </div>

            <Separator />

            {/* Fenêtre active */}
            {activeWindow && (
              <div className="space-y-2">
                <h4 className="text-sm font-semibold">Fenêtre Active</h4>
                <div className="text-xs space-y-1 p-2 bg-background/50 rounded">
                  <div><span className="font-medium">App:</span> {activeWindow.app_name}</div>
                  <div><span className="font-medium">Titre:</span> {activeWindow.title.substring(0, 30)}...</div>
                  <div><span className="font-medium">Position:</span> ({activeWindow.x}, {activeWindow.y})</div>
                  <div><span className="font-medium">Taille:</span> {activeWindow.width} × {activeWindow.height}</div>
                </div>
              </div>
            )}

            {/* Position verrouillée */}
            {isLocked && lockedPosition && (
              <div className="space-y-2">
                <h4 className="text-sm font-semibold">Position Verrouillée</h4>
                <div className="text-xs space-y-1 p-2 bg-background/50 rounded">
                  <div><span className="font-medium">App:</span> {lockedPosition.application}</div>
                  {lockedPosition.relativePosition && (
                    <>
                      <div><span className="font-medium">Relative:</span> ({lockedPosition.relativePosition.relative_x}, {lockedPosition.relativePosition.relative_y})</div>
                      <div><span className="font-medium">Absolue:</span> ({lockedPosition.x}, {lockedPosition.y})</div>
                    </>
                  )}
                </div>
              </div>
            )}

            <Separator />

            {/* Raccourcis clavier */}
            <div className="space-y-2">
              <h4 className="text-sm font-semibold">Raccourcis Clavier</h4>
              <div className="text-xs space-y-1 p-2 bg-background/50 rounded">
                <div><kbd className="px-1 py-0.5 text-xs bg-background border rounded">Ctrl+Alt+D</kbd> : Ouvrir/Fermer Debug Panel</div>
                <div><kbd className="px-1 py-0.5 text-xs bg-background border rounded">Ctrl+Alt+L</kbd> : Ouvrir/Fermer Fenêtre Logs</div>
                <div><kbd className="px-1 py-0.5 text-xs bg-background border rounded">Ctrl+Alt+I</kbd> : Test injection rapide</div>
                <div><kbd className="px-1 py-0.5 text-xs bg-background border rounded">F9</kbd> : Désactiver click-through (anti-fantôme)</div>
              </div>
            </div>

            <Separator />

            {/* Configuration Logs */}
            <div className="space-y-2">
              <Label className="text-sm font-semibold">Niveau de Logs</Label>
              <div className="grid grid-cols-4 gap-1">
                <Button 
                  size="sm" 
                  variant="outline"
                  onClick={() => logger.setLevel(LogLevel.ERROR)}
                  className="text-xs h-7"
                >
                  ERROR
                </Button>
                <Button 
                  size="sm" 
                  variant="outline"
                  onClick={() => logger.setLevel(LogLevel.WARN)}
                  className="text-xs h-7"
                >
                  WARN
                </Button>
                <Button 
                  size="sm" 
                  variant="outline"
                  onClick={() => logger.setLevel(LogLevel.INFO)}
                  className="text-xs h-7"
                >
                  INFO
                </Button>
                <Button 
                  size="sm" 
                  variant="outline"
                  onClick={() => logger.setLevel(LogLevel.DEBUG)}
                  className="text-xs h-7"
                >
                  DEBUG
                </Button>
              </div>
            </div>

            <Separator />

            {/* Tests manuels */}
            <div className="space-y-2">
              <h4 className="text-sm font-semibold">Tests Manuels</h4>
              
              <Button
                onClick={() => testFunction("Always-On-Top", onToggleAlwaysOnTop)}
                variant="outline"
                size="sm"
                className="w-full justify-start"
                disabled={!isTauriApp}
              >
                🔝 Toggle Always-On-Top
              </Button>

              <Button
                onClick={() => testFunction("Injection", onTestInjection)}
                variant="outline"
                size="sm"
                className="w-full justify-start"
                disabled={!isTauriApp}
              >
                💉 Test Injection
              </Button>

              <div className="flex space-x-2">
                <Button
                  onClick={() => testFunction("Lock Position", onLockPosition)}
                  variant="outline"
                  size="sm"
                  className="flex-1"
                  disabled={!isTauriApp || isLocked}
                >
                  🔒 Lock
                </Button>
                <Button
                  onClick={() => testFunction("Unlock Position", onUnlockPosition)}
                  variant="outline"
                  size="sm"
                  className="flex-1"
                  disabled={!isTauriApp || !isLocked}
                >
                  🔓 Unlock
                </Button>
              </div>

              <Separator />

              {/* Test communication iframe */}
              <div className="space-y-2">
                <h4 className="text-sm font-semibold">Communication Iframe</h4>
                <Button
                  onClick={testIframeCommunication}
                  variant="outline"
                  size="sm"
                  className="w-full justify-start"
                  disabled={iframeTestResult === 'testing'}
                >
                  📡 Test communication iframe
                </Button>
                {iframeTestResult === 'testing' && (
                  <Badge variant="secondary" className="w-full justify-center">⏳ En attente de réponse...</Badge>
                )}
                {iframeTestResult === 'success' && (
                  <Badge variant="default" className="w-full justify-center bg-green-600">✅ Iframe répond</Badge>
                )}
                {iframeTestResult === 'fail' && (
                  <Badge variant="destructive" className="w-full justify-center">❌ Iframe ne répond pas (3s)</Badge>
                )}
              </div>
            </div>
          </TabsContent>
          
          {/* Onglet Base de données */}
          <TabsContent value="database" className="px-4 pb-4 mt-0">
            <DatabaseTab isTauriApp={isTauriApp} />
          </TabsContent>
          
          {/* Onglet TÉO Hub */}
          <TabsContent value="teo_hub" className="px-4 pb-4 mt-0">
            <TeoHubConfig isTauriApp={isTauriApp} />
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
};
