# DeskTidy

A fast, reliable command-line tool written in Rust to organize files into categorized folders.

## Features

- Organizes files into categorized folders:
  - Office Documents (DOC, DOCX, PPT, PPTX, XLS, XLSX)
  - PDFs
  - Images (JPG, PNG, GIF, BMP, TIFF, WEBP, HEIC, RAW, CR2, NEF, ARW)
  - Videos (MP4, MOV, AVI, MKV, WMV, FLV, WEBM, M4V, 3GP)
  - Audio (MP3, WAV, AAC, OGG, FLAC, M4A, WMA, AIFF)
- Detects and groups duplicate files in a separate folder
- Never deletes any files
- Provides a detailed summary of actions taken
- Analysis mode to preview changes without moving files
- Verbose mode for detailed progress information

## Installation

### From Source

1. Clone this repository
2. Build with Rust:
```bash
cargo build --release
```

The binary will be at `target/release/desktidy`

## Usage

### Organize Files
To organize files in a folder:
```bash
desktidy <folder_path>
```

### Analyze Only
To analyze files without moving them (dry run):
```bash
desktidy --analyze <folder_path>
```

### Verbose Output
Add the `-v` or `--verbose` flag for detailed progress:
```bash
desktidy --analyze -v <folder_path>
```

### Examples
```bash
# Organize files
desktidy ~/Documents

# Just analyze without moving files
desktidy --analyze ~/Documents

# Analyze with detailed progress
desktidy --analyze -v ~/Documents
```

## How It Works

1. Scans the specified folder for supported file types
2. Creates category folders for each file type found
3. Moves files into their respective category folders
4. Identifies duplicate files by comparing file size and checksums (MD5 + SHA256)
5. Moves duplicate files to a `Duplicates` folder, keeping the oldest copy
6. Displays a summary of all actions taken
