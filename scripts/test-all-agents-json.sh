#!/bin/bash

# =============================================================================
# COMPREHENSIVE JSON CONSTRUCTION TEST FOR ALL AGENTS
# =============================================================================
# Test Rex, Cleo, and Tess JSON construction patterns

echo "🔬 COMPREHENSIVE AGENT JSON CONSTRUCTION TEST"
echo "=============================================="

# ============================================================================
# TEST 1: Cleo Pattern
# ============================================================================

echo ""
echo "🧪 TEST 1: Cleo JSON Construction Pattern"
echo "----------------------------------------"

CLEO_PROMPT="# Code Quality Review Assignment

You are Cleo, a rigorous code quality enforcement agent. Your mission is to ensure zero-tolerance quality standards for this pull request.

## Your Role
- **Primary Focus**: Code quality enforcement AND CI/CD pipeline setup
- **Quality Tools**: Clippy (pedantic), cargo fmt, cargo test, YAML linting
- **DevOps Setup**: GitHub Actions workflows, Docker image building, CI verification
- **Decision Authority**: Add 'ready-for-qa' label only when ALL quality checks AND CI builds pass
- **Standards**: Zero warnings, perfect formatting, 100% test pass rate, working Docker builds

## Current Context

### Pull Request Information
- **PR Number**: 123
- **PR URL**: https://github.com/test/repo/pull/123
- **Repository**: test/repo
- **Working Directory**: /workspace/repo"

echo "Testing Cleo's problematic pattern:"
echo "USER_COMBINED=\$(printf \"%s\" \"\$CLEO_PROMPT\" | jq -Rs .)"

# Cleo's problematic pattern
CLEO_USER_COMBINED=$(printf "%s" "$CLEO_PROMPT" | jq -Rs .)
CLEO_RESULT=$(printf '{"text":%s}\n' "$CLEO_USER_COMBINED")

echo "Cleo result:"
echo "$CLEO_RESULT" | jq . 2>/dev/null || echo "❌ JSON parsing failed"

# ============================================================================
# TEST 2: Rex Pattern
# ============================================================================

echo ""
echo "🧪 TEST 2: Rex JSON Construction Pattern"
echo "---------------------------------------"

PROMPT_PREFIX="⛔ **CRITICAL TASK ISOLATION REQUIREMENT** ⛔

You are Rex, an elite Rust development specialist. Your expertise spans the entire Rust ecosystem and development lifecycle.

## Your Mission
- **Primary Goal**: Deliver production-ready Rust code with zero compromises
- **Quality Standard**: Enterprise-grade, battle-tested code
- **Focus Areas**: Performance, safety, maintainability, correctness"

PROMPT_CONTENT="Implement a high-performance Rust web service with async handling."

echo "Testing Rex's problematic pattern:"
echo "USER_COMBINED=\$(printf \"%s\" \"\${PROMPT_PREFIX}\$(cat prompt.md)\" | jq -Rs .)"

# Rex's problematic pattern
REX_CONTENT="${PROMPT_PREFIX}${PROMPT_CONTENT}"
REX_USER_COMBINED=$(printf "%s" "$REX_CONTENT" | jq -Rs .)
REX_RESULT=$(printf '{"text":%s}\n' "$REX_USER_COMBINED")

echo "Rex result:"
echo "$REX_RESULT" | jq . 2>/dev/null || echo "❌ JSON parsing failed"

# ============================================================================
# TEST 3: Tess Pattern (Original Problematic)
# ============================================================================

echo ""
echo "🧪 TEST 3: Tess Original Problematic Pattern"
echo "--------------------------------------------"

INITIAL_GUIDANCE="🧪 **TESS ULTRA-STRICT QA TESTING WORKFLOW**

You are Tess - the quality gatekeeper for Task 1.

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

echo "Testing Tess's original problematic pattern:"
echo "USER_COMBINED=\$(printf \"%s\" \"\$INITIAL_GUIDANCE_CLEAN\" | jq -Rs .)"

# Tess's original problematic pattern
TESS_USER_COMBINED=$(printf "%s" "$INITIAL_GUIDANCE" | jq -Rs .)
TESS_RESULT=$(printf '{"text":%s}\n' "$TESS_USER_COMBINED")

