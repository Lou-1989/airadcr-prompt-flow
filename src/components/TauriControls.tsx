import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useTauriWindow } from '@/hooks/useTauriWindow';
import { invoke } from '@tauri-apps/api';
import { toast } from 'sonner';
import { 
  Pin, 
  PinOff, 
  Minimize2, 
  Maximize2, 
  Eye, 
  EyeOff, 
  X,
  Settings,
  Info,
  FileText,
  FolderOpen
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

  const handleOpenLogFolder = async () => {
    try {
      await invoke('open_log_folder');
      toast.success('Dossier de logs ouvert');
    } catch (error) {
      toast.error('Erreur lors de l\'ouverture du dossier de logs');
      console.error(error);
    }
  };

  const handleCopyLogPath = async () => {
    try {
      const logPath = await invoke<string>('get_log_path');
      await navigator.clipboard.writeText(logPath);
      toast.success('Chemin des logs copié dans le presse-papiers');
    } catch (error) {
      toast.error('Erreur lors de la copie du chemin');
      console.error(error);
    }
  };

  // N'afficher les contrôles que dans l'environnement Tauri
  if (!isTauriApp) return null;

  return (
    <div className="fixed top-4 right-4 z-50 bg-background/95 backdrop-blur-sm border rounded-lg p-3 shadow-lg">
      <div className="flex items-center gap-2 mb-2">
        <Badge variant="secondary" className="text-xs">
          AirADCR Desktop v{systemInfo?.version || '1.0.0'}
        </Badge>
        <Badge variant="default" className="text-xs">
          <Pin className="h-3 w-3 mr-1" />
          Always On Top
        </Badge>
      </div>
      
      <Separator className="mb-2" />
      
      <div className="flex items-center gap-1">
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
          <div>Ctrl+Alt+H: Masquer/Afficher</div>
          <div>Ctrl+Alt+Q: Fermer</div>
        </div>

        {/* Boutons de logs */}
        <Separator className="my-2" />
        <div className="flex gap-1">
          <Button
            variant="outline"
            size="sm"
            onClick={handleCopyLogPath}
            className="h-7 flex-1 text-xs"
            title="Copier le chemin des logs"
          >
            <FileText className="h-3 w-3 mr-1" />
            Copier logs
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleOpenLogFolder}
            className="h-7 flex-1 text-xs"
            title="Ouvrir le dossier des logs"
          >
            <FolderOpen className="h-3 w-3 mr-1" />
            Ouvrir
          </Button>
        </div>
      </div>
    </div>
  );
};