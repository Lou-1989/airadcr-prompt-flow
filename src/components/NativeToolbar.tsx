import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { 
  Pin, 
  PinOff, 
  Monitor, 
  Wifi, 
  WifiOff, 
  Settings,
  Minimize2,
  Maximize2,
  X,
  Zap
} from 'lucide-react';
import { useTauriCommands } from '@/hooks/useTauriCommands';

export const NativeToolbar = () => {
  const { 
    isAlwaysOnTop, 
    activeWindow, 
    isOnline,
    isTauriApp,
    toggleAlwaysOnTop,
    focusAirADCRWindow 
  } = useTauriCommands();

  const [isMinimized, setIsMinimized] = useState(false);

  const handleMinimize = () => {
    setIsMinimized(!isMinimized);
    // In real Tauri app, this would call window.minimize()
  };

  const handleMaximize = () => {
    // In real Tauri app, this would call window.toggleMaximize()
  };

  const handleClose = () => {
    // In real Tauri app, this would call window.close()
  };

  return (
    <div className="toolbar flex items-center justify-between px-4 py-2 min-h-[48px]">
      {/* Left section - App info and controls */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 bg-gradient-to-r from-blue-500 to-blue-600 rounded flex items-center justify-center">
            <Zap className="w-4 h-4 text-white" />
          </div>
          <span className="font-semibold text-sm">AirADCR Desktop</span>
          {isTauriApp ? (
            <Badge variant="secondary" className="text-xs">Native</Badge>
          ) : (
            <Badge variant="outline" className="text-xs">Demo</Badge>
          )}
        </div>

        <Separator orientation="vertical" className="h-6" />

        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant={isAlwaysOnTop ? "default" : "ghost"}
            className="toolbar-button"
            onClick={toggleAlwaysOnTop}
          >
            {isAlwaysOnTop ? (
              <>
                <PinOff className="w-4 h-4 mr-1" />
                Désépingler
              </>
            ) : (
              <>
                <Pin className="w-4 h-4 mr-1" />
                Épingler
              </>
            )}
          </Button>

          <Button
            size="sm"
            variant="ghost"
            className="toolbar-button"
            onClick={() => {}}
          >
            <Settings className="w-4 h-4 mr-1" />
            Paramètres
          </Button>
        </div>
      </div>

      {/* Center section - Active window info */}
      <div className="flex items-center gap-4">
        {activeWindow && (
          <div className="flex items-center gap-2 px-3 py-1 bg-secondary/50 rounded-md">
            <Monitor className="w-4 h-4 text-muted-foreground" />
            <div className="text-sm">
              <span className="font-medium">{activeWindow.appName}</span>
              <span className="text-muted-foreground ml-2">
                {activeWindow.windowTitle.length > 30 
                  ? `${activeWindow.windowTitle.substring(0, 30)}...`
                  : activeWindow.windowTitle
                }
              </span>
            </div>
            {activeWindow.isCompatible ? (
              <Badge variant="secondary" className="bg-success/20 text-green-400 text-xs">
                Compatible
              </Badge>
            ) : (
              <Badge variant="secondary" className="bg-warning/20 text-yellow-400 text-xs">
                Limité
              </Badge>
            )}
          </div>
        )}

        <div className={`status-indicator ${isOnline ? 'online' : 'offline'}`}>
          {isOnline ? (
            <>
              <Wifi className="w-4 h-4" />
              <span>En ligne</span>
            </>
          ) : (
            <>
              <WifiOff className="w-4 h-4" />
              <span>Hors ligne</span>
            </>
          )}
        </div>
      </div>

      {/* Right section - Window controls */}
      <div className="flex items-center gap-1">
        <Button
          size="sm"
          variant="ghost"
          className="toolbar-button w-8 h-8 p-0"
          onClick={handleMinimize}
        >
          <Minimize2 className="w-4 h-4" />
        </Button>
        
        <Button
          size="sm"
          variant="ghost"
          className="toolbar-button w-8 h-8 p-0"
          onClick={handleMaximize}
        >
          <Maximize2 className="w-4 h-4" />
        </Button>
        
        <Button
          size="sm"
          variant="ghost"
          className="toolbar-button w-8 h-8 p-0 hover:bg-red-500 hover:text-white"
          onClick={handleClose}
        >
          <X className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
};