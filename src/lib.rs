pub mod file_analyzer;
pub mod duplicate_finder;
pub mod organizer;
pub mod types;
pub mod display;

pub use file_analyzer::FileAnalyzer;
pub use duplicate_finder::DuplicateFinder;
pub use organizer::Organizer;
pub use types::{FileCategory, FileEntry, DuplicateGroup};
