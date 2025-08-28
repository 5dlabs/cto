# Tess Cache Control Error - Root Cause Analysis

## Executive Summary

Despite removing ASCII art from Tess's INITIAL_GUIDANCE, the `cache_control cannot be set for empty text blocks` error persists. This document provides a comprehensive analysis of potential root causes based on the error logs and system architecture.

## Error Details

**Error Message:**
```
API Error: 400 {"type":"error","error":{"type":"invalid_request_error","message":"messages.0.content.1.text: cache_control cannot be set for empty text blocks"},"request_id":"req_011CSamGgpKNZKe4L7k3yqnh"}
```

**Key Information:**
- **Request ID:** `req_011CSamGgpKNZKe4L7k3yqnh` (different from previous errors)
- **Error Location:** `messages.0.content.1.text` (second content block is empty)
- **Timing:** Occurs during initial Tess initialization
- **Sidecar Status:** ❌ **FAILING** - Falls back to FIFO write
- **Critical Finding:** Error occurs in **FALLBACK PATH**, not primary sidecar path

## Current Tess Template Analysis

### INITIAL_GUIDANCE Processing Flow

```bash
# 1. Raw INITIAL_GUIDANCE content
INITIAL_GUIDANCE="🧪 **TESS ULTRA-STRICT QA TESTING WORKFLOW**
[large multi-line content]"

# 2. Cleanup processing
INITIAL_GUIDANCE_CLEAN=$(printf "%s" "$INITIAL_GUIDANCE" | sed '/^[[:space:]]*$/d' | sed 's/[[:space:]]*$//')

# 3. JSON formatting
USER_COMBINED=$(printf "%s" "$INITIAL_GUIDANCE_CLEAN" | jq -Rs .)

# 4. Message construction (FALLBACK PATH)
printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$USER_COMBINED"
```

## Potential Root Causes

### 0. **Sidecar Failure** 🎯 **PRIMARY ROOT CAUSE**

**Hypothesis:** The sidecar `/input` endpoint is failing, forcing fallback to FIFO write, which has different message formatting that causes the cache_control error.

**Evidence:**
- **Log:** `⚠️ Sidecar /input failed, falling back to direct FIFO write`
- **Primary Path:** Sidecar HTTP endpoint (fails)
- **Fallback Path:** Direct FIFO write (where error occurs)
- **Different Message Formats:** Sidecar vs FIFO use different JSON structures

**Critical Finding:** The cache_control error only occurs because the sidecar fails first!

### 1. **Multi-Content Block Issue** 🎯 **SECONDARY - FALLBACK PATH**

**Hypothesis:** The error indicates `messages.0.content.1.text` (second content block) is empty. The fallback FIFO path sends a different message structure than the sidecar path.

**Evidence:**
- Error: `messages.0.content.1.text` (index 1 suggests multiple content blocks)
- **Fallback Template:** `[{"type":"text","text":%s}]` (single content block)
- **Sidecar Template:** `{"text":%s}` (different format)
- FIFO path may be creating multiple content blocks unexpectedly

**Test:** Compare sidecar vs FIFO message formats and identify why FIFO creates multiple blocks.

### 2. **JSON Processing Issues**

**Hypothesis:** The `jq -Rs .` processing or printf formatting is creating malformed JSON.

**Potential Issues:**
- `jq -Rs .` might be adding unexpected formatting
- printf with `%s` might be interpreting content incorrectly
- Hidden characters or encoding issues

**Evidence:**
- The error occurs at JSON processing level
- Sidecar receives the request but Claude API rejects it

### 3. **Template Variable Expansion**

**Hypothesis:** Shell variables in INITIAL_GUIDANCE are not being expanded correctly.

**Variables in Template:**
- `$TASK_ID` - Task identifier
- `$PR_NUM` - Pull request number
- Various backslash-escaped commands

**Evidence:**
- Template contains unexpanded variables
- Variable expansion might create empty sections

### 4. **Content Size/Length Issues**

**Hypothesis:** The INITIAL_GUIDANCE content is too large or contains problematic patterns.

**Content Analysis:**
- Template is ~1000+ lines long
- Contains complex formatting, emojis, and special characters
- Multiple sections with conditional logic

**Evidence:**
- Large content might trigger API limits
- Complex formatting could cause parsing issues

### 5. **Sidecar Processing Issues**

**Hypothesis:** The sidecar is modifying the request in an unexpected way.

**Evidence:**
- Sidecar logs show successful initialization
- Request reaches Claude API (gets request ID)
- But API rejects with content structure error

### 6. **Race Condition Issues**

**Hypothesis:** Timing issues between sidecar startup and message sending.

**Evidence:**
- Sidecar starts successfully
- Message sent immediately after sidecar confirmation
- But content structure is corrupted

## Detailed Investigation Steps

### Step 1: Content Structure Analysis

```bash
# Extract and analyze the actual INITIAL_GUIDANCE content
kubectl get configmap controller-claude-templates -n agent-platform \
  -o jsonpath='{.data.code_container-tess\.sh\.hbs}' > tess-template-raw.sh

# Extract INITIAL_GUIDANCE section
sed -n '/INITIAL_GUIDANCE="/,/^"/p' tess-template-raw.sh > initial-guidance-content.txt

# Analyze content structure
wc -l initial-guidance-content.txt  # Line count
grep -c '^[[:space:]]*$' initial-guidance-content.txt  # Empty lines
grep -o '[^[:print:]]' initial-guidance-content.txt | sort | uniq -c  # Non-printable chars
```

### Step 2: JSON Processing Debug

