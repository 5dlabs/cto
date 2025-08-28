# Root Cause Analysis of Tess Cache Control Error

**Author:** Gemini
**Date:** August 29, 2024
**Source Analysis:** `tess-cache-control-analysis.md`

---

## 1. Executive Summary

The recurring `cache_control cannot be set for empty text blocks` error is a **symptom**, not the root cause of the problem.

The investigation outlined in the source analysis document correctly identifies that the primary failure lies in the communication with the sidecar container. When the initial attempt to send guidance via the sidecar's `/input` endpoint fails, the system triggers a fallback mechanism. This fallback path constructs a malformed message that the Claude API rejects, producing the `cache_control` error.

The immediate priority is to diagnose and resolve the sidecar communication failure. Strengthening the fallback path is a secondary goal to improve system robustness.

## 2. Root Cause Analysis

The issue stems from a two-part failure sequence:

### 🎯 Primary Root Cause: Sidecar Communication Failure

The core of the problem is the failure of the primary communication path, where the Tess container attempts to send the `INITIAL_GUIDANCE` payload to its sidecar via an HTTP POST request.

-   **Evidence:** The logs clearly state: `⚠️ Sidecar /input failed, falling back to direct FIFO write`.
-   **Impact:** This failure prevents the intended, primary logic from executing and forces the system onto a secondary, less-tested fallback path.
-   **Possible Reasons for Failure:**
    1.  **Sidecar Unhealthy:** The sidecar's web server may not be running or may be in a crash loop.
    2.  **Network Connectivity:** A networking issue within the pod could be preventing the `tess` container from reaching the sidecar on `127.0.0.1:8080`.
    3.  **Malformed Payload:** The JSON payload being sent to the sidecar via `curl` could be malformed, causing the `curl` command itself or the sidecar's server to reject the request.

### 🎯 Secondary Issue (Symptom): Flawed Fallback Mechanism

When the sidecar communication fails, the script falls back to writing the message directly to a FIFO pipe. This path has a critical flaw.

-   **Evidence:** The API error `messages.0.content.1.text: cache_control cannot be set for empty text blocks` indicates that the API received a message with multiple content blocks, and the second block (at index `1`) was empty.
-   **Impact:** The message format in the fallback path differs from the format used in the primary sidecar path. This discrepancy leads to the creation of an invalid request structure that the Claude API rejects.
-   **Conclusion:** The `cache_control` error is a direct result of the flawed fallback logic and would not occur if the primary sidecar path was working correctly.

## 3. Proposed Solutions & Action Plan

The action plan must prioritize fixing the root cause (the sidecar) before addressing the symptom (the fallback path).

### 🚨 Priority 1: Investigate and Fix the Sidecar Failure

1.  **Verify Sidecar Health:**
    -   Exec into the running Tess pod and directly check the sidecar's health endpoint:
        ```bash
        curl http://127.0.0.1:8080/health
        ```
2.  **Analyze Sidecar Logs:**
    -   Inspect the logs for the `sidecar` container specifically. Look for startup errors, stack traces, or any logs related to incoming requests on the `/input` endpoint.
        ```bash
        kubectl logs [tess-pod-name] -c sidecar -n agent-platform
        ```
3.  **Isolate the Endpoint:**
    -   Perform a manual `curl` from within the `tess` container to test the sidecar's `/input` endpoint with a minimal, valid payload. This will determine if the endpoint itself is functional.
        ```bash
        curl -X POST -H 'Content-Type: application/json' \
          -d '{"text": "simple test"}' \
          http://127.0.0.1:8080/input -v
        ```
4.  **Inspect the Full Payload:**
    -   Log the exact JSON payload being sent by the primary `curl` command just before it is executed. It's possible that an issue with shell variable expansion or `jq` processing is creating invalid JSON that causes the request to fail.

### 🎯 Priority 2: Harden the Fallback Mechanism

Once the sidecar is stable, the fallback path should be made more resilient.

1.  **Correct Message Formatting:**
    -   Modify the `printf` command in the fallback path to ensure it constructs a valid JSON object with a single, non-empty content block, mirroring the structure expected by the Claude API.
2.  **Add Content Validation:**
    -   Implement a check before the fallback `printf` to verify that the `USER_COMBINED` variable is not null or empty, preventing the creation of an empty text block.
3.  **Improve Logging:**
    -   Add more detailed logging to the fallback path to record the exact message being sent (excluding sensitive data) to simplify future debugging.
