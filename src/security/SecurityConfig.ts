// Configuration de sécurité pour l'application AirADCR
import { logger } from '@/utils/logger';
export const SECURITY_CONFIG = {
  // URLs autorisées pour l'iframe
  ALLOWED_ORIGINS: [
    'https://airadcr.com',
    'https://www.airadcr.com',
  ] as const,
  
  // Content Security Policy
  CSP_DIRECTIVES: {
    'default-src': ["'self'"],
    'script-src': ["'self'", "'unsafe-inline'", 'https://fonts.googleapis.com'],
    'style-src': ["'self'", "'unsafe-inline'", 'https://fonts.googleapis.com'],
    'font-src': ["'self'", 'https://fonts.gstatic.com'],
    'img-src': ["'self'", 'data:', 'https:'],
    'connect-src': ["'self'", 'https://airadcr.com'],
    'frame-src': ['https://airadcr.com'],
    'frame-ancestors': ["'none'"], // Protection contre clickjacking
  },
  
  // Configuration iframe sécurisée
  IFRAME_SECURITY: {
    // Permissions strictement nécessaires
    allow: 'clipboard-read; clipboard-write; fullscreen',
    // Sandbox avec permissions minimales incluant clipboard
    sandbox: 'allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-clipboard-read allow-clipboard-write',
    // Politique de référent pour protéger les données
    referrerPolicy: 'strict-origin-when-cross-origin' as const,
  },
  
  // Messages autorisés entre iframe et parent
  ALLOWED_MESSAGE_TYPES: [
    'airadcr:ready',
    'airadcr:inject',
    'airadcr:status',
    'airadcr:lock',
    'airadcr:unlock',
    'airadcr:update_lock',
    'airadcr:lock_status', // Feedback du statut de verrouillage
    'airadcr:request_status', // Demande du statut initial
  ] as const,
} as const;

// Validation de l'URL AirADCR
export const validateAirADCRUrl = (url: string): boolean => {
  try {
    const urlObj = new URL(url);
    return (SECURITY_CONFIG.ALLOWED_ORIGINS as readonly string[]).includes(urlObj.origin);
  } catch {
    return false;
  }
};

// Génération du Content Security Policy
export const generateCSP = (): string => {
  const directives = Object.entries(SECURITY_CONFIG.CSP_DIRECTIVES)
    .map(([key, values]) => `${key} ${values.join(' ')}`)
    .join('; ');
  
  return directives;
};

// Validation des messages postMessage
export const isValidMessage = (event: MessageEvent): boolean => {
  // Vérifier l'origine
  if (!(SECURITY_CONFIG.ALLOWED_ORIGINS as readonly string[]).includes(event.origin)) {
    logger.warn('Message rejeté - origine non autorisée:', event.origin);
    return false;
  }
  
  // Vérifier le type de message
  const messageType = event.data?.type;
  if (!messageType || !(SECURITY_CONFIG.ALLOWED_MESSAGE_TYPES as readonly string[]).includes(messageType)) {
    logger.warn('Message rejeté - type non autorisé:', messageType);
    return false;
  }
  
  return true;
};