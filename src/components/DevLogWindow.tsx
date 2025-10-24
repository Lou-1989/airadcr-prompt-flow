import { useState, useEffect, useRef } from 'react';
import { logger, LogEntry } from '@/utils/logger';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { X, Copy, Trash2 } from 'lucide-react';
import { toast } from 'sonner';

interface DevLogWindowProps {
  isVisible: boolean;
  onClose: () => void;
}

export const DevLogWindow = ({ isVisible, onClose }: DevLogWindowProps) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filterLevel, setFilterLevel] = useState<string>('all');
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unsubscribe = logger.subscribe((entry) => {
      setLogs(prev => [...prev.slice(-99), entry]); // Garder les 100 derniers
    });

    return unsubscribe;
  }, []);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const copyAllLogs = () => {
    const text = logs
      .map(log => `[${log.timestamp}] [${log.level.toUpperCase()}] ${log.message}`)
      .join('\n');
    navigator.clipboard.writeText(text);
    toast.success('Logs copiés dans le presse-papier');
  };

  const clearLogs = () => {
    setLogs([]);
    toast.success('Logs effacés');
  };

  const filteredLogs = filterLevel === 'all' 
    ? logs 
    : logs.filter(log => log.level === filterLevel);

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error': return 'text-destructive';
      case 'warn': return 'text-yellow-500';
      case 'info': return 'text-blue-500';
      case 'debug': return 'text-muted-foreground';
      default: return 'text-foreground';
    }
  };

  if (!isVisible) return null;

  return (
    <div className="fixed bottom-4 right-4 w-[500px] bg-background/95 backdrop-blur-sm border rounded-lg shadow-xl z-[9999]">
      <div className="flex items-center justify-between p-3 border-b">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-semibold">Dev Logs</h3>
          <div className="flex gap-1">
            <Button
              size="sm"
              variant={filterLevel === 'all' ? 'default' : 'ghost'}
              onClick={() => setFilterLevel('all')}
              className="h-6 px-2 text-xs"
            >
              All
            </Button>
            <Button
              size="sm"
              variant={filterLevel === 'error' ? 'default' : 'ghost'}
              onClick={() => setFilterLevel('error')}
              className="h-6 px-2 text-xs"
            >
              Error
            </Button>
            <Button
              size="sm"
              variant={filterLevel === 'warn' ? 'default' : 'ghost'}
              onClick={() => setFilterLevel('warn')}
              className="h-6 px-2 text-xs"
            >
              Warn
            </Button>
            <Button
              size="sm"
              variant={filterLevel === 'debug' ? 'default' : 'ghost'}
              onClick={() => setFilterLevel('debug')}
              className="h-6 px-2 text-xs"
            >
              Debug
            </Button>
          </div>
        </div>
        <div className="flex gap-1">
          <Button
            size="sm"
            variant="ghost"
            onClick={clearLogs}
            className="h-6 w-6 p-0"
            title="Effacer les logs"
          >
            <Trash2 className="h-3 w-3" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={copyAllLogs}
            className="h-6 w-6 p-0"
            title="Copier tous les logs"
          >
            <Copy className="h-3 w-3" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={onClose}
            className="h-6 w-6 p-0"
            title="Fermer (Ctrl+Shift+L)"
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      </div>
      
      <ScrollArea className="h-[300px] p-3">
        <div ref={scrollRef} className="space-y-1">
          {filteredLogs.length === 0 ? (
            <p className="text-xs text-muted-foreground">Aucun log pour le moment...</p>
          ) : (
            filteredLogs.map((log, idx) => (
              <div key={idx} className="text-xs font-mono">
                <span className="text-muted-foreground">[{log.timestamp}]</span>{' '}
                <span className={getLevelColor(log.level)}>[{log.level.toUpperCase()}]</span>{' '}
                <span>{log.message}</span>
                {log.args && log.args.length > 0 && (
                  <span className="text-muted-foreground"> {JSON.stringify(log.args)}</span>
                )}
              </div>
            ))
          )}
        </div>
      </ScrollArea>
      
      <div className="p-2 border-t bg-muted/50">
        <p className="text-xs text-muted-foreground">
          💡 Appuyez sur <kbd className="px-1 py-0.5 text-xs bg-background border rounded">Ctrl+Shift+L</kbd> pour masquer/afficher • <kbd className="px-1 py-0.5 text-xs bg-background border rounded">F12</kbd> pour SpeechMike (prod) / DevTools (dev)
        </p>
      </div>
    </div>
  );
};
