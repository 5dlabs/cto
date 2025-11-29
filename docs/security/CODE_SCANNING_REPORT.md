# Code Scanning Security Report

**Generated**: 2025-11-29  
**Total Open Alerts**: 21  
**Tools**: CodeQL (Rust), Trivy (Kubernetes)

---

## Executive Summary

| Severity | Count | Categories |
|----------|-------|------------|
| 🔴 HIGH | 16 | Cleartext transmission, HTTP URLs, RBAC networking, Read-only filesystem |
| 🟡 MEDIUM | 1 | Untrusted container registry |
| 🟢 LOW | 4 | Container UID/GID, capabilities |

---

## Priority 1: HIGH Severity Issues

### 1. Cleartext Transmission of Sensitive Information

**Alert**: `rust/cleartext-transmission`  
**Severity**: 🔴 HIGH  
**Tool**: CodeQL  
**File**: `tools/src/server/http_server.rs`  
**Open Alerts**: 1 (6 dismissed)

#### Description

The HTTP server transmits `session_id` values over potentially unencrypted HTTP connections. Session IDs are sensitive authentication tokens that could be intercepted in transit.

#### Affected Code

```rust:1611:tools/src/server/http_server.rs
let mut request_builder = client
    .post(&message_url)
    .header("Accept", "application/json,text/event-stream");

if let Some(ref sid) = mcp_session_id {
    request_builder = request_builder.header("Mcp-Session-Id", sid);
}
```

#### Risk Analysis

- **Attack Vector**: Network sniffing, man-in-the-middle (MITM)
- **Impact**: Session hijacking, unauthorized access to MCP sessions
- **Exploitability**: Moderate (requires network access)

#### Remediation

**Option A: Enforce HTTPS in Production** (Recommended)
```rust
fn validate_url_scheme(url: &str, allow_http: bool) -> Result<()> {
    if !allow_http && url.starts_with("http://") {
        return Err(anyhow::anyhow!(
            "HTTP URLs are not allowed in production. Use HTTPS."
        ));
    }
    Ok(())
}

// Use environment variable or config to control
let allow_http = std::env::var("ALLOW_INSECURE_HTTP")
    .map(|v| v == "true")
    .unwrap_or(false);
validate_url_scheme(&message_url, allow_http)?;
```

**Option B: Add TLS Verification**
```rust
let client = reqwest::Client::builder()
    .danger_accept_invalid_certs(false)  // Ensure TLS cert validation
    .min_tls_version(reqwest::tls::Version::TLS_1_2)
    .build()?;
```

**Option C: Mark as Acceptable Risk**
If this server only runs in trusted internal networks (e.g., Kubernetes cluster with mTLS via service mesh), document this assumption and dismiss the alert with a justification.

---

### 2. Cleartext Logging of Sensitive Information

**Alert**: `rust/cleartext-logging`  
**Severity**: 🔴 HIGH  
**Tool**: CodeQL  
**File**: `controller/src/tasks/code/naming.rs`  
**Open Alerts**: 4

#### Description

Test assertions log `uid_suffix` and `uid` values which CodeQL identifies as potentially sensitive. These are Kubernetes resource UIDs used in job naming, not user credentials.

#### Affected Code

```rust:398:controller/src/tasks/code/naming.rs
assert!(
    job_name.contains("pr1627"),
    "Label should take priority over env: {job_name}"
);
assert!(
    !job_name.contains("pr9999"),
    "Env var PR should not appear when label exists: {job_name}"
);
```

Lines 398, 402, 413, 424 - all in test functions.

#### Risk Analysis

- **False Positive Assessment**: ⚠️ **Likely False Positive**
- **Reason**: These are test assertions, not production logging
- **Data Type**: Kubernetes UIDs are not user-identifiable information

#### Remediation

**Option A: Suppress in Test Code** (Recommended)
```rust
#[allow(clippy::print_stdout)]  // Test output is acceptable
#[test]
fn job_name_prefers_label_over_env_var() {
    // ... test code
}
```

**Option B: Use Opaque Formatting**
```rust
assert!(
    job_name.contains("pr1627"),
    "Label should take priority over env: {}", 
    job_name.chars().take(20).collect::<String>()  // Truncate
);
```

**Option C: Dismiss with Justification**
These alerts can be safely dismissed with the note: "Test assertions logging Kubernetes resource UIDs, not user credentials."

---

### 3. Failure to Use HTTPS URLs

**Alert**: `rust/non-https-url`  
**Severity**: 🔴 HIGH  
**Tool**: CodeQL  
**Open Alerts**: 7

#### 3.1 Monitor CLI (`monitor/src/main.rs`)

**Lines**: 1967, 1987, 1999, 2004

