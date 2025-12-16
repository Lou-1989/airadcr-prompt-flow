#!/bin/bash
# ============================================================================
# AIRADCR Desktop - Script de test du serveur HTTP local
# ============================================================================
# Ce script teste tous les endpoints du serveur HTTP sur le port 8741
# ============================================================================

set -e

BASE_URL="http://localhost:8741"
API_KEY="airadcr_dev_key_2024"

echo "🧪 AIRADCR Desktop - Tests du serveur HTTP local"
echo "=================================================="
echo ""

# ============================================================================
# Test 1: Health Check
# ============================================================================
echo "📋 Test 1: GET /health"
echo "---"
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
HTTP_CODE=$(echo "$HEALTH_RESPONSE" | tail -1)
BODY=$(echo "$HEALTH_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Health check réussi (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Health check échoué (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
    exit 1
fi
echo ""

# ============================================================================
# Test 2: Store Pending Report (avec API Key)
# ============================================================================
echo "📋 Test 2: POST /pending-report (avec API Key valide)"
echo "---"
STORE_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/pending-report" \
    -H "Content-Type: application/json" \
    -H "X-API-Key: $API_KEY" \
    -d '{
        "technical_id": "TEST_SCRIPT_001",
        "structured": {
            "title": "Test IRM Cérébrale",
            "indication": "Script de test automatique",
            "technique": "Séquences T1, T2, FLAIR",
            "results": "Résultats du test IA automatique...",
            "conclusion": ""
        },
        "source_type": "test_script",
        "ai_modules": ["test_module"],
        "expires_in_hours": 1
    }')
HTTP_CODE=$(echo "$STORE_RESPONSE" | tail -1)
BODY=$(echo "$STORE_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Stockage réussi (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Stockage échoué (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 3: Store with Patient-Safe violation
# ============================================================================
echo "📋 Test 3: POST /pending-report (Patient-Safe violation - doit échouer)"
echo "---"
VIOLATION_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/pending-report" \
    -H "Content-Type: application/json" \
    -H "X-API-Key: $API_KEY" \
    -d '{
        "technical_id": "TEST_VIOLATION",
        "structured": {
            "title": "Test",
            "patient_id": "12345"
        }
    }')
HTTP_CODE=$(echo "$VIOLATION_RESPONSE" | tail -1)
BODY=$(echo "$VIOLATION_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "400" ]; then
    echo "✅ Patient-Safe violation détectée correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Patient-Safe violation non détectée (HTTP $HTTP_CODE - attendu 400)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 4: Store without API Key (should fail)
# ============================================================================
echo "📋 Test 4: POST /pending-report (sans API Key - doit échouer)"
echo "---"
NO_KEY_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/pending-report" \
    -H "Content-Type: application/json" \
    -d '{
        "technical_id": "TEST_NO_KEY",
        "structured": {"title": "Test"}
    }')
