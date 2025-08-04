/*
 * 5D Labs Agent Platform - Kubernetes Orchestrator for AI Coding Agents
 * Copyright (C) 2025 5D Labs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Orchestrator core library
//!
//! This crate provides the core functionality for the unified orchestration service,
//! including Kubernetes client wrapper, job orchestration, and request handling.

pub mod tasks;
pub mod crds;

// Re-export commonly used types
pub use tasks::config::ControllerConfig;
pub use crds::{CodeRun, CodeRunSpec, CodeRunStatus, DocsRun, DocsRunSpec, DocsRunStatus};
