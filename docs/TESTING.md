# Testing Guide

## Overview

DeskTidy has a comprehensive test suite with **40 tests** covering all core functionality:
- **28 unit tests** in `src/` modules
- **12 integration tests** in `tests/integration_tests.rs`

All tests pass with 100% success rate.

## Running Tests

### Run all tests
```bash
cargo test
```

### Run only unit tests
```bash
cargo test --lib
```

### Run only integration tests
```bash
cargo test --test integration_tests
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run a specific test
```bash
cargo test test_duplicate_detection_workflow
```

## Unit Tests

### File Analyzer (`src/file_analyzer.rs`)

**Extension Mapping Tests:**
- `test_get_extension_category` - Basic extension categorization
- `test_all_office_extensions` - PPT, PPTX, DOC, DOCX, XLS, XLSX
- `test_all_image_extensions` - JPG, PNG, GIF, BMP, TIFF, WEBP, HEIC
- `test_all_video_extensions` - MP4, MOV, AVI, MKV, WMV, FLV, WEBM, M4V, 3GP
- `test_all_audio_extensions` - MP3, WAV, AAC, OGG, FLAC, M4A, WMA, AIFF
- `test_case_insensitive_extensions` - Uppercase/mixed case handling

**File Analysis Tests:**
- `test_analyze_empty_folder` - Empty directory handling
- `test_analyze_with_files` - Basic file analysis
- `test_analyze_skips_directories` - Subdirectory exclusion
- `test_analyze_mixed_supported_unsupported` - Mixed file type handling

### Duplicate Finder (`src/duplicate_finder.rs`)

**Duplicate Detection Tests:**
- `test_identical_files` - Two identical files
- `test_different_files` - Two different files
- `test_find_duplicates` - Basic duplicate group detection
- `test_no_duplicates` - No duplicates in set
- `test_multiple_duplicate_groups` - Multiple separate groups
- `test_empty_file_duplicates` - Empty file handling
- `test_size_mismatch_not_duplicates` - Size-based filtering
- `test_single_file_not_duplicate` - Single file handling

### Organizer (`src/organizer.rs`)

**Filename Handling Tests:**
- `test_clean_filename` - Remove trailing numbers
- `test_clean_filename_with_multiple_numbers` - Multi-digit cleanup
- `test_clean_filename_no_extension` - Files without extensions
- `test_get_unique_path` - Conflict resolution
- `test_get_unique_path_multiple_conflicts` - Multiple conflicts

**Folder Operations Tests:**
- `test_create_category_folders` - Create new folders
- `test_create_existing_category_folders` - Skip existing folders

**File Organization Tests:**
- `test_organize_files_dry_run` - Preview mode
- `test_organize_files_actual_move` - Actual file movement
- `test_organize_files_skip_duplicates` - Duplicate handling

## Integration Tests

### Full Workflow Tests (`tests/integration_tests.rs`)

**Basic Workflows:**
- `test_full_workflow_with_mixed_files` - 7 file types + unsupported
- `test_empty_folder` - Empty directory
- `test_only_unsupported_files` - No supported files

**Duplicate Detection:**
- `test_duplicate_detection_workflow` - Identify duplicates
- `test_duplicate_organization` - Move duplicates to folder
- `test_multiple_duplicates_in_group` - 3+ identical files
- `test_large_file_duplicate_detection` - 1MB files

**Organization:**
- `test_organization_dry_run` - Preview without changes
- `test_organization_actual_move` - Real file movement
- `test_filename_conflict_handling` - Numbered conflicts

**File Type Support:**
- `test_case_insensitive_extension_matching` - Mixed case extensions
- `test_all_supported_file_types` - 14 different file types

## Test Data

Tests use `tempfile::TempDir` for isolated, temporary file systems. Each test:
1. Creates a temporary directory
2. Sets up test files
3. Runs the operation
4. Verifies results
5. Automatically cleans up

## Coverage Areas

| Module | Coverage | Tests |
|--------|----------|-------|
| File Analyzer | Extension mapping, categorization, directory scanning | 10 |
| Duplicate Finder | Checksum calculation, duplicate detection, edge cases | 8 |
| Organizer | File movement, conflict resolution, dry-run | 10 |
| Integration | Full workflows, mixed scenarios | 12 |

## Adding New Tests

### Unit Test Template
```rust
#[test]
fn test_new_feature() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Setup
    File::create(temp_dir.path().join("test.pdf"))?;
    
    // Execute
    let analyzer = FileAnalyzer::new(temp_dir.path().to_path_buf(), false);
    let result = analyzer.analyze()?;
    
    // Assert
    assert_eq!(result.supported_files, 1);
    
    Ok(())
}
```

### Integration Test Template
```rust
#[test]
fn test_new_workflow() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Setup files
    File::create(temp_path.join("file.pdf"))?;
    
    // Run workflow
    let analyzer = FileAnalyzer::new(temp_path.to_path_buf(), false);
    let analysis = analyzer.analyze()?;
    
    // Verify
    assert_eq!(analysis.supported_files, 1);
    
    Ok(())
}
```

## Continuous Integration

To ensure tests pass before committing:
```bash
cargo test && cargo build --release
```

## Performance Notes

- Full test suite completes in ~1 second
- Large file tests (1MB) complete in ~0.3 seconds
- All tests use isolated temporary directories
- No file system pollution after test completion
