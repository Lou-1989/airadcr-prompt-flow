import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  RefreshCw, 
  ArrowLeft, 
  ArrowRight,
  Home,
  Lock
} from 'lucide-react';

interface WebViewContainerProps {
  className?: string;
}

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  const [currentUrl] = useState('https://airadcr.com');
  const [isLoading, setIsLoading] = useState(false);

  const handleRefresh = () => {
    setIsLoading(true);
    // Refresh the iframe
    const iframe = document.getElementById('airadcr-webview') as HTMLIFrameElement;
    if (iframe) {
      iframe.src = iframe.src;
    }
    setTimeout(() => setIsLoading(false), 1000);
  };

  const handleNavigation = (direction: 'back' | 'forward' | 'home') => {
    const iframe = document.getElementById('airadcr-webview') as HTMLIFrameElement;
    if (iframe) {
      if (direction === 'back') {
        iframe.contentWindow?.history.back();
      } else if (direction === 'forward') {
        iframe.contentWindow?.history.forward();
      } else if (direction === 'home') {
        iframe.src = 'https://airadcr.com';
      }
    }
  };

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Browser Navigation Bar */}
      <div className="flex items-center gap-2 p-2 bg-secondary/10 border-b border-border min-h-[50px]">
        <div className="flex items-center gap-1">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleNavigation('back')}
            className="w-8 h-8 p-0 hidden sm:flex"
          >
            <ArrowLeft className="w-4 h-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleNavigation('forward')}
            className="w-8 h-8 p-0 hidden sm:flex"
          >
            <ArrowRight className="w-4 h-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={handleRefresh}
            className="w-8 h-8 p-0"
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleNavigation('home')}
            className="w-8 h-8 p-0"
          >
            <Home className="w-4 h-4" />
          </Button>
        </div>

        <div className="flex-1 flex items-center gap-2 bg-background rounded-md px-2 sm:px-3 py-2 border min-w-0">
          <Lock className="w-4 h-4 text-green-500 flex-shrink-0" />
          <span className="text-xs sm:text-sm font-mono truncate">{currentUrl}</span>
        </div>

        <Badge variant="secondary" className="bg-green-500/20 text-green-600 hidden sm:flex">
          Sécurisé
        </Badge>
      </div>

      {/* WebView Content Area - Direct airadcr.com integration */}
      <div className="flex-1 webview-container">
        <iframe
          id="airadcr-webview"
          src="https://airadcr.com"
          className="w-full h-full border-0"
          title="AirADCR Web Interface"
          allow="clipboard-read; clipboard-write"
          sandbox="allow-same-origin allow-scripts allow-forms allow-navigation allow-popups"
        />
      </div>
    </div>
  );
};