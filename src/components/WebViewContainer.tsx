import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  RefreshCw, 
  ExternalLink, 
  Globe, 
  Shield,
  AlertCircle,
  CheckCircle
} from 'lucide-react';

interface WebViewContainerProps {
  className?: string;
}

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [url] = useState('https://airadcr.com');

  useEffect(() => {
    // Simulate loading time
    const timer = setTimeout(() => {
      setIsLoading(false);
    }, 2000);

    return () => clearTimeout(timer);
  }, []);

  const handleRefresh = () => {
    setIsLoading(true);
    setHasError(false);
    
    // Simulate refresh
    setTimeout(() => {
      setIsLoading(false);
    }, 1000);
  };

  const handleOpenExternal = () => {
    window.open(url, '_blank');
  };

  if (hasError) {
    return (
      <div className={`webview-container flex flex-col items-center justify-center ${className}`}>
        <div className="text-center space-y-4">
          <AlertCircle className="w-12 h-12 text-destructive mx-auto" />
          <div>
            <h3 className="text-lg font-semibold mb-2">Erreur de connexion</h3>
            <p className="text-muted-foreground mb-4">
              Impossible de charger AirADCR. Vérifiez votre connexion internet.
            </p>
          </div>
          <div className="flex gap-2">
            <Button onClick={handleRefresh} variant="default">
              <RefreshCw className="w-4 h-4 mr-2" />
              Réessayer
            </Button>
            <Button onClick={handleOpenExternal} variant="outline">
              <ExternalLink className="w-4 h-4 mr-2" />
              Ouvrir dans le navigateur
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`webview-container flex flex-col ${className}`}>
      {/* WebView Header */}
      <div className="flex items-center justify-between p-3 bg-secondary/30 border-b border-border">
        <div className="flex items-center gap-2">
          <Globe className="w-4 h-4 text-muted-foreground" />
          <span className="text-sm font-mono text-muted-foreground">{url}</span>
          <Badge variant="secondary" className="text-xs">
            <Shield className="w-3 h-3 mr-1" />
            HTTPS
          </Badge>
        </div>
        
        <div className="flex items-center gap-2">
          {!isLoading && (
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <CheckCircle className="w-3 h-3 text-success" />
              <span>Chargé</span>
            </div>
          )}
          
          <Button
            size="sm"
            variant="ghost"
            onClick={handleRefresh}
            disabled={isLoading}
            className="h-7 px-2"
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
          
          <Button
            size="sm"
            variant="ghost"
            onClick={handleOpenExternal}
            className="h-7 px-2"
          >
            <ExternalLink className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* WebView Content */}
      <div className="flex-1 relative bg-white">
        {isLoading ? (
          <div className="absolute inset-0 flex items-center justify-center bg-white">
            <div className="text-center space-y-4">
              <div className="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto" />
              <div>
                <p className="text-sm font-medium text-gray-700">Chargement d'AirADCR...</p>
                <p className="text-xs text-gray-500 mt-1">Connexion sécurisée via HTTPS</p>
              </div>
            </div>
          </div>
        ) : (
          <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
            <div className="text-center space-y-6 p-8">
              <div className="w-16 h-16 bg-gradient-to-r from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto">
                <Globe className="w-8 h-8 text-white" />
              </div>
              
              <div>
                <h2 className="text-2xl font-bold text-gray-800 mb-2">AirADCR WebView</h2>
                <p className="text-gray-600 max-w-md mx-auto">
                  Cette zone contiendra le site web AirADCR intégré via Tauri WebView. 
                  En mode développement, nous montrons cette interface de démonstration.
                </p>
              </div>

              <div className="space-y-2">
                <Badge variant="outline" className="text-sm">
                  URL: {url}
                </Badge>
                <div className="text-xs text-gray-500">
                  Mode: {window.location.hostname === 'localhost' ? 'Développement' : 'Production'}
                </div>
              </div>

              <Button onClick={handleOpenExternal} className="mt-4">
                <ExternalLink className="w-4 h-4 mr-2" />
                Ouvrir le vrai site
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};