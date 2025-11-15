use crate::types::{FileCategory, FileEntry, AnalysisResult};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct FileAnalyzer {
    folder_path: PathBuf,
    verbose: bool,
}

impl FileAnalyzer {
    pub fn new(folder_path: PathBuf, verbose: bool) -> Self {
        Self {
            folder_path: folder_path.canonicalize().unwrap_or(folder_path),
            verbose,
        }
    }

    fn get_extension_category(ext: &str) -> Option<FileCategory> {
        match ext.to_lowercase().as_str() {
            // Office Documents
            "ppt" | "pptx" => Some(FileCategory::Presentations),
            "doc" | "docx" => Some(FileCategory::Documents),
            "xls" | "xlsx" => Some(FileCategory::Spreadsheets),
            // PDFs
            "pdf" => Some(FileCategory::PDFs),
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "heic" | "raw"
            | "cr2" | "nef" | "arw" => Some(FileCategory::Images),
            // Videos
            "mp4" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "webm" | "m4v" | "3gp" => {
                Some(FileCategory::Videos)
            }
            // Audio
            "mp3" | "wav" | "aac" | "ogg" | "flac" | "m4a" | "wma" | "aiff" => {
                Some(FileCategory::Audio)
            }
            _ => None,
        }
    }

    pub fn analyze(&self) -> Result<AnalysisResult> {
        if self.verbose {
            println!("\n[*] Starting file analysis...");
        }

        let mut categories: HashMap<FileCategory, Vec<FileEntry>> = HashMap::new();
        let mut total_files = 0;
        let mut supported_files = 0;

        let duplicates_dir = self.folder_path.join("Duplicates");

        for entry in fs::read_dir(&self.folder_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip directories and Duplicates folder
            if path.is_dir() || path == duplicates_dir {
                if self.verbose {
                    println!("[~] Skipping folder: {}", path.display());
                }
                continue;
            }

            total_files += 1;

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Some(category) = Self::get_extension_category(ext) {
                    supported_files += 1;
                    if self.verbose {
                        println!(
                            "[+] Found {} file: {}",
                            category.as_str(),
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    }
                    categories.entry(category.clone()).or_insert_with(Vec::new).push(FileEntry {
                        path,
                        category,
                    });
                }
            }
        }

        if self.verbose {
            println!("\n[*] Analysis Summary:");
            println!("[*] Total files scanned: {}", total_files);
            println!("[*] Supported files found: {}", supported_files);
            for category in FileCategory::order() {
                if let Some(files) = categories.get(&category) {
                    println!("[*] {}: {} files", category.as_str(), files.len());
                }
            }
        }

        Ok(AnalysisResult {
            total_files,
            supported_files,
            categories,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_get_extension_category() {
        assert_eq!(
            FileAnalyzer::get_extension_category("pdf"),
            Some(FileCategory::PDFs)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("docx"),
            Some(FileCategory::Documents)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("mp4"),
            Some(FileCategory::Videos)
        );
        assert_eq!(FileAnalyzer::get_extension_category("unknown"), None);
    }

    #[test]
    fn test_analyze_empty_folder() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let analyzer = FileAnalyzer::new(temp_dir.path().to_path_buf(), false);
        let result = analyzer.analyze()?;

        assert_eq!(result.total_files, 0);
        assert_eq!(result.supported_files, 0);
        assert!(result.categories.is_empty());

        Ok(())
    }

    #[test]
    fn test_analyze_with_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pdf_path = temp_dir.path().join("test.pdf");
        let doc_path = temp_dir.path().join("test.docx");

        File::create(pdf_path)?;
        File::create(doc_path)?;

        let analyzer = FileAnalyzer::new(temp_dir.path().to_path_buf(), false);
        let result = analyzer.analyze()?;

        assert_eq!(result.total_files, 2);
        assert_eq!(result.supported_files, 2);
        assert_eq!(result.categories.len(), 2);

        Ok(())
    }

    #[test]
    fn test_all_office_extensions() -> Result<()> {
        assert_eq!(
            FileAnalyzer::get_extension_category("ppt"),
            Some(FileCategory::Presentations)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("pptx"),
            Some(FileCategory::Presentations)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("doc"),
            Some(FileCategory::Documents)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("docx"),
            Some(FileCategory::Documents)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("xls"),
            Some(FileCategory::Spreadsheets)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("xlsx"),
            Some(FileCategory::Spreadsheets)
        );
        Ok(())
    }

    #[test]
    fn test_all_image_extensions() -> Result<()> {
        let image_exts = vec!["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic"];
        for ext in image_exts {
            assert_eq!(
                FileAnalyzer::get_extension_category(ext),
                Some(FileCategory::Images),
                "Failed for extension: {}",
                ext
            );
        }
        Ok(())
    }

    #[test]
    fn test_all_video_extensions() -> Result<()> {
        let video_exts = vec!["mp4", "mov", "avi", "mkv", "wmv", "flv", "webm", "m4v", "3gp"];
        for ext in video_exts {
            assert_eq!(
                FileAnalyzer::get_extension_category(ext),
                Some(FileCategory::Videos),
                "Failed for extension: {}",
                ext
            );
        }
        Ok(())
    }

    #[test]
    fn test_all_audio_extensions() -> Result<()> {
        let audio_exts = vec!["mp3", "wav", "aac", "ogg", "flac", "m4a", "wma", "aiff"];
        for ext in audio_exts {
            assert_eq!(
                FileAnalyzer::get_extension_category(ext),
                Some(FileCategory::Audio),
                "Failed for extension: {}",
                ext
            );
        }
        Ok(())
    }

    #[test]
    fn test_case_insensitive_extensions() -> Result<()> {
        assert_eq!(
            FileAnalyzer::get_extension_category("PDF"),
            Some(FileCategory::PDFs)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("DocX"),
            Some(FileCategory::Documents)
        );
        assert_eq!(
            FileAnalyzer::get_extension_category("MP4"),
            Some(FileCategory::Videos)
        );
        Ok(())
    }

    #[test]
    fn test_analyze_skips_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir)?;
        File::create(subdir.join("test.pdf"))?;
        File::create(temp_dir.path().join("test.docx"))?;

        let analyzer = FileAnalyzer::new(temp_dir.path().to_path_buf(), false);
        let result = analyzer.analyze()?;

        // Should only count the file in root, not in subdirectory
        assert_eq!(result.total_files, 1);
        assert_eq!(result.supported_files, 1);

        Ok(())
    }

    #[test]
    fn test_analyze_mixed_supported_unsupported() -> Result<()> {
        let temp_dir = TempDir::new()?;
        File::create(temp_dir.path().join("doc.docx"))?;
        File::create(temp_dir.path().join("image.jpg"))?;
        File::create(temp_dir.path().join("random.xyz"))?;
        File::create(temp_dir.path().join("data.bin"))?;

        let analyzer = FileAnalyzer::new(temp_dir.path().to_path_buf(), false);
        let result = analyzer.analyze()?;

        assert_eq!(result.total_files, 4);
        assert_eq!(result.supported_files, 2);

        Ok(())
    }
}
