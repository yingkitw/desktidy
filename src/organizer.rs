use crate::types::{DuplicateGroup, FileEntry, OrganizationSummary};
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Organizer {
    folder_path: PathBuf,
    verbose: bool,
}

impl Organizer {
    pub fn new(folder_path: PathBuf, verbose: bool) -> Self {
        Self {
            folder_path: folder_path.canonicalize().unwrap_or(folder_path),
            verbose,
        }
    }

    fn clean_filename(&self, filepath: &Path) -> PathBuf {
        if let Some(file_name) = filepath.file_name().and_then(|n| n.to_str()) {
            if let Some(stem_start) = file_name.rfind('.') {
                let stem = &file_name[..stem_start];
                let ext = &file_name[stem_start..];

                // Match patterns like "_1", "_2", etc. at the end of the filename
                if let Ok(re) = Regex::new(r"_\d+$") {
                    if re.is_match(stem) {
                        let clean_stem = re.replace(stem, "").to_string();
                        return filepath.parent().unwrap().join(format!("{}{}", clean_stem, ext));
                    }
                }
            }
        }
        filepath.to_path_buf()
    }

    fn get_unique_path(&self, target_path: &Path) -> PathBuf {
        let target_path = self.clean_filename(target_path);

        if !target_path.exists() {
            return target_path;
        }

        let mut counter = 1;
        loop {
            let stem = target_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = target_path.extension().unwrap_or_default().to_string_lossy();
            let new_name = format!("{} ({}){}", stem, counter, if ext.is_empty() { String::new() } else { format!(".{}", ext) });
            let new_path = target_path.parent().unwrap().join(new_name);

            if !new_path.exists() {
                return new_path;
            }
            counter += 1;
        }
    }

    fn safe_move(&self, source: &Path, dest: &Path) -> Result<bool> {
        let source = source.canonicalize()?;
        let dest = dest.canonicalize().unwrap_or_else(|_| dest.to_path_buf());

        // Create parent directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&source, &dest)?;
        Ok(true)
    }

    pub fn create_category_folders(&self, categories: &[&str]) -> Result<Vec<String>> {
        let mut actions = Vec::new();

        for category in categories {
            let category_folder = self.folder_path.join(category);
            if !category_folder.exists() {
                fs::create_dir_all(&category_folder)?;
                actions.push(format!("Created category folder: {}", category));
            }
        }

        Ok(actions)
    }

