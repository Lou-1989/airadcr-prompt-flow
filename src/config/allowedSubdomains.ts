// Liste blanche des sous-domaines AirADCR autorisés
export const ALLOWED_SUBDOMAINS = {
  // Sous-domaines principaux
  PATHOLOGY: 'https://pathology.airadcr.com',
  TRANSCRIPTION: 'https://transcription.airadcr.com',
  FLASHMODE: 'https://flashmode.airadcr.com',
  
  // Domaines principaux
  MAIN: 'https://airadcr.com',
  WWW: 'https://www.airadcr.com',
} as const;

// Liste complète pour validation
export const ALL_ALLOWED_DOMAINS = Object.values(ALLOWED_SUBDOMAINS);

// Fonction utilitaire pour vérifier si un domaine est autorisé
export const isAllowedDomain = (url: string): boolean => {
  try {
    const urlObj = new URL(url);
    return (ALL_ALLOWED_DOMAINS as readonly string[]).includes(urlObj.origin);
  } catch {
    return false;
  }
};
