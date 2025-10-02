import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { logger } from '@/utils/logger';
import { useInjectionContext } from '@/contexts/InjectionContext';

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
      
      <CardContent className="space-y-4">
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
        </div>
      </CardContent>
    </Card>
  );
};