#### Affected Code

```rust:1967:monitor/src/main.rs
let resp = reqwest::get(&url).await?;  // URL from openmemory_url config
```

```rust:1987:monitor/src/main.rs
.post(format!("{openmemory_url}/api/v1/search"))
```

#### Risk Analysis

The `openmemory_url` is configurable and may be set to HTTP for local development but should enforce HTTPS in production.

#### Remediation

```rust
fn get_openmemory_url() -> Result<String> {
    let url = std::env::var("OPENMEMORY_URL")
        .unwrap_or_else(|_| "https://localhost:8080".to_string());
    
    // Warn if using HTTP (allow for local dev)
    if url.starts_with("http://") && !url.contains("localhost") && !url.contains("127.0.0.1") {
        tracing::warn!("Using insecure HTTP connection to OpenMemory. Consider HTTPS for production.");
    }
    
    Ok(url)
}
```

#### 3.2 Integration Tests (`tools/tests/integration/real_servers/http_sse_servers.rs`)

**Lines**: 54, 137, 169

#### Risk Analysis

- **False Positive Assessment**: ⚠️ **Acceptable for Tests**
- **Reason**: Integration tests connecting to local test servers
- **Context**: Test infrastructure, not production code

#### Remediation

**Option A: Annotate Test File**
Add a comment at the top of the test file:
```rust
//! Integration tests for HTTP/SSE servers.
//! 
//! # Security Note
//! These tests use HTTP URLs to connect to local test servers.
//! This is acceptable for testing but should not be used in production.
```

**Option B: Dismiss Alerts**
Dismiss these alerts with justification: "Integration test code connecting to local test servers."

---

### 4. Kubernetes RBAC: Manage Networking Resources

**Alert**: `KSV056`  
**Severity**: 🔴 HIGH  
**Tool**: Trivy  
**Files**: `rbac/doc-server-networking.yaml`, `rbac/github-webhooks-networking.yaml`  
**Open Alerts**: 3

#### Description

Roles have permissions to create/update/delete networking resources (services, endpoints, ingresses). This is flagged because networking access can be used to intercept traffic.

#### Current Configuration

```yaml:14:infra/gitops/rbac/github-webhooks-networking.yaml
rules:
  - apiGroups: [""]
    resources: ["services"]
    verbs: ["create"]
  - apiGroups: [""]
    resources: ["services"]
    resourceNames: ["github-eventsource-svc"]
    verbs: ["get", "list", "watch", "update", "patch", "delete"]
```

#### Risk Analysis

- **Purpose**: These roles exist for ArgoCD to manage specific networking resources
- **Scope**: Already limited to specific `resourceNames`
- **Assessment**: This is intentional and necessary for GitOps automation

#### Remediation

**Option A: Document the Necessity** (Recommended)
Add comments to the YAML files explaining why these permissions are required:

```yaml
# This role allows ArgoCD to manage the github-eventsource service.
# Permissions are scoped to specific resourceNames for minimal access.
# Security review: 2025-11-29 - Accepted risk for GitOps automation.
```

**Option B: Further Restrict Verbs**
If possible, reduce verbs to only what's needed:
```yaml
verbs: ["get", "list", "watch", "patch"]  # Remove create/delete if not needed
```

**Note**: Files may have been moved. Current location is `infra/gitops/rbac/`.

---

### 5. Kubernetes: Root Filesystem Not Read-Only

**Alert**: `KSV014`  
**Severity**: 🔴 HIGH  
**Tool**: Trivy  
**File**: `resources/doc-server-databases/postgres.yaml`  
**Open Alerts**: 2 (containers: `postgres`, `ensure-permissions`)

#### Description

PostgreSQL containers don't have read-only root filesystem, which could allow attackers to modify container files if compromised.

#### Risk Analysis

- **Challenge**: PostgreSQL requires write access to `/var/lib/postgresql/data`
- **Assessment**: This is a common exception for database containers

#### Remediation

**Option A: Add Read-Only with Volume Mounts** (Recommended)
```yaml
containers:
- name: postgres
  securityContext:
    readOnlyRootFilesystem: true
  volumeMounts:
  - name: data
    mountPath: /var/lib/postgresql/data
  - name: run
    mountPath: /var/run/postgresql
  - name: tmp
    mountPath: /tmp
volumes:
- name: data
  persistentVolumeClaim:
    claimName: postgres-data
- name: run
  emptyDir: {}
- name: tmp
  emptyDir: {}
```

**Option B: Accept Risk with Documentation**
If the above isn't feasible, document the exception:
```yaml
# Security exception: PostgreSQL requires writable filesystem for data directory.
# Mitigated by: Network policies, pod security policies, non-root user.
```

