import { cn } from '@/lib/utils';
import { PRODUCTION_CONFIG } from '@/config/production';
import { SECURITY_CONFIG, validateAirADCRUrl } from '@/security/SecurityConfig';
import { useSecureMessaging } from '@/hooks/useSecureMessaging';
import { useEffect, useState } from 'react';

interface WebViewContainerProps {
  className?: string;
}

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  const [isSecureUrl, setIsSecureUrl] = useState(false);
  const [loadError, setLoadError] = useState(false);
  const { sendSecureMessage } = useSecureMessaging();
  
  // Validation de l'URL au chargement
  useEffect(() => {
    const isValid = validateAirADCRUrl(PRODUCTION_CONFIG.AIRADCR_URL);
    setIsSecureUrl(isValid);
    
    if (!isValid) {
      console.error('[Sécurité] URL AirADCR non autorisée:', PRODUCTION_CONFIG.AIRADCR_URL);
    }
  }, []);
  
  // Gestion des erreurs de chargement
  const handleIframeError = () => {
    setLoadError(true);
    console.error('[Sécurité] Erreur de chargement de l\'iframe AirADCR');
  };
  
  // Gestion du chargement réussi
  const handleIframeLoad = () => {
    setLoadError(false);
    console.log('[Sécurité] Iframe AirADCR chargée avec succès');
  };
  
  // Ne pas afficher l'iframe si l'URL n'est pas sécurisée
  if (!isSecureUrl) {
    return (
      <div className={cn("h-full w-full flex items-center justify-center bg-destructive/10", className)}>
        <div className="text-center p-6">
          <h2 className="text-lg font-semibold text-destructive mb-2">
            🛡️ Erreur de Sécurité
          </h2>
          <p className="text-sm text-muted-foreground">
            L'URL AirADCR n'est pas autorisée pour des raisons de sécurité.
          </p>
        </div>
      </div>
    );
  }
  
  // Affichage en cas d'erreur de chargement
  if (loadError) {
    return (
      <div className={cn("h-full w-full flex items-center justify-center bg-warning/10", className)}>
        <div className="text-center p-6">
          <h2 className="text-lg font-semibold text-warning mb-2">
            ⚠️ Erreur de Chargement
          </h2>
          <p className="text-sm text-muted-foreground mb-4">
            Impossible de charger AirADCR. Vérifiez votre connexion internet.
          </p>
          <button 
            onClick={() => setLoadError(false)}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm"
          >
            Réessayer
          </button>
        </div>
      </div>
    );
  }
  
  return (
    <div className={cn("h-full w-full", className)}>
      <iframe
        src={PRODUCTION_CONFIG.AIRADCR_URL}
        className="w-full h-full border-0"
        title="AirADCR"
        allow={SECURITY_CONFIG.IFRAME_SECURITY.allow}
        sandbox={SECURITY_CONFIG.IFRAME_SECURITY.sandbox}
        referrerPolicy={SECURITY_CONFIG.IFRAME_SECURITY.referrerPolicy}
        loading="eager"
        onLoad={handleIframeLoad}
        onError={handleIframeError}
        // Protection supplémentaire contre le clickjacking
        style={{ 
          colorScheme: 'normal',
          isolation: 'isolate' 
        }}
      />
    </div>
  );
};