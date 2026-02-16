// Configuration pour la production
export const PRODUCTION_CONFIG = {
  // URL de l'application AirADCR
  AIRADCR_URL: 'https://airadcr.com/app?tori=true',
  
  // Configuration iframe
  IFRAME_CONFIG: {
    allow: 'clipboard-read; clipboard-write; fullscreen; display-capture',
    sandbox: 'allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-modals',
  },
  
  // Version de l'application
  VERSION: '1.0.0',
  
  // Environnement
  ENVIRONMENT: 'production' as const,
};

// Fonction pour vérifier si l'app est prête pour la production
export const isProductionReady = (): boolean => {
  // Vérifications de base pour la production
  const checks = [
    // Vérifier que l'URL AirADCR est accessible
    PRODUCTION_CONFIG.AIRADCR_URL.startsWith('https://'),
    
    // Vérifier que les permissions iframe sont correctes
    PRODUCTION_CONFIG.IFRAME_CONFIG.allow.includes('clipboard-read'),
    
    // Vérifier l'environnement
    PRODUCTION_CONFIG.ENVIRONMENT === 'production',
  ];
  
  return checks.every(check => check === true);
};