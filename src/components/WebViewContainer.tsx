import { cn } from '@/lib/utils';
import { PRODUCTION_CONFIG } from '@/config/production';
import { SECURITY_CONFIG, validateAirADCRUrl } from '@/security/SecurityConfig';
import { useSecureMessaging } from '@/hooks/useSecureMessaging';
import { useInjectionContext } from '@/contexts/InjectionContext';
import { useInteractionMode } from '@/hooks/useInteractionMode';
import { useEffect, useState, useRef, useCallback } from 'react';
import { logger } from '@/utils/logger';
import { listen } from '@tauri-apps/api/event';


interface WebViewContainerProps {
  className?: string;
}

/** Extract tid from a URL string */
const extractTid = (url: string): string | null => {
  try {
    const u = new URL(url);
    return u.searchParams.get('tid');
  } catch {
    // Fallback regex for relative URLs
    const match = url.match(/[?&]tid=([^&]+)/);
    return match ? decodeURIComponent(match[1]) : null;
  }
};

export const WebViewContainer = ({ className }: WebViewContainerProps) => {
  const [isSecureUrl, setIsSecureUrl] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [retryCount, setRetryCount] = useState(0);
  const [currentUrl, setCurrentUrl] = useState(PRODUCTION_CONFIG.AIRADCR_URL);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const { sendSecureMessage } = useSecureMessaging();
  const { isInjecting } = useInjectionContext();
  const { isInteractionMode } = useInteractionMode(isInjecting);

  const MAX_AUTO_RETRIES = 3;
  const RETRY_DELAY_MS = 5000;

  // Progress animation removed — spinner only

  // Validation de l'URL au chargement
  useEffect(() => {
    const isValid = validateAirADCRUrl(PRODUCTION_CONFIG.AIRADCR_URL);
    setIsSecureUrl(isValid);
    if (!isValid) {
      logger.error('[Sécurité] URL AirADCR non autorisée:', PRODUCTION_CONFIG.AIRADCR_URL);
    }
  }, []);

  // Écouter les événements de navigation depuis le serveur HTTP (RIS)
  useEffect(() => {
    const setupNavigationListener = async () => {
      try {
        const unlisten = await listen<string>('airadcr:navigate_to_report', (event) => {
          const tid = event.payload;
          logger.debug('[Navigation] Événement reçu: tid=' + tid);

          // Construire l'URL avec le tid + cache-buster pour forcer le rechargement
          const separator = PRODUCTION_CONFIG.AIRADCR_URL.includes('?') ? '&' : '?';
          const cacheBuster = `_r=${Date.now()}`;
          const newUrl = `${PRODUCTION_CONFIG.AIRADCR_URL}${separator}tid=${encodeURIComponent(tid)}&${cacheBuster}`;

          // Valider l'URL avant navigation
          if (validateAirADCRUrl(newUrl)) {
            logger.debug('[Navigation] Chargement rapport:', newUrl);
            // Splash screen pendant la transition
            setIsLoading(true);
            setLoadError(false);
            setRetryCount(0);
            setCurrentUrl(newUrl);
          } else {
            logger.error('[Navigation] URL non autorisée:', newUrl);
          }
        });

        return unlisten;
      } catch (error) {
        logger.error('[Navigation] Erreur setup listener:', error);
        return () => {};
      }
    };

    const unlistenPromise = setupNavigationListener();
    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [currentUrl]);

  // Auto-retry logic
  useEffect(() => {
    if (!loadError || retryCount >= MAX_AUTO_RETRIES) return;

    retryTimerRef.current = setTimeout(() => {
      logger.debug(`[Retry] Tentative automatique ${retryCount + 1}/${MAX_AUTO_RETRIES}`);
      setRetryCount(prev => prev + 1);
      setLoadError(false);
      setIsLoading(true);
      // Force iframe reload by appending a cache-buster
      setCurrentUrl(prev => {
        const url = new URL(prev, window.location.origin);
        url.searchParams.set('_r', String(Date.now()));
        return url.toString();
      });
    }, RETRY_DELAY_MS);

    return () => {
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current);
    };
  }, [loadError, retryCount]);

  // Gestion des erreurs de chargement
  const handleIframeError = useCallback(() => {
    setLoadError(true);
    logger.error('[Sécurité] Erreur de chargement de l\'iframe AirADCR');
  }, []);

  // Gestion du chargement réussi
  const handleIframeLoad = useCallback(() => {
    setLoadError(false);
    setIsLoading(false);
    setRetryCount(0);
    logger.debug('[Sécurité] Iframe AirADCR chargée avec succès');
  }, []);

  const handleManualRetry = useCallback(() => {
    setRetryCount(0);
    setLoadError(false);
    setIsLoading(true);
    setCurrentUrl(prev => {
      const url = new URL(prev, window.location.origin);
      url.searchParams.set('_r', String(Date.now()));
      return url.toString();
    });
  }, []);

  // Ne pas afficher l'iframe si l'URL n'est pas sécurisée
  if (!isSecureUrl) {
    return (
      <div className={cn("h-full w-full flex items-center justify-center bg-destructive/10", className)}>
        <div className="text-center p-6">
          <img
            src="/lovable-uploads/IMG_9255.png"
            alt="AirADCR Logo"
            className="h-16 w-auto mx-auto mb-4"
          />
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

  // Affichage en cas d'erreur de chargement (après épuisement des retries auto)
  if (loadError && retryCount >= MAX_AUTO_RETRIES) {
    return (
      <div className={cn("h-full w-full flex items-center justify-center bg-background", className)}>
        <div className="text-center p-6">
          <img
            src="/lovable-uploads/IMG_9255.png"
            alt="AirADCR Logo"
            className="h-16 w-auto mx-auto mb-4 opacity-60"
          />
          <h2 className="text-lg font-semibold text-foreground mb-2">
            Connexion impossible
          </h2>
          <p className="text-sm text-muted-foreground mb-4">
            Impossible de charger AirADCR. Vérifiez votre connexion internet.
          </p>
          <button
            onClick={handleManualRetry}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90 transition-colors"
          >
            Réessayer
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={cn("h-full w-full relative", className)}>
      {/* Splash Screen de chargement */}
      <div
        className={cn(
          "absolute inset-0 z-10 flex flex-col items-center justify-center bg-background transition-opacity duration-700 ease-out",
          isLoading ? "opacity-100" : "opacity-0 pointer-events-none"
        )}
      >
        {/* Minimal geometric loader — medical blue accent */}
        <div className="relative mb-10">
          <div className="h-12 w-12 rounded-full border-[3px] border-muted" />
          <div
            className="absolute inset-0 h-12 w-12 rounded-full border-[3px] border-transparent border-t-primary"
            style={{ animation: 'spin 1s cubic-bezier(0.4, 0, 0.2, 1) infinite' }}
          />
        </div>
        {loadError && retryCount < MAX_AUTO_RETRIES && (
          <p className="text-xs text-muted-foreground">
            Reconnexion… ({retryCount + 1}/{MAX_AUTO_RETRIES})
          </p>
        )}
      </div>

      {/* Iframe - toujours montée, invisible pendant le chargement */}
      <iframe
        ref={iframeRef}
        src={currentUrl}
        className={cn(
          "w-full h-full border-0 transition-opacity duration-500",
          isLoading ? "opacity-0" : "opacity-100"
        )}
        title="AirADCR"
        allow={SECURITY_CONFIG.IFRAME_SECURITY.allow}
        sandbox={SECURITY_CONFIG.IFRAME_SECURITY.sandbox}
        referrerPolicy={SECURITY_CONFIG.IFRAME_SECURITY.referrerPolicy}
        loading="eager"
        onLoad={handleIframeLoad}
        onError={handleIframeError}
        style={{
          colorScheme: 'normal',
          isolation: 'isolate'
        }}
      />
    </div>
  );
};
