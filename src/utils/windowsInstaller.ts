// Utilitaires pour l'installation Windows de l'application AirADCR

export const WINDOWS_CONFIG = {
  // Configuration pour PWA (Progressive Web App)
  PWA_CONFIG: {
    name: 'AirADCR Desktop',
    short_name: 'AirADCR',
    description: 'Application desktop AirADCR avec injection système',
    theme_color: '#3b82f6', // Bleu médical AirADCR
    background_color: '#ffffff',
    display: 'standalone',
    start_url: '/?source=pwa',
    scope: '/',
    orientation: 'any',
  },
  
  // Arguments Chrome/Edge pour mode app
  BROWSER_ARGS: [
    '--app={{URL}}',
    '--window-size=1200,800',
    '--disable-web-security',
    '--user-data-dir={{USER_DATA_DIR}}',
    '--disable-features=TranslateUI',
    '--disable-extensions',
    '--no-first-run',
    '--disable-default-apps',
  ],
  
  // Chemins Windows standards
  PATHS: {
    CHROME: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
    EDGE: 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
    DESKTOP: '%USERPROFILE%\\Desktop',
    STARTUP: '%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\Startup',
    APP_DATA: '%APPDATA%\\AirADCR',
  },
} as const;

// Génération du manifest PWA
export const generatePWAManifest = (baseUrl: string) => {
  return {
    ...WINDOWS_CONFIG.PWA_CONFIG,
    start_url: `${baseUrl}/?source=pwa`,
    scope: baseUrl,
    icons: [
      {
        src: '/favicon.ico',
        sizes: '16x16 32x32 48x48',
        type: 'image/x-icon',
      },
      {
        src: '/icon-192.png',
        sizes: '192x192',
        type: 'image/png',
        purpose: 'any maskable',
      },
      {
        src: '/icon-512.png',
        sizes: '512x512',
        type: 'image/png',
        purpose: 'any maskable',
      },
    ],
    screenshots: [
      {
        src: '/screenshot-desktop.png',
        sizes: '1200x800',
        type: 'image/png',
        form_factor: 'wide',
      },
    ],
  };
};

// Instructions d'installation pour l'utilisateur
export const getInstallationInstructions = () => {
  return {
    chrome: {
      title: 'Installation via Chrome',
      steps: [
        'Ouvrir Google Chrome',
        'Naviguer vers l\'application AirADCR',
        'Cliquer sur l\'icône "Installer" dans la barre d\'adresse',
        'Confirmer l\'installation',
        'L\'application apparaît dans le menu Démarrer Windows',
      ],
    },
    edge: {
      title: 'Installation via Edge',
      steps: [
        'Ouvrir Microsoft Edge',
        'Naviguer vers l\'application AirADCR',
        'Cliquer sur "..." puis "Applications" > "Installer cette application"',
        'Confirmer l\'installation',
        'L\'application apparaît dans le menu Démarrer Windows',
      ],
    },
    manual: {
      title: 'Installation manuelle (raccourci)',
      steps: [
        'Télécharger le script PowerShell d\'installation',
        'Clic-droit > "Exécuter avec PowerShell"',
        'Suivre les instructions à l\'écran',
        'Le raccourci est créé sur le bureau',
      ],
    },
  };
};

// Détection des capacités du navigateur
export const detectBrowserCapabilities = () => {
  const userAgent = navigator.userAgent;
  const isChrome = /Chrome/.test(userAgent) && !/Edg/.test(userAgent);
  const isEdge = /Edg/.test(userAgent);
  const isWindows = /Windows/.test(userAgent);
  
  // Vérification du support PWA
  const supportsPWA = 'serviceWorker' in navigator && 'PushManager' in window;
  
  // Vérification du support clipboard
  const supportsClipboard = 'clipboard' in navigator;
  
  return {
    browser: isChrome ? 'chrome' : isEdge ? 'edge' : 'other',
    isWindows,
    supportsPWA,
    supportsClipboard,
    canInstall: (isChrome || isEdge) && isWindows && supportsPWA,
  };
};

// Vérification si l'app est déjà installée comme PWA
export const isInstalledPWA = (): boolean => {
  return window.matchMedia('(display-mode: standalone)').matches ||
         (window.navigator as any).standalone === true;
};

// Hook pour gérer l'installation PWA
export const usePWAInstall = () => {
  // Cette logique serait implémentée dans un hook React
  // pour gérer l'événement beforeinstallprompt
  return {
    canInstall: false,
    install: () => Promise.resolve(),
    isInstalled: isInstalledPWA(),
  };
};