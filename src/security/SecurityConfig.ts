// Configuration de sécurité pour l'application AirADCR
import { logger } from '@/utils/logger';
export const SECURITY_CONFIG = {
  // URLs autorisées pour l'iframe et l'application Desktop
  ALLOWED_ORIGINS: [
    'https://airadcr.com',
    'https://www.airadcr.com',
    'https://pathology.airadcr.com',
    'https://transcription.airadcr.com',
    'https://flashmode.airadcr.com',
    'tauri://localhost',      // Application Tauri (v1)
    'https://tauri.localhost', // Application Tauri (v2)
  ] as const,
  
  // Content Security Policy
  CSP_DIRECTIVES: {
    'default-src': ["'self'"],
    'script-src': ["'self'", "'unsafe-inline'", 'https://fonts.googleapis.com'],
    'style-src': ["'self'", "'unsafe-inline'", 'https://fonts.googleapis.com'],
    'font-src': ["'self'", 'https://fonts.gstatic.com'],
    'img-src': ["'self'", 'data:', 'https:'],
    'connect-src': ["'self'", 'https://airadcr.com', 'https://pathology.airadcr.com', 'https://transcription.airadcr.com', 'https://flashmode.airadcr.com'],
    'frame-src': ['https://airadcr.com', 'https://pathology.airadcr.com', 'https://transcription.airadcr.com', 'https://flashmode.airadcr.com'],
    'frame-ancestors': ["'none'"], // Protection contre clickjacking
  },
  
  // Configuration iframe sécurisée
  IFRAME_SECURITY: {
    // Permissions strictement nécessaires
    allow: 'clipboard-read; clipboard-write; fullscreen; microphone; camera; autoplay',
    // Sandbox avec permissions minimales incluant clipboard
    sandbox: 'allow-same-origin allow-scripts allow-forms allow-navigation allow-popups allow-clipboard-read allow-clipboard-write allow-modals',
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
    'airadcr:injection_ack', // Acknowledgment immédiat de la requête d'injection
    'airadcr:injection_status', // Statut final de l'injection (success/fail + reason)
    
    // 🆕 Commandes clavier globales (Desktop → Iframe)
    'airadcr:toggle_recording',       // Start/Stop dictée (Ctrl+F10)
    'airadcr:toggle_pause',            // Pause/Resume (Ctrl+F9)
    'airadcr:request_injection',       // Demande injection brut/structuré (Ctrl+F11/F12)
    'airadcr:finalize_and_inject',     // Finaliser + injecter (F12 SpeechMike)
    
    // SpeechMike commands (Desktop → Web)
    'airadcr:speechmike_record', // Démarre ou reprend l'enregistrement
    'airadcr:speechmike_pause', // Met en pause l'enregistrement
    'airadcr:speechmike_finish', // Termine l'enregistrement et déclenche transcription
    
    // Recording notifications (Web → Desktop)
    'airadcr:recording_started', // Notification: enregistrement démarré
    'airadcr:recording_paused', // Notification: enregistrement en pause
    'airadcr:recording_finished', // Notification: enregistrement terminé
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
  const allowedOrigins = SECURITY_CONFIG.ALLOWED_ORIGINS as readonly string[];
  
  // Accepter les messages depuis l'application Tauri elle-même
  const isTauriInternal = event.origin.startsWith('tauri://') || 
                          event.origin === 'https://tauri.localhost';
  const isAirADCR = allowedOrigins.includes(event.origin);
  
  if (!isTauriInternal && !isAirADCR) {
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