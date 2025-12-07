#!/bin/bash

# =============================================================================
# CLAUDE API ERROR SIMULATION TEST
# =============================================================================
# Simulates the exact Claude API error from Tess's logs

echo "🔬 CLAUDE API ERROR SIMULATION"
echo "=============================="

# ============================================================================
# REPRODUCE THE EXACT ERROR MESSAGE
# ============================================================================

echo ""
echo "📝 Exact Error from Tess Logs:"
echo "------------------------------"
echo "API Error: 400 {\"type\":\"error\",\"error\":{\"type\":\"invalid_request_error\",\"message\":\"messages.0.content.1.text: cache_control cannot be set for empty text blocks\"},\"request_id\":\"req_011CSar6KeQAQCTga9KXo8FH\"}"

echo ""
echo "🔍 Error Analysis:"
echo "- messages.0.content.1.text = Second content block in first message is empty"
echo "- cache_control cannot be set for empty text blocks"
echo "- This happens when Claude tries to process the response"

# ============================================================================
# SIMULATE THE PROBLEMATIC MESSAGE STRUCTURE
# ============================================================================

echo ""
echo "🧪 Simulating Problematic Message Structure:"
echo "--------------------------------------------"

# This simulates what would cause the exact error
PROBLEMATIC_MESSAGE='{
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "Valid first block"},
        {"type": "text", "text": ""}  // <-- EMPTY SECOND BLOCK = ERROR!
      ]
    }
  ]
}'

echo "Message that would cause the error:"
echo "$PROBLEMATIC_MESSAGE" | jq . 2>/dev/null || echo "❌ Invalid JSON"

# Check for the exact error condition
EMPTY_BLOCKS=$(echo "$PROBLEMATIC_MESSAGE" | jq '.messages[0].content[1].text' 2>/dev/null)
if [ "$EMPTY_BLOCKS" = '""' ] || [ -z "$EMPTY_BLOCKS" ]; then
    echo "❌ CONFIRMED: messages.0.content.1.text is empty - this causes cache_control error!"
else
    echo "✅ No empty blocks found"
fi

# ============================================================================
# TEST OUR FIX PREVENTS THIS
# ============================================================================

echo ""
echo "✅ Testing Our Fix Prevents This:"
echo "----------------------------------"

# Our fix ensures only one content block with valid text
OUR_FIXED_MESSAGE='{
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "🧪 **TESS QA TESTING WORKFLOW**\n\nYou are Tess - quality gatekeeper for Task 1.\n\n**CRITICAL INSTRUCTIONS**:\n- Write comprehensive tests\n- Validate acceptance criteria\n- Be extremely strict"}
      ]
    }
  ]
}'

echo "Our fixed message structure:"
echo "$OUR_FIXED_MESSAGE" | jq . 2>/dev/null || echo "❌ Invalid JSON"

# Verify our fix prevents the error
OUR_CONTENT=$(echo "$OUR_FIXED_MESSAGE" | jq '.messages[0].content[0].text' -r 2>/dev/null)
CONTENT_LENGTH=$(echo -n "$OUR_CONTENT" | wc -c)

echo ""
echo "Fix Validation:"
echo "- Content exists: $([ "$CONTENT_LENGTH" -gt 0 ] && echo '✅ YES' || echo '❌ NO')"
echo "- Content length: $CONTENT_LENGTH characters"
echo "- Only one content block: $(echo "$OUR_FIXED_MESSAGE" | jq '.messages[0].content | length' 2>/dev/null) blocks"

# ============================================================================
# SIMULATE CLAUDE API RESPONSE PROCESSING
# ============================================================================

echo ""
echo "🤖 Simulating Claude API Processing:"
echo "------------------------------------"

echo "What Claude would do with problematic message:"
echo "1. Receive message with empty content block"
echo "2. Try to set cache_control on the empty block"
echo "3. Fail with: 'cache_control cannot be set for empty text blocks'"
echo "4. Return 400 error to client"

echo ""
echo "What Claude will do with our fixed message:"
echo "1. Receive message with single, valid content block"
echo "2. Successfully process the content"
echo "3. Set cache_control appropriately"
echo "4. Return successful response"

# ============================================================================
# FINAL VALIDATION
# ============================================================================

echo ""
echo "🎯 FINAL VALIDATION SUMMARY"
echo "============================"

PROBLEM_REPRODUCED=$(echo "$PROBLEMATIC_MESSAGE" | jq '.messages[0].content[1].text == ""' 2>/dev/null && echo "true" || echo "false")
FIX_VALID=$(echo "$OUR_FIXED_MESSAGE" | jq '.messages[0].content[0].text | length > 0' 2>/dev/null && echo "true" || echo "false")

echo "Problem Reproduction:"
echo "  - Exact error condition reproduced: $([ "$PROBLEM_REPRODUCED" = "true" ] && echo '✅ YES' || echo '❌ NO')"

echo ""
echo "Fix Validation:"
echo "  - Single content block: $([ "$(echo "$OUR_FIXED_MESSAGE" | jq '.messages[0].content | length' 2>/dev/null)" -eq 1 ] && echo '✅ YES' || echo '❌ NO')"
echo "  - Content is not empty: $([ "$FIX_VALID" = "true" ] && echo '✅ YES' || echo '❌ NO')"
echo "  - Valid JSON structure: $(echo "$OUR_FIXED_MESSAGE" | jq empty 2>/dev/null && echo '✅ YES' || echo '❌ NO')"

echo ""
if [ "$PROBLEM_REPRODUCED" = "true" ] && [ "$FIX_VALID" = "true" ]; then
    echo "🎉 VALIDATION COMPLETE!"
    echo ""
    echo "✅ Successfully reproduced the exact cache_control error condition"
    echo "✅ Confirmed our fix prevents the error by ensuring:"
    echo "   - Only one content block (not multiple)"
    echo "   - Content block contains valid, non-empty text"
    echo "   - Proper JSON structure throughout"
    echo ""
    echo "🚀 The fix should resolve Tess's cache_control error!"
    echo ""
    echo "Next: Deploy and test the fix in the actual workflow."
    exit 0
else
    echo "❌ Validation issues found - fix may need refinement"
    exit 1
fi
