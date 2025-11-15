use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Documents,
    PDFs,
    Presentations,
    Spreadsheets,
    Images,
    Videos,
    Audio,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::Documents => "Documents",
            FileCategory::PDFs => "PDFs",
            FileCategory::Presentations => "Presentations",
            FileCategory::Spreadsheets => "Spreadsheets",
            FileCategory::Images => "Images",
            FileCategory::Videos => "Videos",
            FileCategory::Audio => "Audio",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            FileCategory::Documents => "blue",
            FileCategory::PDFs => "red",
            FileCategory::Presentations => "magenta",
            FileCategory::Spreadsheets => "green",
            FileCategory::Images => "cyan",
            FileCategory::Videos => "yellow",
            FileCategory::Audio => "red",
        }
    }

    pub fn order() -> Vec<FileCategory> {
        vec![
            FileCategory::Documents,
            FileCategory::PDFs,
            FileCategory::Presentations,
            FileCategory::Spreadsheets,
            FileCategory::Images,
            FileCategory::Videos,
            FileCategory::Audio,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub category: FileCategory,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub checksum_key: String,
    pub files: Vec<FileEntry>,
}

pub struct AnalysisResult {
    pub total_files: usize,
    pub supported_files: usize,
    pub categories: HashMap<FileCategory, Vec<FileEntry>>,
}

pub struct OrganizationSummary {
    pub actions_taken: Vec<String>,
    pub duplicates_found: Vec<DuplicateGroup>,
}
