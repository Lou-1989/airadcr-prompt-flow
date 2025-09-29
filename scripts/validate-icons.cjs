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

const EXPECTED_ICONS = [
  { file: '32x32.png', type: 'PNG', minSize: 1000, dir: ICONS_DIR },
  { file: '128x128.png', type: 'PNG', minSize: 5000, dir: ICONS_DIR },
  { file: '128x128@2x.png', type: 'PNG', minSize: 10000, dir: ICONS_DIR },
  { file: 'icon.png', type: 'PNG', minSize: 20000, dir: ICONS_DIR },
  { file: 'icon.ico', type: 'ICO', minSize: 5000, dir: ICONS_DIR },
  { file: 'installer.ico', type: 'ICO', minSize: 5000, dir: path.join(__dirname, '../src-tauri/assets') }
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
  console.log('🔍 VALIDATION DES ICÔNES TAURI\n');

  let hasErrors = false;
  let totalIcons = 0;
  let validIcons = 0;

  for (const iconConfig of EXPECTED_ICONS) {
    const filePath = path.join(iconConfig.dir, iconConfig.file);
    const signature = iconConfig.type === 'PNG' ? PNG_SIGNATURE : ICO_SIGNATURE;
    
    totalIcons++;
    console.log(`📋 Validation: ${iconConfig.file} (${iconConfig.type})`);
    
    const result = validateSignature(filePath, signature, iconConfig.type);
    
    if (!result.valid) {
      if (iconConfig.optional && result.error.includes('Fichier manquant')) {
        console.log(`  ⚠️  Fichier optionnel manquant, ignoré`);
        totalIcons--; // Ne pas compter les optionnels manquants
        continue;
      }
      
      console.log(`  ❌ ERREUR: ${result.error}`);
      hasErrors = true;
      continue;
    }
    
    if (result.size < iconConfig.minSize) {
      console.log(`  ❌ ERREUR: Fichier trop petit (${result.size} < ${iconConfig.minSize} bytes)`);
      hasErrors = true;
      continue;
    }
    
    // Validation RGBA spécifique pour les PNG
    if (iconConfig.type === 'PNG') {
      const buffer = fs.readFileSync(filePath);
      const rgbaResult = validatePngRgba(buffer, filePath);
      
      if (!rgbaResult.valid) {
        console.log(`  ❌ ERREUR: ${rgbaResult.error}`);
        hasErrors = true;
        continue;
      }
      
      console.log(`  ✅ VALIDE: PNG RGBA (${result.size} bytes, ${rgbaResult.bitDepth}bit, colorType=${rgbaResult.colorType})`);
    } else {
      console.log(`  ✅ VALIDE: ICO authentique (${result.size} bytes)`);
    }
    
    validIcons++;
  }

  console.log(`\n📊 RÉSUMÉ DE VALIDATION:`);
  console.log(`   Total des icônes: ${totalIcons}`);
  console.log(`   Icônes valides: ${validIcons}`);
  console.log(`   Icônes invalides: ${totalIcons - validIcons}`);

  if (hasErrors) {
    console.log('\n❌ VALIDATION ÉCHOUÉE - Des erreurs ont été détectées');
    console.log('Cela causera des erreurs "Invalid PNG signature", "Invalid ICO format" ou "Failed to create app icon" dans Tauri');
    process.exit(1);
  } else {
    console.log('\n✅ VALIDATION RÉUSSIE - Toutes les icônes sont conformes');
    console.log('✅ Prêt pour npx tauri build');
    process.exit(0);
  }
}

// Exécution
if (require.main === module) {
  validateAllIcons();
}

module.exports = { validateAllIcons, validateSignature };