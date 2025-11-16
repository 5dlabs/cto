//! Shared cleanup helpers (labels, annotations, TTL parsing)

use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

/// Label used to classify cleanup scope (system/run/lock)
pub const LABEL_CLEANUP_SCOPE: &str = "cleanup.cto.dev/scope";
/// Label that stores the owning run name
pub const LABEL_CLEANUP_RUN: &str = "cleanup.cto.dev/run";
/// Label that stores the type of run (coderun/docsrun)
pub const LABEL_CLEANUP_KIND: &str = "cleanup.cto.dev/run-kind";

/// Cleanup scope values
pub const SCOPE_RUN: &str = "run";
pub const SCOPE_SYSTEM: &str = "system";
pub const SCOPE_LOCK: &str = "lock";

/// Annotation to preserve resources (skip TTL cleanup)
pub const ANNOTATION_PRESERVE: &str = "cleanup.cto.dev/preserve";
/// Annotation to override TTL in seconds for an individual run/resource
pub const ANNOTATION_TTL_SECONDS: &str = "cleanup.cto.dev/ttl-seconds";

/// Returns true if cleanup should be skipped because the resource is marked as preserved.
#[must_use]
pub fn is_preserved(meta: &ObjectMeta) -> bool {
    meta.annotations
        .as_ref()
        .and_then(|annotations| annotations.get(ANNOTATION_PRESERVE))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

/// Extracts a TTL override from the object's annotations (in seconds).
#[must_use]
pub fn ttl_override_seconds(meta: &ObjectMeta) -> Option<u64> {
    meta.annotations
        .as_ref()
        .and_then(|annotations| annotations.get(ANNOTATION_TTL_SECONDS))
        .and_then(|raw| raw.trim().parse::<u64>().ok())
}
