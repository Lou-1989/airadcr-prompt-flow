#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Script de validation des icônes pour éviter les erreurs de compilation Tauri
 * Vérifie les signatures binaires PNG et ICO
 */

const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');

// Signatures binaires attendues
const PNG_SIGNATURE = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
const ICO_SIGNATURE = Buffer.from([0x00, 0x00, 0x01, 0x00]);

// Configuration des icônes attendues
const EXPECTED_ICONS = [
  { file: '32x32.png', type: 'PNG', minSize: 32 * 32 },
  { file: '128x128.png', type: 'PNG', minSize: 128 * 128 },
  { file: '128x128@2x.png', type: 'PNG', minSize: 256 * 256 },
  { file: 'icon.png', type: 'PNG', minSize: 256 * 256, optional: true },
  { file: 'icon.ico', type: 'ICO', minSize: 1000 }
];

/**
 * Vérifie la signature binaire d'un fichier
 */
function validateSignature(filePath, expectedSignature, fileType) {
  if (!fs.existsSync(filePath)) {
    return { valid: false, error: `Fichier manquant: ${filePath}` };
  }

  try {
    const buffer = fs.readFileSync(filePath);
    
    if (buffer.length < expectedSignature.length) {
      return { valid: false, error: `Fichier trop petit: ${filePath}` };
    }

    const actualSignature = buffer.subarray(0, expectedSignature.length);
    
    if (!actualSignature.equals(expectedSignature)) {
      return { 
        valid: false, 
        error: `Signature ${fileType} invalide dans ${filePath}. Attendu: ${expectedSignature.toString('hex')}, obtenu: ${actualSignature.toString('hex')}` 
      };
    }

    return { valid: true, size: buffer.length };
  } catch (error) {
    return { valid: false, error: `Erreur lecture ${filePath}: ${error.message}` };
  }
}

/**
 * Vérifie qu'un PNG est au format RGBA (colorType=6, bitDepth=8)
 */
function validatePngRgba(buffer, filePath) {
  try {
    // Vérifier que c'est un PNG valide d'abord
    if (buffer.length < 33) { // PNG signature (8) + IHDR chunk (25 minimum)
      return { valid: false, error: `Fichier PNG trop petit: ${filePath}` };
    }

    // Après la signature PNG (8 bytes), le premier chunk doit être IHDR
    const IHDR_OFFSET = 8;
    const chunkLength = buffer.readUInt32BE(IHDR_OFFSET);
    const chunkType = buffer.toString('ascii', IHDR_OFFSET + 4, IHDR_OFFSET + 8);
    
    if (chunkType !== 'IHDR' || chunkLength !== 13) {
      return { valid: false, error: `IHDR invalide dans ${filePath}` };
    }
    
    // Lire les paramètres IHDR
    const bitDepth = buffer[IHDR_OFFSET + 8 + 8];   // byte 16 (width=4, height=4, bitDepth=1)
    const colorType = buffer[IHDR_OFFSET + 8 + 9];  // byte 17
    
    if (bitDepth !== 8 || colorType !== 6) {
      return { 
        valid: false, 
        error: `PNG non RGBA dans ${filePath}: bitDepth=${bitDepth}, colorType=${colorType} (attendu: bitDepth=8, colorType=6 pour RGBA)`
      };
    }
    
    return { valid: true, bitDepth, colorType };
  } catch (error) {
    return { valid: false, error: `Erreur validation RGBA ${filePath}: ${error.message}` };
  }
}

/**
 * Valide toutes les icônes
 */
function validateAllIcons() {
  console.log('🔍 Validation des icônes Tauri...\n');

  let hasErrors = false;
  let totalIcons = 0;
  let validIcons = 0;

  for (const iconConfig of EXPECTED_ICONS) {
    const filePath = path.join(ICONS_DIR, iconConfig.file);
    const signature = iconConfig.type === 'PNG' ? PNG_SIGNATURE : ICO_SIGNATURE;
    
    totalIcons++;
    
    const result = validateSignature(filePath, signature, iconConfig.type);
    
    if (!result.valid) {
      if (iconConfig.optional && result.error.includes('Fichier manquant')) {
        console.log(`⚠️  ${iconConfig.file} (optionnel): Ignoré`);
        totalIcons--; // Ne pas compter les optionnels manquants
        continue;
      }
      
      console.error(`❌ ${result.error}`);
      hasErrors = true;
    } else {
      // Validation RGBA spécifique pour les PNG
      if (iconConfig.type === 'PNG') {
        const buffer = fs.readFileSync(filePath);
        const rgbaResult = validatePngRgba(buffer, filePath);
        
        if (!rgbaResult.valid) {
          console.error(`❌ ${rgbaResult.error}`);
          hasErrors = true;
          continue;
        } else {
          console.log(`✅ ${iconConfig.file}: PNG RGBA valide (${result.size} bytes, bitDepth=${rgbaResult.bitDepth}, colorType=${rgbaResult.colorType})`);
        }
      }
      
      validIcons++;
      
      if (result.size < iconConfig.minSize) {
        console.warn(`⚠️  ${iconConfig.file}: Taille potentiellement trop petite (${result.size} bytes)`);
      } else if (iconConfig.type !== 'PNG') {
        console.log(`✅ ${iconConfig.file}: Signature ${iconConfig.type} valide (${result.size} bytes)`);
      }
    }
  }

  console.log(`\n📊 Résumé: ${validIcons}/${totalIcons} icônes valides`);

  if (hasErrors) {
    console.error('\n❌ Validation échouée: Des icônes ont des signatures invalides');
    console.error('Cela causera des erreurs "Invalid PNG signature" ou "Invalid ICO format" dans Tauri');
    process.exit(1);
  } else {
    console.log('\n✅ Toutes les icônes sont valides pour la compilation Tauri');
    process.exit(0);
  }
}

// Exécution
if (require.main === module) {
  validateAllIcons();
}

module.exports = { validateAllIcons, validateSignature };