HTTP_CODE=$(echo "$NO_KEY_RESPONSE" | tail -1)
BODY=$(echo "$NO_KEY_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "401" ]; then
    echo "✅ Authentification requise correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Authentification non vérifiée (HTTP $HTTP_CODE - attendu 401)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 5: Get Pending Report
# ============================================================================
echo "📋 Test 5: GET /pending-report?tid=TEST_SCRIPT_001"
echo "---"
GET_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/pending-report?tid=TEST_SCRIPT_001")
HTTP_CODE=$(echo "$GET_RESPONSE" | tail -1)
BODY=$(echo "$GET_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Récupération réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Récupération échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 6: Get Non-existent Report
# ============================================================================
echo "📋 Test 6: GET /pending-report?tid=NON_EXISTENT (doit retourner 404)"
echo "---"
NOT_FOUND_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/pending-report?tid=NON_EXISTENT")
HTTP_CODE=$(echo "$NOT_FOUND_RESPONSE" | tail -1)
BODY=$(echo "$NOT_FOUND_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "404" ]; then
    echo "✅ 404 retourné correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Code incorrect (HTTP $HTTP_CODE - attendu 404)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 7: Delete Pending Report
# ============================================================================
echo "📋 Test 7: DELETE /pending-report?tid=TEST_SCRIPT_001"
echo "---"
DELETE_RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/pending-report?tid=TEST_SCRIPT_001")
HTTP_CODE=$(echo "$DELETE_RESPONSE" | tail -1)
BODY=$(echo "$DELETE_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Suppression réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Suppression échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 8: Verify deletion
# ============================================================================
echo "📋 Test 8: GET /pending-report?tid=TEST_SCRIPT_001 (après suppression)"
echo "---"
VERIFY_DELETE=$(curl -s -w "\n%{http_code}" "$BASE_URL/pending-report?tid=TEST_SCRIPT_001")
HTTP_CODE=$(echo "$VERIFY_DELETE" | tail -1)

if [ "$HTTP_CODE" == "404" ]; then
    echo "✅ Rapport bien supprimé (HTTP $HTTP_CODE)"
else
    echo "❌ Rapport encore présent après suppression (HTTP $HTTP_CODE)"
fi
echo ""

# ============================================================================
# Test 9: Store report with RIS identifiers for /find-report tests
# ============================================================================
echo "📋 Test 9: POST /pending-report (avec identifiants RIS)"
echo "---"
STORE_RIS_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/pending-report" \
    -H "Content-Type: application/json" \
    -H "X-API-Key: $API_KEY" \
    -d '{
        "technical_id": "TEO_ACC2024_TEST",
        "patient_id": "PAT_TEST_001",
        "accession_number": "ACC2024TEST",
        "exam_uid": "1.2.3.4.5.6.7.8.9",
        "structured": {
            "title": "IRM Cérébrale - Test RIS",
            "indication": "Test endpoints RIS",
            "technique": "T1, T2, FLAIR",
            "results": "Résultats du test...",
            "conclusion": ""
        },
        "source_type": "teo_hub",
        "modality": "MR",
        "expires_in_hours": 1
    }')
HTTP_CODE=$(echo "$STORE_RIS_RESPONSE" | tail -1)
BODY=$(echo "$STORE_RIS_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Stockage avec identifiants RIS réussi (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Stockage échoué (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 10: Find report by accession_number
# ============================================================================
echo "📋 Test 10: GET /find-report?accession_number=ACC2024TEST"
echo "---"
FIND_ACC_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report?accession_number=ACC2024TEST")
HTTP_CODE=$(echo "$FIND_ACC_RESPONSE" | tail -1)
BODY=$(echo "$FIND_ACC_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Recherche par accession_number réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Recherche échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 11: Find report by patient_id
# ============================================================================
echo "📋 Test 11: GET /find-report?patient_id=PAT_TEST_001"
echo "---"
FIND_PAT_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report?patient_id=PAT_TEST_001")
HTTP_CODE=$(echo "$FIND_PAT_RESPONSE" | tail -1)
BODY=$(echo "$FIND_PAT_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Recherche par patient_id réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Recherche échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 12: Find report by exam_uid
# ============================================================================
echo "📋 Test 12: GET /find-report?exam_uid=1.2.3.4.5.6.7.8.9"
echo "---"
FIND_UID_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report?exam_uid=1.2.3.4.5.6.7.8.9")
HTTP_CODE=$(echo "$FIND_UID_RESPONSE" | tail -1)
BODY=$(echo "$FIND_UID_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Recherche par exam_uid réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Recherche échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 13: Find report with multiple identifiers
# ============================================================================
echo "📋 Test 13: GET /find-report?patient_id=PAT_TEST_001&accession_number=ACC2024TEST"
echo "---"
FIND_MULTI_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report?patient_id=PAT_TEST_001&accession_number=ACC2024TEST")
HTTP_CODE=$(echo "$FIND_MULTI_RESPONSE" | tail -1)
BODY=$(echo "$FIND_MULTI_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Recherche multi-critères réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Recherche échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 14: Find non-existent report
# ============================================================================
echo "📋 Test 14: GET /find-report?accession_number=NON_EXISTENT (doit retourner 404)"
echo "---"
FIND_404_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report?accession_number=NON_EXISTENT")
HTTP_CODE=$(echo "$FIND_404_RESPONSE" | tail -1)
BODY=$(echo "$FIND_404_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "404" ]; then
    echo "✅ 404 retourné correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Code incorrect (HTTP $HTTP_CODE - attendu 404)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 15: Find without identifiers (should fail)
# ============================================================================
echo "📋 Test 15: GET /find-report (sans identifiant - doit retourner 400)"
echo "---"
FIND_NO_ID_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/find-report")
HTTP_CODE=$(echo "$FIND_NO_ID_RESPONSE" | tail -1)
BODY=$(echo "$FIND_NO_ID_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "400" ]; then
    echo "✅ 400 retourné correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Code incorrect (HTTP $HTTP_CODE - attendu 400)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 16: Open report by technical_id
# ============================================================================
echo "📋 Test 16: POST /open-report?tid=TEO_ACC2024_TEST"
echo "---"
OPEN_TID_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/open-report?tid=TEO_ACC2024_TEST")
HTTP_CODE=$(echo "$OPEN_TID_RESPONSE" | tail -1)
BODY=$(echo "$OPEN_TID_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Navigation déclenchée par tid (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Navigation échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 17: Open report by accession_number
# ============================================================================
echo "📋 Test 17: POST /open-report?accession_number=ACC2024TEST"
echo "---"
OPEN_ACC_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/open-report?accession_number=ACC2024TEST")
HTTP_CODE=$(echo "$OPEN_ACC_RESPONSE" | tail -1)
BODY=$(echo "$OPEN_ACC_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Navigation déclenchée par accession_number (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Navigation échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 18: Open report by patient_id + accession_number
# ============================================================================
echo "📋 Test 18: POST /open-report?patient_id=PAT_TEST_001&accession_number=ACC2024TEST"
echo "---"
OPEN_MULTI_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/open-report?patient_id=PAT_TEST_001&accession_number=ACC2024TEST")
HTTP_CODE=$(echo "$OPEN_MULTI_RESPONSE" | tail -1)
BODY=$(echo "$OPEN_MULTI_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Navigation multi-critères réussie (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Navigation échouée (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 19: Open non-existent report
# ============================================================================
echo "📋 Test 19: POST /open-report?tid=NON_EXISTENT (doit retourner 404)"
echo "---"
OPEN_404_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/open-report?tid=NON_EXISTENT")
HTTP_CODE=$(echo "$OPEN_404_RESPONSE" | tail -1)
BODY=$(echo "$OPEN_404_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "404" ]; then
    echo "✅ 404 retourné correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Code incorrect (HTTP $HTTP_CODE - attendu 404)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 20: Open without identifiers (should fail)
# ============================================================================
echo "📋 Test 20: POST /open-report (sans identifiant - doit retourner 400)"
echo "---"
OPEN_NO_ID_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/open-report")
HTTP_CODE=$(echo "$OPEN_NO_ID_RESPONSE" | tail -1)
BODY=$(echo "$OPEN_NO_ID_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "400" ]; then
    echo "✅ 400 retourné correctement (HTTP $HTTP_CODE)"
    echo "   Response: $BODY"
else
    echo "❌ Code incorrect (HTTP $HTTP_CODE - attendu 400)"
    echo "   Response: $BODY"
fi
echo ""

# ============================================================================
# Test 21: Cleanup - Delete test report
# ============================================================================
echo "📋 Test 21: DELETE /pending-report?tid=TEO_ACC2024_TEST (nettoyage)"
echo "---"
CLEANUP_RESPONSE=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/pending-report?tid=TEO_ACC2024_TEST")
HTTP_CODE=$(echo "$CLEANUP_RESPONSE" | tail -1)
BODY=$(echo "$CLEANUP_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Nettoyage réussi (HTTP $HTTP_CODE)"
else
    echo "⚠️  Nettoyage: HTTP $HTTP_CODE"
fi
echo ""

# ============================================================================
# Résumé
# ============================================================================
echo "=================================================="
echo "🏁 Tests terminés (21 tests)"
echo "=================================================="
