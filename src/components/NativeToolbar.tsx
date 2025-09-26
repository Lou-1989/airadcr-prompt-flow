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
    <div className="toolbar flex items-center justify-between px-2 sm:px-4 py-2 min-h-[48px]">
      {/* Left section - App info and controls */}
      <div className="flex items-center gap-2 sm:gap-4 min-w-0">
        <div className="flex items-center gap-2 min-w-0">
          <div className="w-6 h-6 bg-medical-blue rounded flex items-center justify-center flex-shrink-0">
            <Zap className="w-4 h-4 text-white" />
          </div>
          <span className="font-semibold text-sm truncate">AirADCR</span>
          <Badge variant="outline" className="text-xs hidden sm:flex">
            {isTauriApp ? 'Native' : 'Web'}
          </Badge>
        </div>

        <Separator orientation="vertical" className="h-6 hidden md:block" />

        <div className="flex items-center gap-1 sm:gap-2">
          <Button
            size="sm"
            variant={isAlwaysOnTop ? "default" : "ghost"}
            className="toolbar-button px-2 sm:px-3"
            onClick={toggleAlwaysOnTop}
          >
            {isAlwaysOnTop ? (
              <>
                <PinOff className="w-4 h-4" />
                <span className="hidden lg:inline ml-1">Désépingler</span>
              </>
            ) : (
              <>
                <Pin className="w-4 h-4" />
                <span className="hidden lg:inline ml-1">Épingler</span>
              </>
            )}
          </Button>

          <Button
            size="sm"
            variant="ghost"
            className="toolbar-button px-2 sm:px-3 hidden md:flex"
            onClick={() => {}}
          >
            <Settings className="w-4 h-4" />
            <span className="hidden lg:inline ml-1">Paramètres</span>
          </Button>
        </div>
      </div>

      {/* Center section - Active window info (responsive) */}
      <div className="flex items-center gap-2 sm:gap-4 min-w-0 flex-1 justify-center">
        {activeWindow && (
          <div className="flex items-center gap-2 px-2 sm:px-3 py-1 bg-secondary/50 rounded-md min-w-0 max-w-xs lg:max-w-md">
            <Monitor className="w-4 h-4 text-muted-foreground flex-shrink-0" />
            <div className="text-sm min-w-0">
              <span className="font-medium">{activeWindow.appName}</span>
              <span className="text-muted-foreground ml-2 truncate hidden sm:inline">
                {activeWindow.windowTitle.length > 20 
                  ? `${activeWindow.windowTitle.substring(0, 20)}...`
                  : activeWindow.windowTitle
                }
              </span>
            </div>
            {activeWindow.isCompatible ? (
              <Badge variant="secondary" className="bg-success/20 text-green-400 text-xs hidden sm:flex">
                ✓
              </Badge>
            ) : (
              <Badge variant="secondary" className="bg-warning/20 text-yellow-400 text-xs hidden sm:flex">
                !
              </Badge>
            )}
          </div>
        )}

        <div className={`status-indicator ${isOnline ? 'online' : 'offline'} hidden sm:flex`}>
          {isOnline ? (
            <>
              <Wifi className="w-4 h-4" />
              <span className="hidden md:inline">En ligne</span>
            </>
          ) : (
            <>
              <WifiOff className="w-4 h-4" />
              <span className="hidden md:inline">Hors ligne</span>
            </>
          )}
        </div>
      </div>

      {/* Right section - Window controls */}
      <div className="flex items-center gap-1 flex-shrink-0">
        <Button
          size="sm"
          variant="ghost"
          className="toolbar-button w-8 h-8 p-0 hidden sm:flex"
          onClick={handleMinimize}
        >
          <Minimize2 className="w-4 h-4" />
        </Button>
        
        <Button
          size="sm"
          variant="ghost"
          className="toolbar-button w-8 h-8 p-0 hidden sm:flex"
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