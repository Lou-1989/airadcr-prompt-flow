import { useState, useEffect, useCallback, useRef } from 'react';
import { logger } from '@/utils/logger';

interface ClipboardBridgeState {
  isMonitoring: boolean;
  lastClipboardContent: string;
  detectedReports: number;
}

// Pattern pour détecter les rapports AirADCR
const AIRADCR_REPORT_PATTERNS = [
  /rapport\s+(radiologique|médical)/i,
  /examen\s+(radiologique|scanner|irm|échographie)/i,
  /conclusion\s*:/i,
  /impression\s*:/i,
  /airadcr/i,
  /patient\s*:/i,
  /technique\s*:/i,
  /analyse\s*:/i
];

export const useClipboardBridge = (onReportDetected?: (content: string) => void) => {
  const [state, setState] = useState<ClipboardBridgeState>({
    isMonitoring: false,
    lastClipboardContent: '',
    detectedReports: 0
  });

  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastContentRef = useRef<string>('');

  // Détecter si le contenu est un rapport AirADCR
  const isAirADCRReport = useCallback((content: string): boolean => {
    if (!content || content.length < 50) return false;
    
    // Compter les patterns qui correspondent
    const matchingPatterns = AIRADCR_REPORT_PATTERNS.filter(pattern => 
      pattern.test(content)
    ).length;
    
    // Au moins 2 patterns doivent correspondre pour considérer comme un rapport
    return matchingPatterns >= 2;
  }, []);

  // Surveillance du clipboard
  const monitorClipboard = useCallback(async () => {
    try {
      if (!navigator.clipboard || !navigator.clipboard.readText) {
        logger.warn('Clipboard API non disponible');
        return;
      }

      const clipboardContent = await navigator.clipboard.readText();
      
      // Vérifier si le contenu a changé
      if (clipboardContent !== lastContentRef.current && clipboardContent.length > 0) {
        lastContentRef.current = clipboardContent;
        
        setState(prev => ({
          ...prev,
          lastClipboardContent: clipboardContent
        }));

        // Vérifier si c'est un rapport AirADCR
        if (isAirADCRReport(clipboardContent)) {
          logger.info('Rapport AirADCR détecté dans le clipboard');
          
          setState(prev => ({
            ...prev,
            detectedReports: prev.detectedReports + 1
          }));

          // Déclencher le callback avec le contenu détecté
          if (onReportDetected) {
            onReportDetected(clipboardContent);
          }
        }
      }
    } catch (error) {
      // Erreur silencieuse car normale quand l'app n'a pas le focus
      if (error instanceof Error && !error.message.includes('Document is not focused')) {
        logger.warn('Erreur surveillance clipboard:', error.message);
      }
    }
  }, [isAirADCRReport, onReportDetected]);

  // Démarrer la surveillance
  const startMonitoring = useCallback(() => {
    if (intervalRef.current) return;

    logger.info('Démarrage surveillance clipboard bridge');
    setState(prev => ({ ...prev, isMonitoring: true }));
    
    // Surveillance toutes les 1000ms
    intervalRef.current = setInterval(monitorClipboard, 1000);
  }, [monitorClipboard]);

  // Arrêter la surveillance
  const stopMonitoring = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    
    logger.info('Arrêt surveillance clipboard bridge');
    setState(prev => ({ ...prev, isMonitoring: false }));
  }, []);

  // Test manuel pour vérifier le contenu actuel du clipboard
  const testCurrentClipboard = useCallback(async (): Promise<string | null> => {
    try {
      const content = await navigator.clipboard.readText();
      const isReport = isAirADCRReport(content);
      
      logger.info('Test clipboard:', {
        length: content.length,
        isReport,
        preview: content.substring(0, 100)
      });
      
      return isReport ? content : null;
    } catch (error) {
      logger.error('Erreur test clipboard:', error);
      return null;
    }
  }, [isAirADCRReport]);

  // Nettoyage au démontage
  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  // Démarrage automatique
  useEffect(() => {
    startMonitoring();
  }, [startMonitoring]);

  return {
    ...state,
    startMonitoring,
    stopMonitoring,
    testCurrentClipboard,
    isClipboardAvailable: !!(navigator.clipboard && navigator.clipboard.readText)
  };
};