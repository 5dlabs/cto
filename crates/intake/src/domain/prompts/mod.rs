//! Prompt generation domain for intake workflow Session 2.
//!
//! This module handles AI-based prompt generation for individual tasks,
//! running after task planning (Session 1) completes.
//!
//! ## Workflow
//!
//! 1. Session 1 generates `tasks.json` with task definitions
//! 2. `split_tasks()` divides tasks.json into individual `task-N.json` files
//! 3. `PromptGenerator` processes each task file with AI to generate:
//!    - `prompt.md` - Detailed markdown prompt
//!    - `prompt.xml` - Structured XML prompt
//!    - `acceptance.md` - Acceptance criteria checklist
//!    - `code-examples.md` (optional) - Code examples for implementation
//!
//! The `templates` module provides template-based fallback generation.

mod generator;
mod split;
pub mod templates;

pub use generator::{GeneratePromptsConfig, PromptFiles, PromptGenerator, PromptGeneratorResult};
pub use split::{split_tasks, SplitTasksResult};
pub use templates::generate_all_docs;
