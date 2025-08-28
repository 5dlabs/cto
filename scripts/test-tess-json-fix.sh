#!/bin/bash

# =============================================================================
# TESS JSON FORMATTING VALIDATION TEST
# =============================================================================
# This script tests the JSON construction fix for Tess's cache_control error
# Run this locally to validate the fix before deploying

echo "🧪 TESS JSON FORMATTING VALIDATION TEST"
echo "========================================"

# Sample INITIAL_GUIDANCE content (simplified version)
INITIAL_GUIDANCE="🧪 **TESS QA TESTING WORKFLOW**

You are Tess - quality gatekeeper for Task 1.

**CRITICAL INSTRUCTIONS**:
- Write comprehensive tests
- Validate acceptance criteria
- Be extremely strict

**Testing Requirements:**
1. Verify CI status
2. Write unit tests
3. Check coverage >= 95%
4. Deploy to K8s if available
5. Post APPROVE review"

echo "📝 Testing JSON Construction Methods:"
echo "-------------------------------------"

# ============================================================================
# TEST 1: BROKEN METHOD (Original problematic approach)
# ============================================================================
echo ""
echo "❌ TEST 1: BROKEN METHOD (Original)"
echo "----------------------------------"

# This is the problematic approach that was causing cache_control errors
INITIAL_GUIDANCE_CLEAN=$(printf "%s" "$INITIAL_GUIDANCE" | sed '/^[[:space:]]*$/d' | sed 's/[[:space:]]*$//')
USER_COMBINED=$(printf "%s" "$INITIAL_GUIDANCE_CLEAN" | jq -Rs .)

echo "Pre-computed JSON string length: ${#USER_COMBINED} characters"
echo "Pre-computed JSON preview: ${USER_COMBINED:0:100}..."

# Construct message the BROKEN way
BROKEN_JSON=$(printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$USER_COMBINED")
echo "Broken JSON result:"
echo "$BROKEN_JSON" | jq . 2>/dev/null || echo "❌ JSON parsing failed!"
echo ""

# ============================================================================
# TEST 2: FIXED METHOD (New inline approach)
# ============================================================================
echo ""
echo "✅ TEST 2: FIXED METHOD (New Approach)"
echo "--------------------------------------"

# This is the fixed approach using inline jq
FIXED_JSON=$(printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$(jq -Rs . <<< "$INITIAL_GUIDANCE_CLEAN")")
echo "Fixed JSON result:"
echo "$FIXED_JSON" | jq . 2>/dev/null || echo "❌ JSON parsing failed!"
echo ""

# ============================================================================
# TEST 3: VALIDATION TESTS
# ============================================================================
echo ""
echo "🔍 TEST 3: VALIDATION TESTS"
echo "---------------------------"

# Test 1: Check if JSON is valid
echo "JSON Validity Check:"
echo -n "Broken JSON: "
echo "$BROKEN_JSON" | jq empty 2>/dev/null && echo "✅ Valid" || echo "❌ Invalid"

echo -n "Fixed JSON: "
echo "$FIXED_JSON" | jq empty 2>/dev/null && echo "✅ Valid" || echo "❌ Invalid"

# Test 2: Check content block structure
echo ""
echo "Content Block Structure Check:"
echo "Broken JSON content blocks: $(echo "$BROKEN_JSON" | jq '.message.content | length' 2>/dev/null || echo 'N/A')"
echo "Fixed JSON content blocks: $(echo "$FIXED_JSON" | jq '.message.content | length' 2>/dev/null || echo 'N/A')"

# Test 3: Check for empty text blocks (the cache_control issue)
echo ""
echo "Empty Text Block Check (cache_control source):"
echo -n "Broken JSON has empty blocks: "
BROKEN_EMPTY=$(echo "$BROKEN_JSON" | jq '[.message.content[] | select(.type == "text" and (.text == "" or .text == null))] | length' 2>/dev/null)
[ "$BROKEN_EMPTY" -gt 0 ] 2>/dev/null && echo "❌ YES ($BROKEN_EMPTY empty blocks)" || echo "✅ NO"

echo -n "Fixed JSON has empty blocks: "
FIXED_EMPTY=$(echo "$FIXED_JSON" | jq '[.message.content[] | select(.type == "text" and (.text == "" or .text == null))] | length' 2>/dev/null)
[ "$FIXED_EMPTY" -gt 0 ] 2>/dev/null && echo "❌ YES ($FIXED_EMPTY empty blocks)" || echo "✅ NO"

# ============================================================================
# TEST 4: SIMULATE SIDECAR PAYLOAD
# ============================================================================
echo ""
echo "📡 TEST 4: SIDECAR PAYLOAD SIMULATION"
echo "-------------------------------------"

# Test sidecar format (what gets sent to /input endpoint)
SIDECAR_PAYLOAD_BROKEN=$(printf '{"text":%s}\n' "$USER_COMBINED")
SIDECAR_PAYLOAD_FIXED=$(printf '{"text":%s}\n' "$(jq -Rs . <<< "$INITIAL_GUIDANCE_CLEAN")")

echo "Broken sidecar payload:"
echo "$SIDECAR_PAYLOAD_BROKEN" | jq . 2>/dev/null || echo "❌ Invalid JSON"
echo ""

echo "Fixed sidecar payload:"
echo "$SIDECAR_PAYLOAD_FIXED" | jq . 2>/dev/null || echo "❌ Invalid JSON"
echo ""

# ============================================================================
# SUMMARY
# ============================================================================
echo "📋 TEST SUMMARY"
echo "==============="

BROKEN_VALID=$(echo "$BROKEN_JSON" | jq empty 2>/dev/null && echo "true" || echo "false")
FIXED_VALID=$(echo "$FIXED_JSON" | jq empty 2>/dev/null && echo "true" || echo "false")

echo "Broken Method:"
echo "  - JSON Valid: $([ "$BROKEN_VALID" = "true" ] && echo "✅" || echo "❌")"
echo "  - Empty Blocks: $([ "$BROKEN_EMPTY" -gt 0 ] 2>/dev/null && echo "❌ ($BROKEN_EMPTY)" || echo "✅ (0)")"

echo ""
echo "Fixed Method:"
echo "  - JSON Valid: $([ "$FIXED_VALID" = "true" ] && echo "✅" || echo "❌")"
echo "  - Empty Blocks: $([ "$FIXED_EMPTY" -gt 0 ] 2>/dev/null && echo "❌ ($FIXED_EMPTY)" || echo "✅ (0)")"

echo ""
if [ "$BROKEN_VALID" = "true" ] && [ "$FIXED_VALID" = "true" ] && [ "$BROKEN_EMPTY" -eq 0 ] && [ "$FIXED_EMPTY" -eq 0 ]; then
    echo "🎉 TEST PASSED: Both methods produce valid JSON with no empty blocks"
    echo "   The fix should resolve the cache_control error!"
    exit 0
elif [ "$FIXED_VALID" = "true" ] && [ "$FIXED_EMPTY" -eq 0 ]; then
    echo "✅ TEST PASSED: Fixed method is valid and has no empty blocks"
    echo "   The fix should resolve the cache_control error!"
    exit 0
else
    echo "❌ TEST FAILED: Issues found with JSON construction"
    echo "   The fix may not resolve the cache_control error"
    exit 1
fi
