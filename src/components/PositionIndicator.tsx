import { useInjection } from '@/hooks/useInjection';

export const PositionIndicator = () => {
  const { externalPositions, isMonitoring } = useInjection();
  
  const lastPosition = externalPositions[0];
  const isPositionRecent = lastPosition && (Date.now() - lastPosition.timestamp) < 30000;
  
  return (
    <div className="fixed top-4 right-4 bg-background/90 border rounded-lg p-3 shadow-lg backdrop-blur-sm">
      <div className="flex items-center gap-2 text-sm">
        <div className={`w-2 h-2 rounded-full ${
          isMonitoring && isPositionRecent 
            ? 'bg-green-500 animate-pulse' 
            : isMonitoring 
              ? 'bg-yellow-500' 
              : 'bg-red-500'
        }`} />
        <span className="text-muted-foreground">
          {isMonitoring && isPositionRecent 
            ? `Position: (${lastPosition.x}, ${lastPosition.y})` 
            : isMonitoring 
              ? 'En attente de position...' 
              : 'Surveillance arrêtée'
          }
        </span>
      </div>
      {lastPosition && (
        <div className="text-xs text-muted-foreground mt-1">
          Dernière capture: {new Date(lastPosition.timestamp).toLocaleTimeString()}
        </div>
      )}
    </div>
  );
};