# Architecture

## Overview

DeskTidy is a Rust CLI application for organizing files into categorized folders. The architecture follows a modular, trait-friendly design with clear separation of concerns.

## Project Structure

```
src/
├── main.rs              # CLI entry point with clap argument parsing
├── lib.rs               # Library root, exports public modules
├── types.rs             # Core data types (FileCategory, FileEntry, etc.)
├── file_analyzer.rs     # File scanning and categorization logic
├── duplicate_finder.rs  # Duplicate detection using checksums
├── organizer.rs         # File movement and organization logic
└── display.rs           # Output formatting with comfy-table
```

## Core Modules

### `types.rs`
Defines the fundamental data structures:
- **FileCategory**: Enum representing file types (Documents, PDFs, Images, Videos, Audio)
- **FileEntry**: Represents a single file with its path and category
- **DuplicateGroup**: Groups of identical files with their checksum key
- **AnalysisResult**: Result of file analysis containing categorized entries
- **OrganizationSummary**: Summary of actions taken during organization

### `file_analyzer.rs`
Responsible for scanning directories and categorizing files:
- Reads files from the target folder (root level only)
- Maps file extensions to categories
- Collects statistics (total files, supported files)
- Supports verbose logging

**Key Methods:**
- `analyze()`: Main entry point, returns AnalysisResult
- `get_extension_category()`: Maps file extensions to categories

### `duplicate_finder.rs`
Detects duplicate files using content-based comparison:
- Calculates MD5 and SHA256 checksums for each file
- Compares file sizes first (fast path)
- Verifies duplicates using both checksums
- Sorts duplicates by creation time (keeps oldest)

**Key Methods:**
- `find_duplicates()`: Identifies all duplicate groups
- `are_files_identical()`: Compares two files for identity
- `calculate_checksums()`: Computes MD5 and SHA256 hashes

### `organizer.rs`
Handles file movement and organization:
- Creates category folders as needed
- Moves files to their respective category folders
- Handles filename conflicts with numbering scheme
- Moves duplicates to a `Duplicates` folder
- Supports dry-run mode for preview

**Key Methods:**
- `organize_files()`: Main organization logic
- `get_unique_path()`: Generates unique paths for conflicting files
- `clean_filename()`: Removes old-style numbering from filenames
- `safe_move()`: Safely moves files with error handling

### `display.rs`
Formats and displays results:
- Renders file summary table using comfy-table
- Lists duplicate groups
- Shows actions taken or proposed
- Distinguishes between dry-run and actual organization

### `main.rs`
CLI interface using clap:
- Parses command-line arguments
- Orchestrates the workflow: analyze → find duplicates → organize
- Handles both dry-run and actual organization modes
- Supports verbose logging

## Data Flow

```
main.rs
  ↓
FileAnalyzer::analyze()
  ↓ (AnalysisResult)
DuplicateFinder::find_duplicates()
  ↓ (Vec<DuplicateGroup>)
Organizer::organize_files()
  ↓ (OrganizationSummary)
DisplayFormatter::display_summary()
  ↓
Console output
```

## Design Principles

- **Modularity**: Each module has a single responsibility
- **Testability**: All core logic is unit tested with tempfile for isolation
- **Error Handling**: Uses `anyhow::Result` for ergonomic error propagation
- **Immutability**: Prefers immutable data structures where possible
- **Trait-Friendly**: Designed to support future trait-based extensions

## Dependencies

- **clap**: Command-line argument parsing with derive macros
- **anyhow**: Ergonomic error handling
- **comfy-table**: ASCII table formatting for display
- **md5**: MD5 hashing for duplicate detection
- **sha2**: SHA256 hashing for duplicate detection
- **regex**: Pattern matching for filename cleaning
- **tempfile**: Testing utilities (dev-dependency)
- **insta**: Snapshot testing (dev-dependency)

## Testing

All core modules include unit tests:
- `file_analyzer::tests`: Tests extension mapping and file analysis
- `duplicate_finder::tests`: Tests checksum calculation and duplicate detection
- `organizer::tests`: Tests filename cleaning and unique path generation

Run tests with:
```bash
cargo test
```

## Future Extensibility

The architecture supports:
- Custom file extension mappings via configuration
- Trait-based organizer implementations for different strategies
- Integration with external services (e.g., watsonx for AI categorization)
- Parallel processing for large directories
- Plugin system for custom file handlers
