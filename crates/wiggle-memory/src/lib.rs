//! Tervixa Wiggle personal memory engine.

mod extractor;
mod store;

pub use extractor::MemoryExtractor;
pub use store::{MemoryEntry, MemoryStore};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("memory db error: {0}")]
    Db(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type MemoryResult<T> = Result<T, MemoryError>;

impl From<rusqlite::Error> for MemoryError {
    fn from(e: rusqlite::Error) -> Self {
        MemoryError::Db(e.to_string())
    }
}
