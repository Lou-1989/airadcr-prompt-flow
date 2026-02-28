
# Plan de correction macOS — TERMINÉ ✅

Toutes les corrections macOS ont été implémentées. Aucun impact sur Windows/Linux.

## Corrections appliquées

| # | Correction | Fichier | Statut |
|---|---|---|---|
| 1 | `allow-microphone allow-camera` dans sandbox iframe | `src/security/SecurityConfig.ts` | ✅ |
| 2 | `media-src 'self' blob: mediastream:` dans CSP | `src-tauri/tauri.conf.json` | ✅ |
| 3 | `ensure_accessibility()` garde avant Enigo | `src-tauri/src/main.rs` | ✅ |
| 4 | `Alt+Space` / `Alt+Shift+Space` (évite conflit Spotlight) | `src-tauri/src/main.rs` | ✅ |
| 5 | Entitlement `allow-unsigned-executable-memory` | `src-tauri/Entitlements.plist` | ✅ |
| 6 | Fallbacks 0 au lieu de 1920x1080 avec `warn!` | `src-tauri/src/main.rs` | ✅ |
| 7 | `display-capture` dans `allow` iframe | `src/security/SecurityConfig.ts` | ✅ |
