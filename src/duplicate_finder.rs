use crate::types::{DuplicateGroup, FileEntry};
use anyhow::Result;
use md5;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct DuplicateFinder {
    verbose: bool,
}

impl DuplicateFinder {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn calculate_checksums(file_path: &Path) -> Result<(String, String)> {
        let mut file = File::open(file_path)?;
        let mut buffer = [0; 8192];
        let mut md5_hash = md5::Context::new();
        let mut sha256_hash = Sha256::new();

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            md5_hash.consume(&buffer[..bytes_read]);
            sha256_hash.update(&buffer[..bytes_read]);
        }

        let md5_digest = format!("{:x}", md5_hash.compute());
        let sha256_digest = format!("{:x}", sha256_hash.finalize());

        Ok((md5_digest, sha256_digest))
    }

    fn get_file_size(file_path: &Path) -> Result<u64> {
        Ok(std::fs::metadata(file_path)?.len())
    }

    pub fn are_files_identical(file1: &Path, file2: &Path) -> Result<bool> {
        // First compare sizes (fast)
        let size1 = Self::get_file_size(file1)?;
        let size2 = Self::get_file_size(file2)?;

        if size1 != size2 {
            return Ok(false);
        }

        // Then compare checksums (thorough)
        let checksums1 = Self::calculate_checksums(file1)?;
        let checksums2 = Self::calculate_checksums(file2)?;

        Ok(checksums1 == checksums2)
    }

    pub fn find_duplicates(&self, entries: &[FileEntry]) -> Result<Vec<DuplicateGroup>> {
        if self.verbose {
            println!("\n[*] Checking for duplicates...");
        }

        let mut file_checksums: HashMap<String, Vec<FileEntry>> = HashMap::new();

        // First pass: collect all files and their checksums
        for entry in entries {
            match Self::calculate_checksums(&entry.path) {
                Ok((md5, sha256)) => {
                    let checksum_key = format!("{}_{}", md5, sha256);
                    file_checksums
                        .entry(checksum_key)
                        .or_insert_with(Vec::new)
                        .push(entry.clone());
                }
                Err(e) => {
                    if self.verbose {
                        eprintln!(
                            "[-] Error reading file {}: {}",
                            entry.path.display(),
                            e
                        );
                    }
                }
            }
        }

        // Second pass: identify duplicates
        let mut duplicates = Vec::new();

        for (checksum_key, file_entries) in file_checksums {
            if file_entries.len() > 1 {
                // Verify files are actually identical
                let base_file = &file_entries[0];
                let mut identical_files = vec![base_file.clone()];

                for file_entry in &file_entries[1..] {
                    if Self::are_files_identical(&base_file.path, &file_entry.path)? {
                        identical_files.push(file_entry.clone());
                    }
                }

                if identical_files.len() > 1 {
                    // Sort by creation time (oldest first)
                    identical_files.sort_by_key(|e| {
                        std::fs::metadata(&e.path)
                            .ok()
                            .and_then(|m| m.created().ok())
                            .unwrap_or_else(std::time::SystemTime::now)
                    });

                    if self.verbose {
                        println!(
                            "[!] Found duplicates: Keeping {}",
                            identical_files[0].path.display()
                        );
                        for entry in &identical_files[1..] {
                            println!("[!]   - Will move: {}", entry.path.display());
                        }
                    }

                    duplicates.push(DuplicateGroup {
                        checksum_key,
                        files: identical_files,
                    });
                }
            }
        }

        Ok(duplicates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_identical_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"test content")?;

        let mut f2 = File::create(&file2)?;
        f2.write_all(b"test content")?;

        assert!(DuplicateFinder::are_files_identical(&file1, &file2)?);

        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"content1")?;

        let mut f2 = File::create(&file2)?;
        f2.write_all(b"content2")?;

        assert!(!DuplicateFinder::are_files_identical(&file1, &file2)?);

        Ok(())
    }

    #[test]
    fn test_find_duplicates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"duplicate content")?;

        let mut f2 = File::create(&file2)?;
        f2.write_all(b"duplicate content")?;

        let entries = vec![
            FileEntry {
                path: file1,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file2,
                category: crate::types::FileCategory::Documents,
            },
        ];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].files.len(), 2);

        Ok(())
    }

    #[test]
    fn test_no_duplicates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"content1")?;

        let mut f2 = File::create(&file2)?;
        f2.write_all(b"content2")?;

        let entries = vec![
            FileEntry {
                path: file1,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file2,
                category: crate::types::FileCategory::Documents,
            },
        ];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 0);

        Ok(())
    }

    #[test]
    fn test_multiple_duplicate_groups() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Group 1: identical files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let mut f1 = File::create(&file1)?;
        f1.write_all(b"group1")?;
        let mut f2 = File::create(&file2)?;
        f2.write_all(b"group1")?;

        // Group 2: different identical files
        let file3 = temp_dir.path().join("file3.txt");
        let file4 = temp_dir.path().join("file4.txt");
        let mut f3 = File::create(&file3)?;
        f3.write_all(b"group2")?;
        let mut f4 = File::create(&file4)?;
        f4.write_all(b"group2")?;

        // Unique file
        let file5 = temp_dir.path().join("file5.txt");
        let mut f5 = File::create(&file5)?;
        f5.write_all(b"unique")?;

        let entries = vec![
            FileEntry {
                path: file1,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file2,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file3,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file4,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file5,
                category: crate::types::FileCategory::Documents,
            },
        ];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 2);
        assert_eq!(duplicates[0].files.len(), 2);
        assert_eq!(duplicates[1].files.len(), 2);

        Ok(())
    }

    #[test]
    fn test_empty_file_duplicates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("empty1.txt");
        let file2 = temp_dir.path().join("empty2.txt");

        File::create(&file1)?;
        File::create(&file2)?;

        let entries = vec![
            FileEntry {
                path: file1,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file2,
                category: crate::types::FileCategory::Documents,
            },
        ];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].files.len(), 2);

        Ok(())
    }

    #[test]
    fn test_size_mismatch_not_duplicates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"short")?;

        let mut f2 = File::create(&file2)?;
        f2.write_all(b"this is a much longer content")?;

        let entries = vec![
            FileEntry {
                path: file1,
                category: crate::types::FileCategory::Documents,
            },
            FileEntry {
                path: file2,
                category: crate::types::FileCategory::Documents,
            },
        ];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 0);

        Ok(())
    }

    #[test]
    fn test_single_file_not_duplicate() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");

        let mut f1 = File::create(&file1)?;
        f1.write_all(b"unique content")?;

        let entries = vec![FileEntry {
            path: file1,
            category: crate::types::FileCategory::Documents,
        }];

        let finder = DuplicateFinder::new(false);
        let duplicates = finder.find_duplicates(&entries)?;

        assert_eq!(duplicates.len(), 0);

        Ok(())
    }
}
