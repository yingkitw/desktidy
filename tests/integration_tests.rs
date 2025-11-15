use desktidy::{
    file_analyzer::FileAnalyzer, duplicate_finder::DuplicateFinder, organizer::Organizer,
};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_full_workflow_with_mixed_files() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    File::create(temp_path.join("document.docx"))?;
    File::create(temp_path.join("presentation.pptx"))?;
    File::create(temp_path.join("spreadsheet.xlsx"))?;
    File::create(temp_path.join("image.jpg"))?;
    File::create(temp_path.join("video.mp4"))?;
    File::create(temp_path.join("audio.mp3"))?;
    File::create(temp_path.join("pdf.pdf"))?;
    File::create(temp_path.join("unsupported.xyz"))?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    assert_eq!(analysis.total_files, 8);
    assert_eq!(analysis.supported_files, 7);
    assert_eq!(analysis.categories.len(), 7);

    Ok(())
}

#[test]
fn test_duplicate_detection_workflow() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create identical files (using supported extensions)
    let content = b"identical content for testing";
    let mut file1 = File::create(temp_path.join("file1.pdf"))?;
    file1.write_all(content)?;

    let mut file2 = File::create(temp_path.join("file2.pdf"))?;
    file2.write_all(content)?;

    let mut file3 = File::create(temp_path.join("file3.pdf"))?;
    file3.write_all(b"different content")?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Find duplicates
    let finder = DuplicateFinder::new(false);
    let duplicates = finder.find_duplicates(&all_entries)?;

    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].files.len(), 2);

    Ok(())
}

#[test]
fn test_organization_dry_run() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    File::create(temp_path.join("doc.docx"))?;
    File::create(temp_path.join("slide.pptx"))?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Organize in dry-run mode
    let organizer = Organizer::new(temp_path.to_path_buf(), false);
    let summary = organizer.organize_files(&all_entries, &[], true)?;

    // Verify files are still in root (dry-run)
    assert!(temp_path.join("doc.docx").exists());
    assert!(temp_path.join("slide.pptx").exists());

    // Verify proposed actions
    assert_eq!(summary.actions_taken.len(), 2);
    assert!(summary.actions_taken[0].contains("Would move"));

    Ok(())
}

#[test]
fn test_organization_actual_move() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    File::create(temp_path.join("doc.docx"))?;
    File::create(temp_path.join("slide.pptx"))?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Create category folders
    let organizer = Organizer::new(temp_path.to_path_buf(), false);
    let categories: Vec<&str> = analysis.categories.keys().map(|c| c.as_str()).collect();
    organizer.create_category_folders(&categories)?;

    // Organize (actual move)
    let summary = organizer.organize_files(&all_entries, &[], false)?;

    // Verify files were moved
    assert!(temp_path.join("Documents").join("doc.docx").exists());
    assert!(temp_path.join("Presentations").join("slide.pptx").exists());
    assert!(!temp_path.join("doc.docx").exists());
    assert!(!temp_path.join("slide.pptx").exists());

    // Verify actions
    assert_eq!(summary.actions_taken.len(), 2);

    Ok(())
}

#[test]
fn test_duplicate_organization() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create identical files (using supported extensions)
    let content = b"duplicate content";
    let mut file1 = File::create(temp_path.join("original.pdf"))?;
    file1.write_all(content)?;

    let mut file2 = File::create(temp_path.join("duplicate.pdf"))?;
    file2.write_all(content)?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Find duplicates
    let finder = DuplicateFinder::new(false);
    let duplicates = finder.find_duplicates(&all_entries)?;

    // Create category folders
    let organizer = Organizer::new(temp_path.to_path_buf(), false);
    let categories: Vec<&str> = analysis.categories.keys().map(|c| c.as_str()).collect();
    organizer.create_category_folders(&categories)?;

    // Organize with duplicates
    let _summary = organizer.organize_files(&all_entries, &duplicates, false)?;

    // Verify duplicate was moved to Duplicates folder
    assert!(temp_path.join("Duplicates").exists());

    Ok(())
}

#[test]
fn test_multiple_duplicates_in_group() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create three identical files (using supported extensions)
    let content = b"triple duplicate";
    for i in 1..=3 {
        let mut file = File::create(temp_path.join(format!("file{}.pdf", i)))?;
        file.write_all(content)?;
    }

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Find duplicates
    let finder = DuplicateFinder::new(false);
    let duplicates = finder.find_duplicates(&all_entries)?;

    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].files.len(), 3);

    Ok(())
}

#[test]
fn test_empty_folder() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    assert_eq!(analysis.total_files, 0);
    assert_eq!(analysis.supported_files, 0);
    assert!(analysis.categories.is_empty());

    Ok(())
}

#[test]
fn test_only_unsupported_files() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    File::create(temp_path.join("file1.xyz"))?;
    File::create(temp_path.join("file2.abc"))?;
    File::create(temp_path.join("file3.unknown"))?;

    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    assert_eq!(analysis.total_files, 3);
    assert_eq!(analysis.supported_files, 0);
    assert!(analysis.categories.is_empty());

    Ok(())
}

#[test]
fn test_filename_conflict_handling() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create Documents folder with existing file
    fs::create_dir(temp_path.join("Documents"))?;
    File::create(temp_path.join("Documents").join("doc.docx"))?;

    // Create another doc.docx in root
    File::create(temp_path.join("doc.docx"))?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Organize
    let organizer = Organizer::new(temp_path.to_path_buf(), false);
    let summary = organizer.organize_files(&all_entries, &[], false)?;

    // Verify conflict was handled
    assert!(temp_path.join("Documents").join("doc.docx").exists());
    assert!(temp_path.join("Documents").join("doc (1).docx").exists());

    Ok(())
}

#[test]
fn test_large_file_duplicate_detection() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create larger files (1MB, using supported extensions)
    let large_content = vec![42u8; 1024 * 1024];

    let mut file1 = File::create(temp_path.join("large1.pdf"))?;
    file1.write_all(&large_content)?;

    let mut file2 = File::create(temp_path.join("large2.pdf"))?;
    file2.write_all(&large_content)?;

    // Analyze
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Find duplicates
    let finder = DuplicateFinder::new(false);
    let duplicates = finder.find_duplicates(&all_entries)?;

    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].files.len(), 2);

    Ok(())
}

#[test]
fn test_case_insensitive_extension_matching() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create files with mixed case extensions
    File::create(temp_path.join("file.DOCX"))?;
    File::create(temp_path.join("file.Pdf"))?;
    File::create(temp_path.join("file.JPG"))?;

    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    assert_eq!(analysis.supported_files, 3);
    assert_eq!(analysis.categories.len(), 3);

    Ok(())
}

#[test]
fn test_all_supported_file_types() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Office Documents
    File::create(temp_path.join("test.ppt"))?;
    File::create(temp_path.join("test.pptx"))?;
    File::create(temp_path.join("test.doc"))?;
    File::create(temp_path.join("test.docx"))?;
    File::create(temp_path.join("test.xls"))?;
    File::create(temp_path.join("test.xlsx"))?;

    // PDFs
    File::create(temp_path.join("test.pdf"))?;

    // Images
    File::create(temp_path.join("test.jpg"))?;
    File::create(temp_path.join("test.png"))?;
    File::create(temp_path.join("test.gif"))?;

    // Videos
    File::create(temp_path.join("test.mp4"))?;
    File::create(temp_path.join("test.mov"))?;

    // Audio
    File::create(temp_path.join("test.mp3"))?;
    File::create(temp_path.join("test.wav"))?;

    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;

    assert_eq!(analysis.supported_files, 14);

    Ok(())
}
