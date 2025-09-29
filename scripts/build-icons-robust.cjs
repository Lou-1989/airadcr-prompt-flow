#!/usr/bin/env node

const sharp = require('sharp');
const toIco = require('to-ico');
const fs = require('fs');
const path = require('path');

console.log('🚀 CONSTRUCTION ROBUSTE DES ICÔNES AIRADCR\n');

const SOURCE_IMAGE = path.join(__dirname, '../src/assets/airadcr-logo.png');
const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');
const ASSETS_DIR = path.join(__dirname, '../src-tauri/assets');

// Créer les dossiers
[ICONS_DIR, ASSETS_DIR].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
    console.log(`📁 Dossier créé: ${path.relative(process.cwd(), dir)}`);
  }
});

// Configuration des PNG requis par Tauri
const PNG_SIZES = [
  { size: 32, name: '32x32.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' },
  { size: 512, name: 'icon.png' }
];

// Tailles pour ICO multi-résolution Windows
const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256];

async function validateSourceImage() {
  console.log('🔍 Validation de l\'image source...');
  
  if (!fs.existsSync(SOURCE_IMAGE)) {
    throw new Error(`Image source introuvable: ${SOURCE_IMAGE}`);
  }
  
  const buffer = fs.readFileSync(SOURCE_IMAGE);
  if (buffer.length === 0) {
    throw new Error('Image source vide');
  }
  
  // Vérifier la signature PNG
  const pngSignature = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
  if (!buffer.subarray(0, 8).equals(pngSignature)) {
    throw new Error('Image source n\'est pas un PNG valide');
  }
  
  console.log(`✅ Image source validée: ${buffer.length} bytes`);
  return buffer;
}

async function generatePngIcons(sourceBuffer) {
  console.log('🖼️  Génération des PNG pour toutes plateformes...');
  
  const results = [];
  
  for (const config of PNG_SIZES) {
    const outputPath = path.join(ICONS_DIR, config.name);
    
    try {
      const pngBuffer = await sharp(sourceBuffer)
        .resize(config.size, config.size, {
          fit: 'contain',
          background: { r: 0, g: 0, b: 0, alpha: 0 } // Transparent
        })
        .png({
          compressionLevel: 9,
          adaptiveFiltering: true,
          palette: false,
          force: true
        })
        .ensureAlpha() // Force RGBA
        .toBuffer();
      
      // Écrire le fichier
      fs.writeFileSync(outputPath, pngBuffer);
      
      // Valider le fichier écrit
      const writtenBuffer = fs.readFileSync(outputPath);
      if (writtenBuffer.length === 0) {
        throw new Error(`Fichier PNG vide après écriture: ${config.name}`);
      }
      
      console.log(`  ✅ ${config.name} (${config.size}x${config.size}) - ${writtenBuffer.length} bytes`);
      results.push({ buffer: pngBuffer, size: config.size, name: config.name });
      
    } catch (error) {
      throw new Error(`Erreur génération ${config.name}: ${error.message}`);
    }
  }
  
  return results;
}

