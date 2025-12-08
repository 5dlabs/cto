//! Storage layer for task persistence.

mod file;
mod traits;

pub use file::FileStorage;
pub use traits::{Storage, UpdateStatusResult};
