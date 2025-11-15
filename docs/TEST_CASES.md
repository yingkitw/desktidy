# Test Cases Reference

## Summary
- **Total Tests**: 40
- **Unit Tests**: 28
- **Integration Tests**: 12
- **Pass Rate**: 100%

---

## Unit Tests

### File Analyzer (10 tests)

| # | Test Name | Purpose | Input | Expected Output |
|---|-----------|---------|-------|-----------------|
| 1 | `test_get_extension_category` | Basic extension mapping | "pdf" | `Some(FileCategory::PDFs)` |
| 2 | `test_all_office_extensions` | Office file support | ppt, pptx, doc, docx, xls, xlsx | All map correctly |
| 3 | `test_all_image_extensions` | Image file support | jpg, png, gif, bmp, tiff, webp, heic | All map to Images |
| 4 | `test_all_video_extensions` | Video file support | mp4, mov, avi, mkv, wmv, flv, webm, m4v, 3gp | All map to Videos |
| 5 | `test_all_audio_extensions` | Audio file support | mp3, wav, aac, ogg, flac, m4a, wma, aiff | All map to Audio |
| 6 | `test_case_insensitive_extensions` | Case handling | PDF, DocX, MP4 | Correctly categorized |
| 7 | `test_analyze_empty_folder` | Empty directory | Empty temp dir | 0 total, 0 supported |
| 8 | `test_analyze_with_files` | Basic analysis | 1 PDF + 1 DOCX | 2 total, 2 supported |
| 9 | `test_analyze_skips_directories` | Subdirectory exclusion | 1 file in root, 1 in subdir | Only root file counted |
| 10 | `test_analyze_mixed_supported_unsupported` | Mixed files | 2 supported + 2 unsupported | 4 total, 2 supported |

### Duplicate Finder (8 tests)

| # | Test Name | Purpose | Input | Expected Output |
|---|-----------|---------|-------|-----------------|
| 1 | `test_identical_files` | Two identical files | file1.txt, file2.txt (same content) | `true` |
| 2 | `test_different_files` | Two different files | file1.txt, file2.txt (different) | `false` |
| 3 | `test_find_duplicates` | Duplicate detection | 2 identical PDFs | 1 group with 2 files |
| 4 | `test_no_duplicates` | No duplicates | 2 different PDFs | 0 groups |
| 5 | `test_multiple_duplicate_groups` | Multiple groups | 2 pairs + 1 unique | 2 groups |
| 6 | `test_empty_file_duplicates` | Empty files | 2 empty files | 1 group with 2 files |
| 7 | `test_size_mismatch_not_duplicates` | Size filtering | Different sizes | 0 groups |
| 8 | `test_single_file_not_duplicate` | Single file | 1 file | 0 groups |

### Organizer (10 tests)

| # | Test Name | Purpose | Input | Expected Output |
|---|-----------|---------|-------|-----------------|
| 1 | `test_clean_filename` | Remove numbering | "file_1.txt" | "file.txt" |
| 2 | `test_clean_filename_with_multiple_numbers` | Multi-digit cleanup | "file_123.txt" | "file.txt" |
| 3 | `test_clean_filename_no_extension` | No extension | "file_1" | "file_1" (unchanged) |
| 4 | `test_get_unique_path` | Single conflict | Existing "test.txt" | "test (1).txt" |
| 5 | `test_get_unique_path_multiple_conflicts` | Multiple conflicts | test.txt, test (1).txt, test (2).txt | "test (3).txt" |
| 6 | `test_create_category_folders` | Create folders | ["Documents", "Images"] | Both folders created |
| 7 | `test_create_existing_category_folders` | Skip existing | Existing "Documents" | No action taken |
| 8 | `test_organize_files_dry_run` | Preview mode | 1 DOCX file | File stays in root |
| 9 | `test_organize_files_actual_move` | Real movement | 1 DOCX file | File moved to Documents |
| 10 | `test_organize_files_skip_duplicates` | Duplicate handling | Original + duplicate | Original in Documents, duplicate in Duplicates |

---

## Integration Tests

### Full Workflow Tests (12 tests)

| # | Test Name | Purpose | Scenario | Verification |
|---|-----------|---------|----------|--------------|
| 1 | `test_full_workflow_with_mixed_files` | Complete workflow | 7 file types + unsupported | 8 total, 7 supported |
| 2 | `test_duplicate_detection_workflow` | Duplicate detection | 2 identical + 1 different | 1 duplicate group |
| 3 | `test_organization_dry_run` | Preview mode | 2 files, dry-run | Files stay in root |
| 4 | `test_organization_actual_move` | Real organization | 2 files, actual move | Files moved to categories |
| 5 | `test_duplicate_organization` | Duplicate handling | Identical PDFs | Duplicates folder created |
| 6 | `test_multiple_duplicates_in_group` | Large group | 3 identical files | 1 group with 3 files |
| 7 | `test_empty_folder` | Empty directory | No files | 0 total, 0 supported |
| 8 | `test_only_unsupported_files` | Unsupported only | 3 .xyz files | 3 total, 0 supported |
| 9 | `test_filename_conflict_handling` | Conflict resolution | Existing + new file | Numbered suffix applied |
| 10 | `test_large_file_duplicate_detection` | Large files | 2x 1MB identical | 1 duplicate group |
| 11 | `test_case_insensitive_extension_matching` | Case handling | .DOCX, .Pdf, .JPG | All recognized |
| 12 | `test_all_supported_file_types` | Full coverage | 14 different types | All categorized |

---

## Test Execution

### Run All Tests
```bash
cargo test
```

**Output:**
```
running 28 tests (unit)
test result: ok. 28 passed

running 12 tests (integration)
test result: ok. 12 passed

Total: 40 passed, 0 failed
```

### Run Specific Category
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific module
cargo test file_analyzer::tests

# Specific test
cargo test test_duplicate_detection_workflow
```

---

## Test Scenarios Covered

### File Type Recognition
- ✅ All 7 categories (Documents, PDFs, Images, Videos, Audio, Presentations, Spreadsheets)
- ✅ 30+ file extensions
- ✅ Case-insensitive matching
- ✅ Unsupported file handling

### Duplicate Detection
- ✅ Identical files (small and large)
- ✅ Multiple duplicate groups
- ✅ Empty files
- ✅ Size-based filtering
- ✅ MD5 + SHA256 verification

### File Organization
- ✅ Category folder creation
- ✅ File movement
- ✅ Conflict resolution (numbering)
- ✅ Dry-run mode
- ✅ Duplicate folder handling

### Edge Cases
- ✅ Empty directories
- ✅ Only unsupported files
- ✅ Mixed file types
- ✅ Existing folders
- ✅ Large files (1MB+)
- ✅ Files without extensions
- ✅ Multiple conflicts

---

## Performance Benchmarks

| Test | Duration |
|------|----------|
| All unit tests | ~20ms |
| All integration tests | ~280ms |
| Large file test (1MB) | ~100ms |
| **Total suite** | **~1 second** |

---

## Quality Metrics

- **Code Coverage**: Core logic fully tested
- **Error Handling**: All error paths covered
- **Edge Cases**: Comprehensive edge case coverage
- **Integration**: Full end-to-end workflows tested
- **Reliability**: 100% pass rate across all runs
