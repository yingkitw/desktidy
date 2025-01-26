# DeskTidy

A command-line tool to organize Microsoft Office documents in a folder.

## Features

- Organizes Microsoft Office files into categorized folders:
  - Presentations (PPT, PPTX)
  - Documents (DOC, DOCX)
  - Spreadsheets (XLS, XLSX)
- Groups duplicate files in a separate folder
- Never deletes any files
- Provides a summary of actions taken
- Analysis mode to preview changes without moving files
- Verbose mode for detailed progress information

## Installation

1. Clone this repository
2. Install dependencies:
```bash
pip install --user -r requirements.txt
```

## Usage

### Organize Files
To organize files in a folder:
```bash
python desktidy.py <folder_path>
```

### Analyze Only
To analyze files without moving them (dry run):
```bash
python desktidy.py --analyze <folder_path>
```

### Verbose Output
Add the `-v` or `--verbose` flag for detailed progress:
```bash
python desktidy.py --analyze -v <folder_path>
```

For example:
```bash
# Organize files
python desktidy.py ~/Documents

# Just analyze without moving files
python desktidy.py --analyze ~/Documents

# Analyze with detailed progress
python desktidy.py --analyze -v ~/Documents
```

The tool will:
1. Scan for Microsoft Office files in the specified folder
   - With verbose mode: Shows each file as it's processed
2. Create category folders (Presentations, Documents, Spreadsheets)
3. Move files into their respective category folders (unless in analyze mode)
4. Identify and group duplicate files
   - With verbose mode: Shows duplicate matches as they're found
5. Show you a summary of all actions taken or proposed