async function generateWindowsIcoIcons(sourceBuffer) {
  console.log('🔷 Génération des ICO authentiques pour Windows...');
  
  try {
    // Générer tous les PNG pour l'ICO
    const icoPngBuffers = [];
    
    for (const size of ICO_SIZES) {
      const pngBuffer = await sharp(sourceBuffer)
        .resize(size, size, {
          fit: 'contain',
          background: { r: 0, g: 0, b: 0, alpha: 0 }
        })
        .png({
          compressionLevel: 9,
          palette: false,
          force: true
        })
        .ensureAlpha()
        .toBuffer();
        
      icoPngBuffers.push(pngBuffer);
    }
    
    // Créer les ICO multi-résolution
    const iconIcoBuffer = await toIco(icoPngBuffers);
    const installerIcoBuffer = await toIco(icoPngBuffers);
    
    // Écrire les fichiers ICO
    const iconIcoPath = path.join(ICONS_DIR, 'icon.ico');
    const installerIcoPath = path.join(ASSETS_DIR, 'installer.ico');
    
    fs.writeFileSync(iconIcoPath, iconIcoBuffer);
    fs.writeFileSync(installerIcoPath, installerIcoBuffer);
    
    // Valider les signatures ICO
    const icoSignature = Buffer.from([0x00, 0x00, 0x01, 0x00]);
    
    [iconIcoPath, installerIcoPath].forEach((icoPath) => {
      const icoBuffer = fs.readFileSync(icoPath);
      if (!icoBuffer.subarray(0, 4).equals(icoSignature)) {
        throw new Error(`Signature ICO invalide: ${path.basename(icoPath)}`);
      }
      console.log(`  ✅ ${path.basename(icoPath)} - ${icoBuffer.length} bytes (ICO authentique)`);
    });
    
  } catch (error) {
    throw new Error(`Erreur génération ICO: ${error.message}`);
  }
}

async function validateAllGeneratedFiles() {
  console.log('🧪 Validation finale des fichiers...');
  
  const requiredFiles = [
    ...PNG_SIZES.map(s => ({ path: path.join(ICONS_DIR, s.name), type: 'PNG' })),
    { path: path.join(ICONS_DIR, 'icon.ico'), type: 'ICO' },
    { path: path.join(ASSETS_DIR, 'installer.ico'), type: 'ICO' }
  ];
  
  let validCount = 0;
  
  for (const file of requiredFiles) {
    if (!fs.existsSync(file.path)) {
      throw new Error(`Fichier manquant: ${path.basename(file.path)}`);
    }
    
    const buffer = fs.readFileSync(file.path);
    if (buffer.length === 0) {
      throw new Error(`Fichier vide: ${path.basename(file.path)}`);
    }
    
    // Vérifier les signatures
    if (file.type === 'PNG') {
      const pngSig = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
      if (!buffer.subarray(0, 8).equals(pngSig)) {
        throw new Error(`Signature PNG invalide: ${path.basename(file.path)}`);
      }
    } else if (file.type === 'ICO') {
      const icoSig = Buffer.from([0x00, 0x00, 0x01, 0x00]);
      if (!buffer.subarray(0, 4).equals(icoSig)) {
        throw new Error(`Signature ICO invalide: ${path.basename(file.path)}`);
      }
    }
    
    validCount++;
  }
  
  console.log(`✅ ${validCount}/${requiredFiles.length} fichiers validés`);
}

async function buildAllIcons() {
  try {
    // 1. Valider l'image source
    const sourceBuffer = await validateSourceImage();
    console.log('');
    
    // 2. Générer les PNG pour toutes les plateformes
    await generatePngIcons(sourceBuffer);
    console.log('');
    
    // 3. Générer les ICO authentiques pour Windows
    await generateWindowsIcoIcons(sourceBuffer);
    console.log('');
    
    // 4. Validation finale
    await validateAllGeneratedFiles();
    
    console.log('\n🎉 SUCCÈS COMPLET !');
    console.log('📂 PNG Tauri: src-tauri/icons/');
    console.log('📂 ICO Windows: src-tauri/icons/icon.ico + src-tauri/assets/installer.ico');
    console.log('🍎 macOS: Tauri convertira automatiquement les PNG en ICNS');
    console.log('🐧 Linux: Utilisera directement les PNG générés');
    console.log('\n✅ Prêt pour npx tauri build sur toutes les plateformes !');
    
  } catch (error) {
    console.error('\n❌ ÉCHEC DE LA GÉNÉRATION DES ICÔNES');
    console.error(`Erreur: ${error.message}`);
    console.error('\nActions recommandées:');
    console.error('1. Vérifier que src/assets/airadcr-logo.png existe et est valide');
    console.error('2. Vérifier les permissions de fichiers');
    console.error('3. Relancer le script après correction');
    process.exit(1);
  }
}

// Exécution
buildAllIcons();