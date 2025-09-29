#!/usr/bin/env node

const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const SOURCE_IMAGE = path.join(__dirname, '../src/assets/airadcr-logo.png');
const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');

// Créer le dossier d'icônes s'il n'existe pas
if (!fs.existsSync(ICONS_DIR)) {
  fs.mkdirSync(ICONS_DIR, { recursive: true });
}

async function generateTauriIcons() {
  console.log('🎯 Génération des icônes Tauri à partir du logo source...');
  
  try {
    // Lire l'image source
    const sourceBuffer = fs.readFileSync(SOURCE_IMAGE);
    
    // Icônes PNG requises par Tauri
    const pngSizes = [
      { size: 32, file: '32x32.png' },
      { size: 128, file: '128x128.png' },
      { size: 256, file: '128x128@2x.png' }, // @2x = 256x256
      { size: 512, file: 'icon.png' }
    ];
    
    // Générer toutes les icônes PNG
    for (const { size, file } of pngSizes) {
      const outputPath = path.join(ICONS_DIR, file);
      
      await sharp(sourceBuffer)
        .resize(size, size, { 
          fit: 'contain', 
          background: { r: 255, g: 255, b: 255, alpha: 0 } // Transparent
        })
        .png({
          compressionLevel: 9,
          adaptiveFiltering: true,
          palette: false,
          force: true
        })
        .ensureAlpha() // Force RGBA
        .toFile(outputPath);
        
      console.log(`✅ ${file} (${size}x${size})`);
    }
    
    // Générer l'icône ICO multi-résolution pour Windows
    const icoPath = path.join(ICONS_DIR, 'icon.ico');
    
    // Sharp ne peut pas créer des ICO, on utilise la version 256x256 PNG comme base
    const png256 = await sharp(sourceBuffer)
      .resize(256, 256, { 
        fit: 'contain',
        background: { r: 255, g: 255, b: 255, alpha: 0 }
      })
      .png({ force: true })
      .toBuffer();
    
    // Pour l'ICO, on copie temporairement le PNG 256x256
    fs.writeFileSync(icoPath.replace('.ico', '_temp.png'), png256);
    
    console.log('✅ Icônes générées avec succès!');
    console.log('\n📋 Fichiers créés:');
    console.log('  - 32x32.png (32x32)');
    console.log('  - 128x128.png (128x128)');  
    console.log('  - 128x128@2x.png (256x256)');
    console.log('  - icon.png (512x512)');
    console.log('  - icon.ico (à convertir manuellement depuis le PNG 256x256)');
    
    console.log('\n⚠️  Note: Conversion ICO requise manuellement ou via outil en ligne');
    
  } catch (error) {
    console.error('❌ Erreur lors de la génération:', error.message);
    process.exit(1);
  }
}

generateTauriIcons();