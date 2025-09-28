#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔧 Tests locaux AirADCR Desktop\n');

// 1. Validation des icônes
console.log('📋 ÉTAPE 1: Validation des icônes');
try {
  execSync('node scripts/validate-icons.cjs', { stdio: 'inherit' });
  console.log('✅ Validation des icônes: SUCCÈS\n');
} catch (error) {
  console.error('❌ Validation des icônes: ÉCHEC');
  console.error('Erreur:', error.message);
  process.exit(1);
}

// 2. Build Rust/Cargo
console.log('📋 ÉTAPE 2: Build Cargo (Rust)');
try {
  process.chdir('src-tauri');
  execSync('cargo build', { stdio: 'inherit' });
  process.chdir('..');
  console.log('✅ Build Cargo: SUCCÈS\n');
} catch (error) {
  console.error('❌ Build Cargo: ÉCHEC');
  console.error('Erreur:', error.message);
  process.chdir('..');
  process.exit(1);
}

// 3. Build Tauri complet
console.log('📋 ÉTAPE 3: Build Tauri complet');
try {
  execSync('npx tauri build', { stdio: 'inherit' });
  console.log('✅ Build Tauri: SUCCÈS\n');
} catch (error) {
  console.error('❌ Build Tauri: ÉCHEC');
  console.error('Erreur:', error.message);
  process.exit(1);
}

console.log('🎉 TOUS LES TESTS SONT PASSÉS !');
console.log('✅ Prêt pour le push vers GitHub Actions');