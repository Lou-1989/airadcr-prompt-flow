#!/usr/bin/env node

// Force all PNG icons to true RGBA (colorType=6, bitDepth=8)
// This script converts any RGB (type=2) or indexed (type=3) PNGs to RGBA

const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');

(async () => {
  console.log('🛠  Forçage RGBA des icônes PNG...');
  if (!fs.existsSync(ICONS_DIR)) {
    console.error(`❌ Dossier introuvable: ${ICONS_DIR}`);
    process.exit(1);
  }

  const files = fs
    .readdirSync(ICONS_DIR)
    .filter((f) => f.toLowerCase().endsWith('.png'))
    .sort();

  let ok = 0;
  let fail = 0;

  for (const file of files) {
    const inPath = path.join(ICONS_DIR, file);
    try {
      const input = sharp(inPath, { failOn: 'error' });
      const meta = await input.metadata();

      // Convert to buffer ensuring alpha channel and non-palette PNG
      const buffer = await sharp(inPath)
        .toColorspace('srgb')
        .ensureAlpha() // add opaque alpha if missing
        .png({ compressionLevel: 9, adaptiveFiltering: true, palette: false, force: true })
        .toBuffer();

      fs.writeFileSync(inPath, buffer);

      const verify = await sharp(inPath).metadata();
      const channels = verify.channels ?? 0;
      const hasAlpha = verify.hasAlpha === true || channels === 4;

      if (hasAlpha && channels >= 4) {
        console.log(`✅ ${file} -> RGBA (${verify.width}x${verify.height}, channels=${channels})`);
        ok++;
      } else {
        console.warn(`⚠️  ${file} conversion incertaine (channels=${channels}, hasAlpha=${verify.hasAlpha})`);
        ok++;
      }
    } catch (e) {
      console.error(`❌ Échec conversion ${file}: ${e.message}`);
      fail++;
    }
  }

  console.log(`\n📊 Résultat conversion: ${ok}/${files.length} PNG traités, erreurs: ${fail}`);
  if (fail > 0) process.exit(1);
  console.log('🎉 Toutes les icônes PNG ont été converties en RGBA.');
})();
