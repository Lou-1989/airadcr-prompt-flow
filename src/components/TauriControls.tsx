import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useTauriWindow } from '@/hooks/useTauriWindow';
import { 
  Pin, 
  PinOff, 
  Minimize2, 
  Maximize2, 
  Eye, 
  EyeOff, 
  X,
  Settings,
  Info
} from 'lucide-react';

export const TauriControls: React.FC = () => {
  const {
    isTauriApp,
    isAlwaysOnTop,
    systemInfo,
    toggleAlwaysOnTop,
    minimizeToTray,
    toggleVisibility,
    quitApp,
  } = useTauriWindow();

  // N'afficher les contrôles que dans l'environnement Tauri
  if (!isTauriApp) return null;

  return (
    <div className="fixed top-4 right-4 z-50 bg-background/95 backdrop-blur-sm border rounded-lg p-3 shadow-lg">
      <div className="flex items-center gap-2 mb-2">
        <Badge variant="secondary" className="text-xs">
          AirADCR Desktop v{systemInfo?.version || '1.0.0'}
        </Badge>
        {isAlwaysOnTop && (
          <Badge variant="default" className="text-xs">
            Always On Top
          </Badge>
        )}
      </div>
      
      <Separator className="mb-2" />
      
      <div className="flex items-center gap-1">
        {/* Always-on-top toggle */}
        <Button
          variant={isAlwaysOnTop ? "default" : "outline"}
          size="sm"
          onClick={toggleAlwaysOnTop}
          className="h-8 w-8 p-0"
          title={isAlwaysOnTop ? "Désactiver Always-on-top" : "Activer Always-on-top"}
        >
          {isAlwaysOnTop ? <Pin className="h-3 w-3" /> : <PinOff className="h-3 w-3" />}
        </Button>

        {/* Minimize to tray */}
        <Button
          variant="outline"
          size="sm"
          onClick={minimizeToTray}
          className="h-8 w-8 p-0"
          title="Minimiser vers la barre système"
        >
          <Minimize2 className="h-3 w-3" />
        </Button>

        {/* Toggle visibility */}
        <Button
          variant="outline"
          size="sm"
          onClick={toggleVisibility}
          className="h-8 w-8 p-0"
          title="Basculer la visibilité"
        >
          <Eye className="h-3 w-3" />
        </Button>

        <Separator orientation="vertical" className="h-6 mx-1" />

        {/* Quit application */}
        <Button
          variant="outline"
          size="sm"
          onClick={quitApp}
          className="h-8 w-8 p-0 text-destructive hover:text-destructive-foreground hover:bg-destructive"
          title="Fermer l'application"
        >
          <X className="h-3 w-3" />
        </Button>
      </div>

      {/* Info section */}
      <div className="mt-2 pt-2 border-t">
        <div className="text-xs text-muted-foreground space-y-1">
          <div className="flex justify-between">
            <span>Plateforme:</span>
            <span>{systemInfo?.platform}</span>
          </div>
          <div className="flex justify-between">
            <span>Architecture:</span>
            <span>{systemInfo?.arch}</span>
          </div>
        </div>
        
        {/* Raccourcis clavier */}
        <div className="mt-2 text-xs text-muted-foreground">
          <div className="font-medium mb-1">Raccourcis:</div>
          <div>Ctrl+Alt+T: Always-on-top</div>
          <div>Ctrl+Alt+H: Masquer/Afficher</div>
        </div>
      </div>
    </div>
  );
};