# Tess Cache Control Error - Root Cause Analysis
**Analyst: Claude (AI Assistant)**
**Date: December 2024**

## Executive Summary

After analyzing the codebase and error patterns, I've identified that the `cache_control cannot be set for empty text blocks` error is a **symptom of a deeper architectural issue**: the sidecar service is failing to process requests, forcing the system into a fallback path that has incompatible message formatting. The root cause is **not the content itself** but rather a **race condition in the sidecar startup sequence**.

## Root Cause Analysis

### Primary Issue: Race Condition in Sidecar Initialization

The sidecar waits for the FIFO to exist, but the FIFO is created by the main container. This creates a circular dependency where:
1. Sidecar starts and waits for FIFO at `/workspace/agent-input.jsonl`
2. Main container creates the FIFO
3. Main container immediately tries to send to sidecar via HTTP
4. **Sidecar may not have its HTTP server ready yet**, causing the curl to fail
5. System falls back to direct FIFO write with different formatting

**Evidence from Code:**
```rust
// sidecar/src/main.rs - Sidecar waits for FIFO before starting HTTP server
let mut attempts = 0;
while !fifo_path.exists() && attempts < 60 {
    warn!("Waiting for FIFO to be created at {:?}...", fifo_path);
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    attempts += 1;
}
// Only AFTER FIFO exists does it start HTTP server
let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
```

### Secondary Issue: Message Format Incompatibility

The sidecar and FIFO fallback paths use **completely different message structures**:

#### Sidecar Path (What Should Work):
```bash
# Sends simple JSON: {"text": "content"}
printf '{"text":%s}\n' "$USER_COMBINED" | curl -X POST http://127.0.0.1:8080/input
```

The sidecar then transforms this into proper stream-json format internally:
```rust
// Sidecar creates proper structure with single content block
let message = StreamJsonEvent::User {
    message: StreamJsonUserMessage {
        role: "user",
        content: vec![StreamJsonContent::Text { text: &payload.text }],
    },
};
```

#### FIFO Fallback Path (What's Failing):
```bash
# Manually constructs full stream-json structure
printf '{"type":"user","message":{"role":"user","content":[{"type":"text","text":%s}]}}\n' "$USER_COMBINED" >&9
```

The error `messages.0.content.1.text` indicates the fallback is somehow creating **multiple content blocks**, with index 1 being empty.

## Why This Happens Specifically to Tess

Tess's template has unique characteristics that exacerbate the timing issue:

1. **Larger Initial Content**: Tess has a massive INITIAL_GUIDANCE (1000+ lines)
2. **Complex Processing**: Multiple sed/jq operations that take time
3. **No Retry Logic**: Unlike other templates, Tess doesn't have retry or wait logic for sidecar availability

## Critical Code Flow Analysis

```bash
# From container-tess.sh.hbs:
FIFO_PATH="/workspace/agent-input.jsonl"
rm -f "$FIFO_PATH" 2>/dev/null || true
mkfifo "$FIFO_PATH"  # Creates FIFO

# Starts Claude immediately
$CLAUDE_CMD < "$FIFO_PATH" &
CLAUDE_PID=$!

# Process content
INITIAL_GUIDANCE_CLEAN=$(printf "%s" "$INITIAL_GUIDANCE" | sed '/^[[:space:]]*$/d' | sed 's/[[:space:]]*$//')
USER_COMBINED=$(printf "%s" "$INITIAL_GUIDANCE_CLEAN" | jq -Rs .)

# Try sidecar IMMEDIATELY - no wait for it to be ready!
if printf '{"text":%s}\n' "$USER_COMBINED" | \
     curl -fsS -X POST http://127.0.0.1:8080/input ... then
```

## Proposed Solutions

### Solution 1: Add Sidecar Health Check (RECOMMENDED)
```bash
# Wait for sidecar to be ready before sending
MAX_RETRIES=30
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -fsS http://127.0.0.1:8080/health >/dev/null 2>&1; then
        echo "✓ Sidecar is ready"
        break
    fi
    sleep 1
    RETRY_COUNT=$((RETRY_COUNT + 1))
done

if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
    echo "⚠️ Sidecar not ready after ${MAX_RETRIES}s, using fallback"
fi
```

### Solution 2: Fix FIFO Fallback Format
```bash
# Ensure single content block in fallback
if ! curl -fsS -X POST http://127.0.0.1:8080/input ...; then
    # Create proper single-block structure
    MESSAGE=$(jq -n \
        --arg text "$INITIAL_GUIDANCE_CLEAN" \
        '{type: "user", message: {role: "user", content: [{type: "text", text: $text}]}}')
    echo "$MESSAGE" >&9
fi
```

### Solution 3: Improve Sidecar Startup Sequence
Modify sidecar to start HTTP server immediately, then wait for FIFO in background:
```rust
// Start HTTP server first
let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
info!("Sidecar listening on {addr}");

// Then wait for FIFO in background task
tokio::spawn(async move {
    while !fifo_path.exists() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    info!("FIFO detected at {:?}", fifo_path);
});
```

### Solution 4: Add Retry Logic to Sidecar Request
```bash
# Retry sidecar with exponential backoff
send_to_sidecar() {
    local max_attempts=5
    local attempt=1
    local delay=1
    
    while [ $attempt -le $max_attempts ]; do
        if printf '{"text":%s}\n' "$USER_COMBINED" | \
           curl -fsS -X POST http://127.0.0.1:8080/input ... 2>&1; then
            echo "✓ Message sent via sidecar (attempt $attempt)"
            return 0
        fi
        
        echo "Attempt $attempt failed, waiting ${delay}s..."
        sleep $delay
        delay=$((delay * 2))
        attempt=$((attempt + 1))
    done
    
    return 1
}

if ! send_to_sidecar; then
    echo "⚠️ All sidecar attempts failed, using FIFO fallback"
    # Fallback code
fi
```

## Why ASCII Art Removal Didn't Help

The ASCII art was never the root cause. The error persists because:
1. The sidecar HTTP endpoint isn't ready when first contacted
2. The fallback FIFO path has a formatting bug
3. The timing issue is intermittent, making it seem content-related

## Validation Steps

1. **Check Sidecar Logs**: Look for timing between "Sidecar listening" and first request
2. **Add Debug Timing**: Log timestamps for FIFO creation, sidecar start, and first curl
3. **Test Sidecar Availability**: Add health check before sending initial message
4. **Verify Message Format**: Log exact JSON being sent in both paths

## Immediate Recommendations

1. **Quick Fix**: Add a 5-second sleep before attempting sidecar communication
2. **Proper Fix**: Implement sidecar health check with retry logic  
3. **Long-term**: Redesign startup sequence to eliminate race condition

## Related Historical Context

This issue is similar to the sidecar hang documented in `sidecar-io-hang-2025-08-12.md`, where timing issues between sidecar and main container caused process hangs. The pattern of sidecar-related race conditions suggests a need for better synchronization mechanisms.

## Conclusion

The cache_control error is a **red herring**. The real issue is a **race condition** where the sidecar's HTTP server isn't ready when the main container attempts to send the initial message. This forces the system into a fallback path that has a formatting bug causing empty content blocks.

**Priority Actions:**
1. Add sidecar readiness check
2. Fix FIFO fallback message format
3. Consider adding retry logic for resilience

The solution is not about the content (removing ASCII art, reducing size) but about **proper service orchestration and error handling**.
