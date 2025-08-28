# 🔍 Tess Debug Logging Guide

## Overview

Comprehensive debug logging has been added to Tess to help identify and resolve cache_control errors. This guide explains what information will now be logged and how to interpret it.

## Debug Session Tracking

Each Tess run now includes:
```
🔍 Debug Session: 2025-08-28 21:50:47 - PID: 12345
📊 Log Level: DEBUG (showing JSON construction details)
```

**What to look for:**
- **Timestamp**: When the session started
- **PID**: Process ID for correlation with system logs
- **Log Level**: Confirms debug mode is active

## JSON Construction Process

### Phase 1: Content Processing
```
🔍 DEBUG: Starting JSON construction process...
📝 Original INITIAL_GUIDANCE length: 1234 characters
📝 Original INITIAL_GUIDANCE preview: 🧪 **TESS QA TESTING WORKFLOW**...

🧹 Cleaning INITIAL_GUIDANCE (removing empty lines and trailing spaces)...
✅ Cleaned INITIAL_GUIDANCE length: 1189 characters
✅ Cleaned INITIAL_GUIDANCE preview: 🧪 **TESS QA TESTING WORKFLOW**...
```

**What to check:**
- Content length reduction shows cleaning is working
- Preview shows content looks reasonable
- Any dramatic length changes indicate issues

### Phase 2: JSON Construction
```
🔧 Constructing JSON for sidecar transmission...
📤 Sidecar JSON payload length: 1250 characters
📤 Sidecar JSON payload preview: {"text":"🧪 **TESS QA TESTING...
🔍 Full sidecar JSON structure:
{
  "text": "🧪 **TESS QA TESTING WORKFLOW**\n\nYou are Tess..."
}
```

**Critical checks:**
- **JSON parsing succeeds**: If you see "❌ Sidecar JSON parsing failed", the JSON is malformed
- **Single text field**: Should only have one "text" field, not multiple content blocks
- **No empty strings**: The text content should not be empty or null

### Phase 3: Transmission Path

#### Sidecar Path (Preferred):
```
📡 Sending to sidecar /input endpoint...
✅ Initial QA guidance sent via sidecar /input
```

#### FIFO Fallback Path:
```
⚠️ Sidecar /input failed, falling back to direct FIFO write
🔧 Constructing FIFO JSON payload...
📤 FIFO JSON payload length: 1300 characters
📤 FIFO JSON payload preview: {"type":"user","message":{"role":"user"...
🔍 Full FIFO JSON structure:
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "type": "text",
        "text": "🧪 **TESS QA TESTING WORKFLOW**..."
      }
    ]
  }
}
✅ Initial QA guidance sent via FIFO fallback
```

**What to check:**
- Which path is used (sidecar vs FIFO)
- If FIFO is used, check if sidecar health check failed
- JSON structure in FIFO path should have proper message format

## Claude Execution Monitoring

```
🤖 Monitoring Claude execution...
🔍 Claude PID: 54321
⏳ Waiting for Claude process to complete...
🔚 Claude process completed with exit code: 0
```

**Success indicators:**
- Exit code 0 = Success
- Exit code 1 = Usually cache_control or API error

## Error Detection

### Cache Control Error Pattern:
If you see exit code 1, it typically means:
```
⚠️ Tess QA testing exited with code: 1
📊 Final status: FAILURE (exit code 1)
🔍 This usually indicates a cache_control or API error occurred
```

### What to Investigate:
1. **Check JSON structure** - Look for malformed JSON in the logs
2. **Empty content blocks** - Ensure no empty text fields
3. **Multiple content blocks** - Should only have one content block
4. **Sidecar vs FIFO** - See which transmission path was used

## Log Analysis Checklist

When debugging cache_control errors, check for:

### ✅ GOOD Signs:
- [ ] JSON parsing succeeds (`jq .` works)
- [ ] Single content block only
- [ ] Non-empty text content
- [ ] Reasonable content length
- [ ] Exit code 0

### ❌ BAD Signs:
- [ ] "JSON parsing failed" messages
- [ ] Multiple content blocks
- [ ] Empty or null text fields
- [ ] Extremely short content
- [ ] Exit code 1

## Sample Debug Output

### Successful Run:
```
🔍 Debug Session: 2025-08-28 21:50:47 - PID: 12345
📝 Original INITIAL_GUIDANCE length: 1234 characters
✅ Cleaned INITIAL_GUIDANCE length: 1189 characters
📤 Sidecar JSON payload length: 1250 characters
🔍 Full sidecar JSON structure: { "text": "content..." }
✅ Initial QA guidance sent via sidecar /input
🔚 Claude process completed with exit code: 0
✅ Tess QA testing completed successfully
```

### Problematic Run:
```
🔍 Debug Session: 2025-08-28 21:50:47 - PID: 12345
📝 Original INITIAL_GUIDANCE length: 1234 characters
✅ Cleaned INITIAL_GUIDANCE length: 1189 characters
❌ Sidecar JSON parsing failed: {"text":"
⚠️ Sidecar /input failed, falling back to direct FIFO write
🔍 Full FIFO JSON structure: { "content": ["", "valid content"] }
🔚 Claude process completed with exit code: 1
⚠️ Tess QA testing exited with code: 1
```

## Next Steps

1. **Deploy this version** with debug logging
2. **Run Tess workflow** and capture the debug output
3. **Analyze the logs** using this guide
4. **Identify any JSON construction issues**
5. **Apply additional fixes** if needed

The debug logging will show exactly what's happening with the JSON construction and help pinpoint any remaining issues causing cache_control errors.
