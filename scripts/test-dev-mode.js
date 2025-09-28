#!/usr/bin/env node

const { spawn } = require('child_process');

console.log('🚀 Lancement du mode développement AirADCR Desktop\n');
console.log('📋 Test fonctionnel:');
console.log('  1. System tray (clic gauche pour afficher/masquer)');
console.log('  2. Menu context (clic droit sur tray)');
console.log('  3. Always on top toggle');
console.log('  4. Position de la souris');
console.log('  5. Injection de texte\n');

const tauriDev = spawn('npx', ['tauri', 'dev'], {
  stdio: 'inherit',
  shell: true
});

tauriDev.on('close', (code) => {
  console.log(`\nMode dev terminé avec le code: ${code}`);
});

tauriDev.on('error', (error) => {
  console.error('Erreur lors du lancement:', error.message);
});