```bash
# Test the exact processing pipeline
INITIAL_GUIDANCE="[extracted content]"
INITIAL_GUIDANCE_CLEAN=$(printf "%s" "$INITIAL_GUIDANCE" | sed '/^[[:space:]]*$/d' | sed 's/[[:space:]]*$//')
USER_COMBINED=$(printf "%s" "$INITIAL_GUIDANCE_CLEAN" | jq -Rs .)

# Check JSON structure
echo "$USER_COMBINED" | jq .

# Test message construction
printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$USER_COMBINED" | jq .
```

### Step 3: Sidecar Request Capture

```bash
# Enable sidecar request logging
kubectl logs -f [tess-pod-name] -c sidecar 2>&1 | grep -E "(POST|input|cache_control)"

# Capture actual HTTP requests
kubectl exec [tess-pod-name] -c sidecar -- tcpdump -i any port 8080 -w /tmp/sidecar-traffic.pcap
```

### Step 4: Claude API Request Analysis

```bash
# Monitor exact request sent to Claude
kubectl logs -f [tess-pod-name] 2>&1 | grep -A 10 -B 10 "req_011CSamGgpKNZKe4L7k3yqnh"

# Check for multiple content blocks
kubectl logs -f [tess-pod-name] 2>&1 | grep -i "content"
```

## Potential Solutions

### Solution 1: Explicit Single Content Block
```bash
# Force single content block structure
MESSAGE='{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "type": "text",
        "text": "'$USER_COMBINED'"
      }
    ]
  }
}'
```

### Solution 2: Content Validation
```bash
# Validate content before sending
if [[ -z "$USER_COMBINED" ]] || [[ "$USER_COMBINED" == '""' ]]; then
  echo "ERROR: Empty content detected"
  exit 1
fi
```

### Solution 3: Simplified Content Structure
```bash
# Use simpler content structure
printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":"%s"}]}}\n' "$USER_COMBINED"
```

### Solution 4: Raw Content Approach
```bash
# Send raw content without jq processing
USER_COMBINED="$INITIAL_GUIDANCE_CLEAN"
```

## Immediate Investigation Priorities

### 🚨 **URGENT: Sidecar Failure Analysis**

**Why Sidecar Fails:**
```bash
# Primary path (FAILS)
if printf '{"text":%s}\n' "$USER_COMBINED" | \
     curl -fsS -X POST http://127.0.0.1:8080/input \
       -H 'Content-Type: application/json' \
       --data-binary @- >/dev/null 2>&1; then
  echo "✓ Initial QA guidance sent via sidecar /input"
else
  echo "⚠️ Sidecar /input failed, falling back to direct FIFO write"
  # FALLBACK PATH - This is where the error occurs!
fi
```

**Investigation Steps:**
1. **Check sidecar health:** `curl http://127.0.0.1:8080/health`
2. **Monitor sidecar logs:** `kubectl logs [tess-pod] -c sidecar`
3. **Test sidecar endpoint:** `curl -X POST http://127.0.0.1:8080/input -d '{"test":"data"}'`
4. **Check network connectivity** between Tess and sidecar

### 🎯 **Secondary: Fallback Path Analysis**

1. **Extract and examine** the actual INITIAL_GUIDANCE content from the running pod
2. **Test JSON processing** pipeline locally with the exact content
3. **Compare message formats** between sidecar and FIFO paths
4. **Debug multi-content block** creation in FIFO path

## Data Points Needed

- [ ] Actual INITIAL_GUIDANCE content from failing pod
- [ ] JSON structure after jq processing
- [ ] Exact HTTP request sent to sidecar
- [ ] Sidecar processing logs
- [ ] Claude API request/response details

## Collaboration Notes

This analysis is being shared with a second Oracle agent for parallel investigation. Key areas for the second agent to focus on:

1. **JSON Processing Pipeline** - Deep dive into jq formatting
2. **Claude API Message Structure** - Compare with working examples
3. **Sidecar Request Modification** - Check if sidecar alters content
4. **Template Variable Expansion** - Verify all variables are properly substituted

## Key Findings Summary

### 🎯 **PRIMARY ROOT CAUSE: Sidecar Failure**
- **Evidence:** `⚠️ Sidecar /input failed, falling back to direct FIFO write`
- **Impact:** Forces system into fallback path with different message formatting
- **Result:** Cache_control error occurs in FIFO fallback, not primary path

### 🎯 **SECONDARY ISSUE: Message Format Mismatch**
- **Primary Path:** `{"text":%s}` (sidecar format)
- **Fallback Path:** `{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}` (FIFO format)
- **Issue:** FIFO format creates multiple content blocks, causing index 1 to be empty

### 🎯 **CRITICAL INSIGHT: Fix Sidecar First**
The cache_control error is a **symptom**, not the root cause. The real issue is the sidecar failing, which forces the problematic fallback path.

## Immediate Action Items

### 🚨 **URGENT: Fix Sidecar Communication**
1. **Debug sidecar `/input` endpoint** - Why is it failing?
2. **Check sidecar health** - Is the service running properly?
3. **Verify network connectivity** - Can Tess reach sidecar?
4. **Monitor sidecar logs** - What errors are occurring?

### 🎯 **Secondary: Improve Fallback Path**
1. **Fix message formatting** in FIFO fallback
2. **Ensure single content block** structure
3. **Add content validation** before sending to Claude
4. **Improve error handling** and logging

## Conclusion

**The ASCII art was never the issue.** The cache_control error is caused by:

1. **Sidecar failure** → Forces fallback to FIFO
2. **Different message formats** → FIFO creates multiple content blocks
3. **Empty content block** → Index 1 has no content, triggering cache_control error

**Priority:** Fix the sidecar communication first, then address any remaining fallback path issues.

**For the second Oracle:** Focus on sidecar health checks and network connectivity between Tess container and sidecar service.
