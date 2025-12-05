//! Research storage module.
//!
//! Stores analyzed content as markdown with YAML frontmatter.

mod dedup;
mod index;
mod markdown;

pub use dedup::DedupTracker;
pub use index::{IndexEntry, ResearchIndex};
pub use markdown::{MarkdownWriter, ResearchEntry};
