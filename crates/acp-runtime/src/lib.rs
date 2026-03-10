//! Shared ACP runtime helpers for CTO services.

pub mod client;
pub mod registry;
pub mod server;
pub mod types;

pub use client::{run_oneshot_prompt, AcpClientProfile};
pub use registry::{AcpRuntimeRegistry, RuntimeSelection};
pub use server::{caller_from_meta, ensure_allowed_caller, serve_stdio_agent, CallerContext};
pub use types::{
    AcpImplementationInfo, AcpPermissionPolicy, AcpPromptRequest, AcpPromptResult, AcpRunState,
    AcpSessionMetadata,
};