echo "Tess original result:"
echo "$TESS_RESULT" | jq . 2>/dev/null || echo "❌ JSON parsing failed"

# ============================================================================
# TEST 4: Our Fixed Pattern
# ============================================================================

echo ""
echo "✅ TEST 4: Our Fixed Inline Pattern"
echo "------------------------------------"

echo "Testing our fixed inline pattern:"
echo "printf '{\"text\":%s}\\n' \"\$(jq -Rs . <<< \"\$CONTENT\")\""

# Our fixed pattern
FIXED_RESULT=$(printf '{"text":%s}\n' "$(jq -Rs . <<< "$INITIAL_GUIDANCE")")

echo "Fixed result:"
echo "$FIXED_RESULT" | jq . 2>/dev/null || echo "❌ JSON parsing failed"

# ============================================================================
# ANALYSIS
# ============================================================================

echo ""
echo "📊 ANALYSIS SUMMARY"
echo "==================="

# Check for JSON validity
CLEO_VALID=$(echo "$CLEO_RESULT" | jq empty 2>/dev/null && echo "✅" || echo "❌")
REX_VALID=$(echo "$REX_RESULT" | jq empty 2>/dev/null && echo "✅" || echo "❌")
TESS_ORIGINAL_VALID=$(echo "$TESS_RESULT" | jq empty 2>/dev/null && echo "✅" || echo "❌")
TESS_FIXED_VALID=$(echo "$FIXED_RESULT" | jq empty 2>/dev/null && echo "✅" || echo "❌")

echo "JSON Validity:"
echo "  Cleo: $CLEO_VALID"
echo "  Rex: $REX_VALID"
echo "  Tess (Original): $TESS_ORIGINAL_VALID"
echo "  Tess (Fixed): $TESS_FIXED_VALID"

# Check content lengths
CLEO_LENGTH=$(echo "$CLEO_RESULT" | jq -r '.text' 2>/dev/null | wc -c)
REX_LENGTH=$(echo "$REX_RESULT" | jq -r '.text' 2>/dev/null | wc -c)
TESS_ORIGINAL_LENGTH=$(echo "$TESS_RESULT" | jq -r '.text' 2>/dev/null | wc -c)
TESS_FIXED_LENGTH=$(echo "$FIXED_RESULT" | jq -r '.text' 2>/dev/null | wc -c)

echo ""
echo "Content Lengths:"
echo "  Cleo: $CLEO_LENGTH characters"
echo "  Rex: $REX_LENGTH characters"
echo "  Tess (Original): $TESS_ORIGINAL_LENGTH characters"
echo "  Tess (Fixed): $TESS_FIXED_LENGTH characters"

# Check for potential issues
echo ""
echo "Potential Issues:"
echo "$CLEO_RESULT" | grep -q '\\"' && echo "  Cleo: ❌ Contains escaped quotes" || echo "  Cleo: ✅ Clean quotes"
echo "$REX_RESULT" | grep -q '\\"' && echo "  Rex: ❌ Contains escaped quotes" || echo "  Rex: ✅ Clean quotes"
echo "$TESS_RESULT" | grep -q '\\"' && echo "  Tess (Original): ❌ Contains escaped quotes" || echo "  Tess (Original): ✅ Clean quotes"
echo "$FIXED_RESULT" | grep -q '\\"' && echo "  Tess (Fixed): ❌ Contains escaped quotes" || echo "  Tess (Fixed): ✅ Clean quotes"

echo ""
if [ "$CLEO_VALID" = "✅" ] && [ "$REX_VALID" = "✅" ] && [ "$TESS_ORIGINAL_VALID" = "✅" ] && [ "$TESS_FIXED_VALID" = "✅" ]; then
    echo "🎉 ALL PATTERNS PRODUCE VALID JSON!"
    echo ""
    echo "This suggests the issue might be:"
    echo "1. Content-specific (what triggers empty blocks)"
    echo "2. Runtime conditions (how jq processes certain content)"
    echo "3. Error handling differences between agents"
    echo ""
    echo "The fix is still valid and should prevent cache_control errors."
else
    echo "❌ Some patterns produce invalid JSON - needs investigation"
fi
