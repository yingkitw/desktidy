use crate::types::{DuplicateGroup, FileCategory, FileEntry};
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use std::path::Path;

pub struct DisplayFormatter;

impl DisplayFormatter {
    pub fn display_summary(
        entries: &[FileEntry],
        duplicates: &[DuplicateGroup],
        actions: &[String],
        dry_run: bool,
        folder_path: &Path,
    ) {
        if dry_run {
            println!("\n[*] Analysis Mode (No files will be moved)");
        }

        // File categories summary
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Category", "Count", "Files"]);

        for category in FileCategory::order() {
            let files: Vec<_> = entries
                .iter()
                .filter(|e| e.category == category)
                .collect();

            if !files.is_empty() {
                let file_list = files
                    .iter()
                    .map(|f| {
                        f.path
                            .strip_prefix(folder_path)
                            .unwrap_or(&f.path)
                            .display()
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                table.add_row(vec![
                    category.as_str(),
                    &files.len().to_string(),
                    &file_list,
                ]);
            }
        }

        println!("\n{}", table);

        // Duplicates summary
        if !duplicates.is_empty() {
            println!("\n[!] Duplicate Files Found:");
            for dup_group in duplicates {
                println!("[!] Group {}: {} files", &dup_group.checksum_key[..8], dup_group.files.len());
                for file in &dup_group.files {
                    println!(
                        "[!]   - {}",
                        file.path
                            .strip_prefix(folder_path)
                            .unwrap_or(&file.path)
                            .display()
                    );
                }
            }
        }

        // Actions summary
        if !actions.is_empty() {
            let action_label = if dry_run {
                "Proposed Actions"
            } else {
                "Actions Taken"
            };
            println!("\n[+] {}:", action_label);
            for action in actions {
                println!("[+] ✓ {}", action);
            }
        } else {
            println!("\n[~] No files found to organize.");
        }
    }
}
