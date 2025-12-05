//! Twitter/X bookmark monitoring module.
//!
//! Provides polling and parsing for Twitter bookmarks.

mod parser;
mod poller;
mod types;

pub use parser::BookmarkParser;
pub use poller::{BookmarkPoller, PollConfig, PollState};
pub use types::{Author, Bookmark, Media};
