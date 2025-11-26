#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::disallowed_macros)]
#![allow(clippy::ignore_without_reason)]
#[cfg(test)]
mod tests {

    #[test]
    fn test_sse_url_detection() {
        // URLs that should trigger SSE detection
        assert!(is_sse_url("http://example.com/sse"));
        assert!(is_sse_url("https://rustdocs-server.com/sse"));
        assert!(is_sse_url("http://localhost:3000/sse"));
        assert!(is_sse_url("https://api.example.com/v1/sse"));
        
        // URLs that should NOT trigger SSE detection
        assert!(!is_sse_url("http://example.com/api"));
        assert!(!is_sse_url("https://mcp.solana.com/mcp"));
        assert!(!is_sse_url("http://localhost:3000/mcp"));
        assert!(!is_sse_url("https://example.com/sse/sub"));
        assert!(!is_sse_url("https://example.com/sse?param=value"));
        assert!(!is_sse_url("https://example.com/api/sse/endpoint"));
    }

    #[test]
    fn test_known_server_urls() {
        // Test known server URLs
        
        // Solana - direct HTTP
        assert!(!is_sse_url("https://mcp.solana.com/mcp"));
        
        // Rust Docs - SSE
        assert!(is_sse_url("http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse"));
        
        // Our own server - direct HTTP
        assert!(!is_sse_url("http://localhost:3000/mcp"));
        assert!(!is_sse_url("http://tools.mcp.svc.cluster.local:3000/mcp"));
    }

    fn is_sse_url(url: &str) -> bool {
        url.ends_with("/sse")
    }
}