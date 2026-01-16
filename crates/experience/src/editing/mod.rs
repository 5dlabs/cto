//! Context editing strategies.
//!
//! Declarative strategies for managing LLM context windows without
//! modifying the original session data.

mod strategies;
mod token_counter;

pub use strategies::{EditParams, EditResult, EditStrategy};
pub use token_counter::TokenCounter;
