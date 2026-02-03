//! CTO Lite MCP Server - Tauri-compatible desktop MCP server
//!
//! This module provides a simplified Model Context Protocol (MCP) server
//! designed for the CTO App desktop application (Tauri + React).
//!
//! Core functionality retained:
//! - Task sync (read/update task files)
//! - Workflow triggering (submit Argo workflows)
//!
//! Enterprise features removed:
//! - Linear integration
//! - Database dependencies
//! - Complex PM server integration

pub mod desktop_server;
pub mod tools;
