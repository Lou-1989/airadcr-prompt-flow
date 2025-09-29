#!/usr/bin/env node

const sharp = require('sharp');
const pngToIco = require('png-to-ico');
const fs = require('fs');
const path = require('path');

const SOURCE_IMAGE = path.join(__dirname, '../src/assets/airadcr-logo.png');
const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');
const ASSETS_DIR = path.join(__dirname, '../src-tauri/assets');

// Configuration des icônes requises
const PNG_CONFIGS = [
  { size: 32, name: '32x32.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' },
  { size: 512, name: 'icon.png' }
];

const ICO_CONFIGS = [
  { sizes: [16, 24, 32, 48, 64, 128, 256], name: 'icon.ico', dir: ICONS_DIR },
  { sizes: [16, 24, 32, 48, 64, 128, 256], name: 'installer.ico', dir: ASSETS_DIR }
];

// Créer les dossiers nécessaires
[ICONS_DIR, ASSETS_DIR].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
});

async function createPngIcons() {
  console.log('🖼️  Génération des icônes PNG...');
  
  if (!fs.existsSync(SOURCE_IMAGE)) {
    throw new Error(`Image source introuvable: ${SOURCE_IMAGE}`);
  }

  const sourceBuffer = fs.readFileSync(SOURCE_IMAGE);
  
  for (const config of PNG_CONFIGS) {
    const outputPath = path.join(ICONS_DIR, config.name);
    
    await sharp(sourceBuffer)
      .resize(config.size, config.size, {
        fit: 'contain',
        background: { r: 255, g: 255, b: 255, alpha: 0 }
      })
      .png({
        compressionLevel: 9,
        adaptiveFiltering: true,
        palette: false,
        force: true
      })
      .ensureAlpha()
      .toFile(outputPath);
      
    console.log(`✅ PNG: ${config.name} (${config.size}x${config.size})`);
  }
}

async function createIcoIcons() {
  console.log('🔷 Génération des icônes ICO authentiques...');
  
  const sourceBuffer = fs.readFileSync(SOURCE_IMAGE);
  
  for (const config of ICO_CONFIGS) {
    const pngBuffers = [];
    
    // Générer tous les PNG nécessaires pour l'ICO
    for (const size of config.sizes) {
      const pngBuffer = await sharp(sourceBuffer)
        .resize(size, size, {
          fit: 'contain',
          background: { r: 255, g: 255, b: 255, alpha: 0 }
        })
        .png({
          compressionLevel: 9,
          palette: false,
          force: true
        })
        .ensureAlpha()
        .toBuffer();
        
      pngBuffers.push(pngBuffer);
    }
    
    // Créer le fichier ICO multi-résolution
    const icoBuffer = await pngToIco(pngBuffers);
    const outputPath = path.join(config.dir, config.name);
    
    fs.writeFileSync(outputPath, icoBuffer);
    console.log(`✅ ICO: ${config.name} (${config.sizes.join(', ')}px)`);
  }
}

async function validateIconStructure() {
  console.log('🔍 Validation de la structure des icônes...');
  
  // Vérifier les PNG
  for (const config of PNG_CONFIGS) {
    const filePath = path.join(ICONS_DIR, config.name);
    if (!fs.existsSync(filePath)) {
      throw new Error(`PNG manquant: ${config.name}`);
    }
    
    const stats = fs.statSync(filePath);
    if (stats.size === 0) {
      throw new Error(`PNG vide: ${config.name}`);
    }
  }
  
  // Vérifier les ICO
  for (const config of ICO_CONFIGS) {
    const filePath = path.join(config.dir, config.name);
    if (!fs.existsSync(filePath)) {
      throw new Error(`ICO manquant: ${config.name}`);
    }
    
    const stats = fs.statSync(filePath);
    if (stats.size === 0) {
      throw new Error(`ICO vide: ${config.name}`);
    }
    
    // Vérifier la signature ICO
    const buffer = fs.readFileSync(filePath);
    const icoSignature = Buffer.from([0x00, 0x00, 0x01, 0x00]);
    if (!buffer.subarray(0, 4).equals(icoSignature)) {
      throw new Error(`Signature ICO invalide: ${config.name}`);
    }
  }
  
  console.log('✅ Structure des icônes validée');
}

async function buildAllIcons() {
  try {
    console.log('🚀 Construction des icônes AirADCR Desktop\n');
    
    await createPngIcons();
    console.log('');
    
    await createIcoIcons();
    console.log('');
    
    await validateIconStructure();
    
    console.log('\n🎉 Toutes les icônes ont été générées avec succès!');
    console.log('📂 PNG: src-tauri/icons/');
    console.log('📂 ICO: src-tauri/icons/ et src-tauri/assets/');
    console.log('\n✅ Prêt pour npx tauri build');
    
  } catch (error) {
    console.error('❌ Erreur lors de la génération des icônes:', error.message);
    process.exit(1);
  }
}

buildAllIcons();