**Note**: File path in scan (`resources/doc-server-databases/postgres.yaml`) may have been moved or deleted.

---

## Priority 2: MEDIUM Severity Issues

### 6. Untrusted Container Registry

**Alert**: `KSV0125`  
**Severity**: 🟡 MEDIUM  
**Tool**: Trivy  
**File**: `resources/doc-server-databases/postgres.yaml`  
**Open Alerts**: 1

#### Description

The PostgreSQL container uses an image from Docker Hub (`postgres:*`) which isn't in the list of trusted registries.

#### Remediation

**Option A: Use Verified/Official Images**
Docker Hub official images (like `postgres`) are generally trusted, but consider:

```yaml
# Use digest for immutability
image: postgres:16@sha256:abc123...
```

**Option B: Mirror to Private Registry**
```yaml
image: ghcr.io/5dlabs/postgres:16  # Or your private registry
```

**Option C: Accept with Documentation**
```yaml
# Uses official PostgreSQL image from Docker Hub.
# Verified as official image: https://hub.docker.com/_/postgres
```

---

## Priority 3: LOW Severity Issues

### 7. Container Runs with Low UID/GID

**Alert**: `KSV020`, `KSV021`  
**Severity**: 🟢 LOW  
**Tool**: Trivy  
**File**: `resources/doc-server-databases/redis.yaml`  
**Open Alerts**: 2

#### Description

Redis container runs with UID/GID less than 10000, which could conflict with host system users.

#### Remediation

```yaml
securityContext:
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
```

**Note**: Redis Alpine image may require specific UID. Test thoroughly before changing.

---

### 8. Container Capabilities Not Dropped

**Alert**: `KSV106`  
**Severity**: 🟢 LOW  
**Tool**: Trivy  
**File**: `resources/doc-server-databases/postgres.yaml`  
**Open Alerts**: 1

#### Description

The `ensure-permissions` init container doesn't drop all capabilities.

#### Remediation

```yaml
initContainers:
- name: ensure-permissions
  securityContext:
    capabilities:
      drop:
      - ALL
```

---

## Dismissed Alerts Summary

The following alerts were previously dismissed (not requiring action):

| Alert # | Rule | File | Reason |
|---------|------|------|--------|
| 339 | rust/cleartext-transmission | http_server.rs:3407 | Dismissed |
| 338 | rust/cleartext-transmission | http_server.rs:3378 | Dismissed |
| 337 | rust/cleartext-transmission | http_server.rs:3348 | Dismissed |
| 336 | rust/cleartext-transmission | http_server.rs:2531 | Dismissed |
| 335 | rust/cleartext-transmission | http_server.rs:2514 | Dismissed |
| 334 | rust/cleartext-transmission | http_server.rs:2491 | Dismissed |
| 333 | rust/cleartext-transmission | http_server.rs:1545 | Dismissed |
| 332 | rust/cleartext-transmission | http_server.rs:1568 | Dismissed |
| 331 | rust/cleartext-logging | http_transport_tests.rs:227 | Fixed |

---

## Recommended Action Plan

### Immediate (This Sprint)

1. **Review and dismiss false positives**:
   - `rust/cleartext-logging` in test code (alerts 304-307)
   - `rust/non-https-url` in integration tests (alerts 340-342)

2. **Add HTTPS enforcement in monitor CLI**:
   - Implement URL validation helper
   - Warn when using HTTP outside localhost

### Short-Term (Next Sprint)

3. **Kubernetes security hardening**:
   - Add `readOnlyRootFilesystem: true` to Postgres with proper volume mounts
   - Drop all capabilities from init containers
   - Review RBAC roles and document necessity

4. **Review cleartext transmission alert** (#347):
   - Determine if HTTPS should be enforced
   - Or dismiss with documentation if running in secure cluster

### Long-Term

5. **Establish container image policy**:
   - Mirror critical images to private registry
   - Use image digests for immutability
   - Document trusted registries list

6. **Create security exception process**:
   - Document process for accepting security risks
   - Regular review of dismissed alerts

---

## References

- [KSV014 - Read-only Root Filesystem](https://avd.aquasec.com/misconfig/ksv014)
- [KSV020 - Run as Non-root User](https://avd.aquasec.com/misconfig/ksv020)
- [KSV056 - Kubernetes Networking Permissions](https://avd.aquasec.com/misconfig/ksv056)
- [KSV106 - Container Capabilities](https://avd.aquasec.com/misconfig/ksv106)
- [KSV0125 - Trusted Registries](https://avd.aquasec.com/misconfig/ksv0125)
- [CodeQL Rust Security Queries](https://codeql.github.com/codeql-query-help/rust/)