    pub fn organize_files(
        &self,
        entries: &[FileEntry],
        duplicates: &[DuplicateGroup],
        dry_run: bool,
    ) -> Result<OrganizationSummary> {
        let mut actions_taken = Vec::new();

        // Create a set of files to skip (duplicates that will be moved)
        let mut files_to_skip = std::collections::HashSet::new();
        for dup_group in duplicates {
            for entry in &dup_group.files[1..] {
                files_to_skip.insert(entry.path.clone());
            }
        }

        // Move files to category folders
        for entry in entries {
            if files_to_skip.contains(&entry.path) {
                continue;
            }

            let category_folder = self.folder_path.join(entry.category.as_str());
            if entry.path.parent() != Some(&category_folder) {
                let new_path = self.get_unique_path(&category_folder.join(entry.path.file_name().unwrap()));

                if !dry_run {
                    if let Ok(true) = self.safe_move(&entry.path, &new_path) {
                        actions_taken.push(format!(
                            "Moved {} to {} folder",
                            entry.path.file_name().unwrap_or_default().to_string_lossy(),
                            entry.category.as_str()
                        ));
                    }
                } else {
                    actions_taken.push(format!(
                        "Would move {} to {} folder",
                        entry.path.file_name().unwrap_or_default().to_string_lossy(),
                        entry.category.as_str()
                    ));
                }
            }
        }

        // Organize duplicates
        if !duplicates.is_empty() {
            let dup_folder = self.folder_path.join("Duplicates");
            if !dry_run {
                fs::create_dir_all(&dup_folder)?;
            }

            for dup_group in duplicates {
                for entry in &dup_group.files[1..] {
                    if entry.path.parent() != Some(&dup_folder) {
                        let new_path = self.get_unique_path(&dup_folder.join(entry.path.file_name().unwrap()));

                        if !dry_run {
                            if let Ok(true) = self.safe_move(&entry.path, &new_path) {
                                let original = &dup_group.files[0];
                                actions_taken.push(format!(
                                    "Moved duplicate {} to Duplicates folder (identical to {})",
                                    entry.path.file_name().unwrap_or_default().to_string_lossy(),
                                    original.path.file_name().unwrap_or_default().to_string_lossy()
                                ));
                            }
                        } else {
                            let original = &dup_group.files[0];
                            actions_taken.push(format!(
                                "Would move duplicate {} to Duplicates folder (identical to {})",
                                entry.path.file_name().unwrap_or_default().to_string_lossy(),
                                original.path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }

        Ok(OrganizationSummary {
            actions_taken,
            duplicates_found: duplicates.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_clean_filename() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let path = PathBuf::from("/path/to/file_1.txt");
        let cleaned = organizer.clean_filename(&path);
        assert_eq!(cleaned.file_name().unwrap(), "file.txt");

        let path2 = PathBuf::from("/path/to/file.txt");
        let cleaned2 = organizer.clean_filename(&path2);
        assert_eq!(cleaned2.file_name().unwrap(), "file.txt");

        Ok(())
    }

    #[test]
    fn test_get_unique_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let file1 = temp_dir.path().join("test.txt");
        File::create(&file1)?;

        let unique_path = organizer.get_unique_path(&file1);
        assert_eq!(unique_path.file_name().unwrap(), "test (1).txt");

        Ok(())
    }

    #[test]
    fn test_get_unique_path_multiple_conflicts() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let file1 = temp_dir.path().join("test.txt");
        let file2 = temp_dir.path().join("test (1).txt");
        let file3 = temp_dir.path().join("test (2).txt");

        File::create(&file1)?;
        File::create(&file2)?;
        File::create(&file3)?;

        let unique_path = organizer.get_unique_path(&file1);
        assert_eq!(unique_path.file_name().unwrap(), "test (3).txt");

        Ok(())
    }

    #[test]
    fn test_clean_filename_with_multiple_numbers() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let path = PathBuf::from("/path/to/file_123.txt");
        let cleaned = organizer.clean_filename(&path);
        assert_eq!(cleaned.file_name().unwrap(), "file.txt");

        Ok(())
    }

    #[test]
    fn test_clean_filename_no_extension() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let path = PathBuf::from("/path/to/file_1");
        let cleaned = organizer.clean_filename(&path);
        // Files without extensions won't match the regex pattern, so they stay unchanged
        assert_eq!(cleaned.file_name().unwrap(), "file_1");

        Ok(())
    }

    #[test]
    fn test_create_category_folders() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        let categories = vec!["Documents", "Images", "Videos"];
        let actions = organizer.create_category_folders(&categories)?;

        assert_eq!(actions.len(), 3);
        assert!(temp_dir.path().join("Documents").exists());
        assert!(temp_dir.path().join("Images").exists());
        assert!(temp_dir.path().join("Videos").exists());

        Ok(())
    }

    #[test]
    fn test_create_existing_category_folders() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);

        // Create folder first
        fs::create_dir(temp_dir.path().join("Documents"))?;

        let categories = vec!["Documents"];
        let actions = organizer.create_category_folders(&categories)?;

        // Should not create again, so no actions
        assert_eq!(actions.len(), 0);

        Ok(())
    }

    #[test]
    fn test_organize_files_dry_run() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("doc.docx");
        File::create(&file1)?;

        let entry = FileEntry {
            path: file1.clone(),
            category: crate::types::FileCategory::Documents,
        };

        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);
        let summary = organizer.organize_files(&[entry], &[], true)?;

        // File should still be in root (dry-run)
        assert!(file1.exists());
        assert!(!temp_dir.path().join("Documents").exists());
        assert_eq!(summary.actions_taken.len(), 1);
        assert!(summary.actions_taken[0].contains("Would move"));

        Ok(())
    }

    #[test]
    fn test_organize_files_actual_move() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("doc.docx");
        File::create(&file1)?;

        let entry = FileEntry {
            path: file1.clone(),
            category: crate::types::FileCategory::Documents,
        };

        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);
        fs::create_dir(temp_dir.path().join("Documents"))?;
        let summary = organizer.organize_files(&[entry], &[], false)?;

        // File should be moved
        assert!(!file1.exists());
        assert!(temp_dir.path().join("Documents").join("doc.docx").exists());
        assert_eq!(summary.actions_taken.len(), 1);
        assert!(summary.actions_taken[0].contains("Moved"));

        Ok(())
    }

    #[test]
    fn test_organize_files_skip_duplicates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("original.txt");
        let file2 = temp_dir.path().join("duplicate.txt");
        File::create(&file1)?;
        File::create(&file2)?;

        let entry1 = FileEntry {
            path: file1.clone(),
            category: crate::types::FileCategory::Documents,
        };
        let entry2 = FileEntry {
            path: file2.clone(),
            category: crate::types::FileCategory::Documents,
        };

        let dup_group = crate::types::DuplicateGroup {
            checksum_key: "test_key".to_string(),
            files: vec![entry1.clone(), entry2.clone()],
        };

        let organizer = Organizer::new(temp_dir.path().to_path_buf(), false);
        fs::create_dir(temp_dir.path().join("Documents"))?;
        let summary = organizer.organize_files(&[entry1, entry2], &[dup_group], false)?;

        // Only original should be in Documents, duplicate in Duplicates
        assert!(temp_dir.path().join("Documents").join("original.txt").exists());
        assert!(temp_dir.path().join("Duplicates").exists());

        Ok(())
    }
}
