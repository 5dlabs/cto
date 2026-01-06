//! Twitter/X bookmark monitoring module.
//!
//! Provides polling and parsing for Twitter bookmarks.

mod parser;
mod poller;
mod types;

pub use parser::BookmarkParser;
pub use poller::{BookmarkPoller, FetchResult, PollConfig, PollState};
pub use types::{tweet_id_to_datetime, tweet_id_within_days, Author, Bookmark, Media};
