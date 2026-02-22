
# Audit de Sécurité Global AIRADCR Desktop — Standards 2026

## Statut d'implémentation

### ✅ Phase 1 — Corrections immédiates (TERMINÉE)

1. **Faille #1** ✅ `clipboard_modifier()` dans `perform_injection_at_position_direct` (main.rs)
2. **Faille #4** ✅ Clé API masquée dans les logs (schema.rs) — affiche seulement les 16 premiers chars
3. **Faille #5** ✅ Limite Content-Length 1 MB ajoutée (mod.rs)
4. **Faille #6** ✅ Validation whitelist `["F10","F11","F12"]` AVANT insertion JS (main.rs)
5. **Faille #13** ✅ Dépendance `md5` supprimée de Cargo.toml

### ✅ Phase 2 — Renforcement authentification (TERMINÉE)

6. **Faille #3** ✅ `/health/extended` et `/metrics` protégés par clé admin (X-Admin-Key)
7. **Faille #9** ✅ `DELETE /pending-report` exige X-API-Key
8. **Faille #12** ✅ `POST /open-report` exige X-API-Key
9. **Faille #14** ✅ Validation TID via `validate_technical_id()` dans open-report handler
10. **Faille #3 bis** ✅ `GET /health` ne retourne plus la version de l'application

### ✅ Phase 3 — Améliorations structurelles (TERMINÉE)

11. **Faille #7** ✅ Chemins de logs cross-platform via `dirs::data_local_dir()` + `PathBuf::join()`
12. **Faille #10** ✅ Header HSTS ajouté dans index.html
13. **Faille #11** ✅ Migration `println!`/`eprintln!` → `log::info!`/`log::warn!`/`log::error!` (handlers.rs, middleware.rs)
14. **macOS** ✅ `open_log_folder()` supporte macOS via `open` command

### ✅ Phase 4 — Chiffrement (TERMINÉE)

15. **Faille #2** ✅ Migration SQLite → SQLCipher (AES-256 au repos), clé stockée dans keychain OS
16. **Faille #16** ✅ Token TEO Hub migré automatiquement du config.toml vers le keychain OS
17. **Faille #8** — Rate limiting renforcé pour échecs 401 (tracking par IP)
18. **Faille #15** — CSP nonces en production (complexe, nécessite build pipeline)

## Points forts confirmés

- SHA-256 pour toutes les clés API
- Comparaison en temps constant (`constant_time_compare`)
- Masquage PII dans les logs (patient_id: 1234****)
- Deep link validation (max 64 chars, regex alphanumérique)
- CORS restrictif (uniquement localhost et airadcr.com)
- Rate limiting global via actix-governor
- Serveur HTTP sur 127.0.0.1 uniquement
- Backup automatique SQLite quotidien
- Clé admin externalisée obligatoire en production
- CSP + X-Frame-Options + HSTS
- Sandbox iframe avec permissions minimales
- Validation postMessage par origin + type
- Single-instance protection
