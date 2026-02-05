 # Intégration Client TÉO Hub - AIRADCR Desktop
 
 ## Vue d'ensemble
 
 Ce document décrit l'architecture du **mode client HTTP** ajouté à AIRADCR Desktop pour communiquer avec le serveur TÉO Hub.
 
 ### Architecture Dual-Mode
 
 AIRADCR Desktop supporte désormais **deux modes simultanés** :
 
 ```
 ┌─────────────────────────────────────────────────────────────┐
 │                    AIRADCR Desktop                          │
 ├─────────────────────────────────────────────────────────────┤
 │                                                             │
 │   MODE SERVEUR (existant)         MODE CLIENT (nouveau)     │
 │   localhost:8741                  -> TÉO Hub:54489          │
 │   ─────────────────               ────────────────────      │
 │   POST /pending-report            GET /th_get_ai_report     │
 │   GET  /pending-report            POST /th_post_approved    │
 │   POST /open-report                                         │
 │                                                             │
 │   Reçoit depuis: RIS/PACS         Envoie vers: TÉO Hub      │
 │   Stocke dans: SQLite local       Récupère depuis: MySQL    │
 │                                                             │
 └─────────────────────────────────────────────────────────────┘
 ```
 
 ---
 
 ## Configuration
 
 ### Fichier config.toml
 
 Emplacement : `%APPDATA%/airadcr-desktop/config.toml`
 
 ```toml
 # Configuration TÉO Hub Client
 [teo_hub]
 enabled = true
 host = "192.168.1.36"
 port = 54489
 health_endpoint = "th_health"
 get_report_endpoint = "th_get_ai_report"
 post_report_endpoint = "th_post_approved_report"
 timeout_secs = 30
 retry_count = 3
 retry_delay_ms = 1000
 
 # Authentification (choisir une méthode)
 api_key = "votre_clé_api"
 # OU
 bearer_token = "votre_token_jwt"
 
 # SSL/TLS (optionnel)
 tls_enabled = false
 ca_file = ""
 cert_file = ""
 key_file = ""
 ```
 
 ---
 
 ## Endpoints TÉO Hub
 
 ### 1. Health Check
 
 ```
 GET /th_health
 ```
 
 **Réponse attendue :**
 ```json
 {
   "status": "ok",
   "version": "1.0.0",
   "uptime_seconds": 3600
 }
 ```
 
 ### 2. Récupérer un rapport IA
 
 ```
 GET /th_get_ai_report?accession_number=ACC001
 ```
 
 **Headers :**
 - `X-API-Key: <api_key>` (si configuré)
 - `Authorization: Bearer <token>` (si configuré)
 
 **Réponse attendue :**
 ```json
 {
   "report_id": "uuid-123",
   "accession_number": "ACC001",
   "patient_id": "PAT12345",
   "study_instance_uid": "1.2.3.4.5",
   "modality": "MG",
   "ai_analysis": {
     "findings": [
       "Masse suspecte détectée quadrant supéro-externe sein droit",
       "Birads 4a recommandé"
     ],
     "measurements": {
       "mass_size_mm": 12.5,
       "density": "high"
     },
     "confidence_score": 0.87,
     "ai_modules": ["b-rayz"],
     "raw_text": "Texte brut généré par sr_to_text()..."
   },
   "template_id": "92ea14b8-b4d5-4c72-b407-91e92cae2a2b",
   "status": "pending",
   "created_at": "2025-02-05T10:30:00Z"
 }
 ```
 
 ### 3. Soumettre un rapport approuvé
 
 ```
 POST /th_post_approved_report
 Content-Type: application/json
 ```
 
 **Body :**
 ```json
 {
   "report_id": "uuid-123",
   "accession_number": "ACC001",
   "approved_text": "MAMMOGRAPHIE BILATERALE\n\nINDICATION: Dépistage...\n\nCONCLUSION: Birads 4a...",
   "radiologist_id": "RAD001",
   "radiologist_name": "Dr. Martin",
   "approval_timestamp": "2025-02-05T11:00:00Z",
   "modifications_made": true,
   "modified_sections": ["conclusion"]
 }
 ```
 
 **Réponse attendue :**
 ```json
 {
   "status": "accepted",
   "message": "Report approved successfully",
   "transaction_id": "txn-456",
   "received_at": "2025-02-05T11:00:01Z"
 }
 ```
 
 ---
 
 ## Workflow Complet
 
 ```
 ┌────────────────────────────────────────────────────────────────────┐
 │                    WORKFLOW COMPLET                                │
 ├────────────────────────────────────────────────────────────────────┤
 │                                                                    │
 │  1. PACS → TÉO Hub                                                 │
 │     [Images DICOM envoyées pour analyse IA]                        │
 │                                                                    │
 │  2. TÉO Hub → Prestataire IA (Gleamer B-rayz, etc.)                │
 │     [Analyse volumétrie, détection lésions, etc.]                  │
 │                                                                    │
 │  3. Prestataire IA → TÉO Hub                                       │
 │     [Résultats IA stockés dans MySQL th_ai_reports]                │
 │                                                                    │
 │  4. RIS/PACS → AIRADCR (appel contextuel)                          │
 │     POST localhost:8741/open-report?accession_number=XXX           │
 │                                                                    │
 │  5. AIRADCR → TÉO Hub [CLIENT HTTP]                                │
 │     GET http://192.168.1.36:54489/th_get_ai_report                 │
 │         ?accession_number=XXX                                      │
 │                                                                    │
 │  6. AIRADCR affiche le rapport pré-rempli                          │
 │     [Radiologue valide/modifie]                                    │
 │                                                                    │
 │  7. AIRADCR → TÉO Hub [CLIENT HTTP]                                │
 │     POST http://192.168.1.36:54489/th_post_approved_report         │
 │                                                                    │
 │  8. AIRADCR → RIS                                                  │
 │     [Injection clavier du rapport final]                           │
 │                                                                    │
 └────────────────────────────────────────────────────────────────────┘
 ```
 
 ---
 
 ## Commandes Tauri (Frontend)
 
 ### JavaScript/TypeScript
 
 ```typescript
 import { invoke } from '@tauri-apps/api/tauri';
 
 // 1. Vérifier la connexion
 const health = await invoke<TeoHealthResponse>('teo_check_health');
 console.log('TÉO Hub status:', health.status);
 
 // 2. Récupérer un rapport IA
 const report = await invoke<TeoAiReport>('teo_fetch_report', {
   accessionNumber: 'ACC001'
 });
 console.log('Rapport IA:', report.ai_analysis);
 
 // 3. Soumettre un rapport approuvé
 const result = await invoke<TeoApprovalResponse>('teo_submit_approved', {
   reportId: report.report_id,
   accessionNumber: report.accession_number,
   approvedText: 'Texte final du rapport...',
   radiologistId: 'RAD001',
   radiologistName: 'Dr. Martin',
   modificationsMade: true
 });
 console.log('Soumission:', result.status);
 
 // 4. Récupérer la configuration
 const config = await invoke<TeoHubConfigInfo>('teo_get_config');
 console.log('TÉO Hub enabled:', config.enabled);
 ```
 
 ---
 
 ## Sécurité
 
 ### Authentification
 
 Trois modes supportés (configurables dans config.toml) :
 
 | Mode | Header | Configuration |
 |------|--------|---------------|
 | API Key | `X-API-Key: <key>` | `api_key = "..."` |
 | Bearer Token | `Authorization: Bearer <token>` | `bearer_token = "..."` |
 | mTLS | Certificat client | `cert_file` + `key_file` |
 
 ### PII Masking
 
 Les identifiants patients sont masqués dans les logs :
 - `patient_id: PAT12345` → `patient_id: PAT1****`
 
 ### Retry Logic
 
 Backoff exponentiel en cas d'erreur réseau :
 - Retry 1 : après `retry_delay_ms` (1000ms)
 - Retry 2 : après `retry_delay_ms * 2` (2000ms)
 - Retry 3 : après `retry_delay_ms * 4` (4000ms)
 
 ---
 
 ## Interface Debug Panel
 
 L'onglet **TÉO Hub** dans le Debug Panel (Ctrl+Alt+D) affiche :
 
 - Statut de connexion (vert/rouge/gris)
 - Adresse du serveur
 - Version du serveur (si disponible)
 - Configuration active (TLS, API Key, retries)
 - Bouton de test de connexion
 
 ---
 
 ## Fichiers Modifiés/Créés
 
 ### Rust (Backend)
 
 | Fichier | Description |
 |---------|-------------|
 | `src-tauri/src/teo_client/mod.rs` | Module client HTTP principal |
 | `src-tauri/src/teo_client/models.rs` | Structures de données |
 | `src-tauri/src/teo_client/errors.rs` | Gestion des erreurs |
 | `src-tauri/src/config.rs` | Configuration étendue |
 | `src-tauri/src/main.rs` | Commandes Tauri ajoutées |
 | `src-tauri/Cargo.toml` | Dépendance reqwest |
 
 ### React (Frontend)
 
 | Fichier | Description |
 |---------|-------------|
 | `src/components/TeoHubConfig.tsx` | Panneau configuration UI |
 | `src/components/DebugPanel.tsx` | Onglet TÉO Hub ajouté |
 
 ---
 
 ## Points à Confirmer avec le CTO
 
 1. **Authentification** : API Key, Bearer, ou mTLS ?
 2. **Format exact** des réponses (champs optionnels, types)
 3. **Codes d'erreur** HTTP et messages
 4. **Timing** : Polling ou événement RIS ?