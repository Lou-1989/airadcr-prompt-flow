#!/usr/bin/env node

const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const SOURCE_IMAGE = path.join(__dirname, '../src/assets/airadcr-logo.png');
const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');

// Créer le dossier d'icônes
if (!fs.existsSync(ICONS_DIR)) {
  fs.mkdirSync(ICONS_DIR, { recursive: true });
}

async function createAllIcons() {
  console.log('🎯 Création des icônes Tauri compatibles...');
  
  try {
    // Lire l'image source
    if (!fs.existsSync(SOURCE_IMAGE)) {
      throw new Error(`Image source introuvable: ${SOURCE_IMAGE}`);
    }

    const sourceBuffer = fs.readFileSync(SOURCE_IMAGE);
    console.log(`📂 Source: ${SOURCE_IMAGE}`);
    
    // Configuration des icônes PNG
    const icons = [
      { size: 32, name: '32x32.png' },
      { size: 128, name: '128x128.png' },
      { size: 256, name: '128x128@2x.png' },
      { size: 512, name: 'icon.png' }
    ];
    
    // Générer chaque icône PNG
    for (const icon of icons) {
      const outputPath = path.join(ICONS_DIR, icon.name);
      
      await sharp(sourceBuffer)
        .resize(icon.size, icon.size, {
          fit: 'contain',
          background: { r: 255, g: 255, b: 255, alpha: 1 }
        })
        .png({
          compressionLevel: 9,
          adaptiveFiltering: true,
          palette: false,
          force: true
        })
        .ensureAlpha()
        .toFile(outputPath);
        
      console.log(`✅ Créé: ${icon.name} (${icon.size}x${icon.size})`);
    }
    
    // Créer un pseudo ICO (en fait un PNG renommé)
    // Windows accepte souvent les PNG renommés en .ico
    const icoPath = path.join(ICONS_DIR, 'icon.ico');
    const png32Buffer = await sharp(sourceBuffer)
      .resize(32, 32, {
        fit: 'contain', 
        background: { r: 255, g: 255, b: 255, alpha: 1 }
      })
      .png({ force: true, palette: false })
      .ensureAlpha()
      .toBuffer();
      
    fs.writeFileSync(icoPath, png32Buffer);
    console.log('✅ Créé: icon.ico (format PNG 32x32)');
    
    console.log('\n🎉 Toutes les icônes ont été générées!');
    console.log('📁 Dossier:', ICONS_DIR);
    console.log('\n🔧 Vous pouvez maintenant exécuter:');
    console.log('npx tauri build');
    
  } catch (error) {
    console.error('❌ Erreur:', error.message);
    process.exit(1);
  }
}

createAllIcons();