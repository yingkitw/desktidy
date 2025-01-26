#!/usr/bin/env python3
import os
import shutil
import hashlib
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple
import click
from rich.console import Console
from rich.table import Table

console = Console()

class DeskTidy:
    # List of supported file extensions
    SUPPORTED_EXTENSIONS = {
        # Office Documents
        'ppt': 'Presentations',
        'pptx': 'Presentations',
        'doc': 'Documents',
        'docx': 'Documents',
        'xls': 'Spreadsheets',
        'xlsx': 'Spreadsheets',
        
        # PDFs
        'pdf': 'PDFs',
        
        # Images
        'jpg': 'Images',
        'jpeg': 'Images',
        'png': 'Images',
        'gif': 'Images',
        'bmp': 'Images',
        'tiff': 'Images',
        'webp': 'Images',
        'heic': 'Images',
        'raw': 'Images',
        'cr2': 'Images',
        'nef': 'Images',
        'arw': 'Images',
        
        # Videos
        'mp4': 'Videos',
        'mov': 'Videos',
        'avi': 'Videos',
        'mkv': 'Videos',
        'wmv': 'Videos',
        'flv': 'Videos',
        'webm': 'Videos',
        'm4v': 'Videos',
        '3gp': 'Videos',
        
        # Audio
        'mp3': 'Audio',
        'wav': 'Audio',
        'aac': 'Audio',
        'ogg': 'Audio',
        'flac': 'Audio',
        'm4a': 'Audio',
        'wma': 'Audio',
        'aiff': 'Audio'
    }

    # Category display order
    CATEGORY_ORDER = [
        'Documents',
        'PDFs',
        'Presentations',
        'Spreadsheets',
        'Images',
        'Videos',
        'Audio'
    ]

    def __init__(self, folder_path: str, verbose: bool = False):
        self.folder_path = Path(folder_path).resolve()
        self.file_categories = defaultdict(list)
        self.duplicate_files = defaultdict(list)
        self.actions_taken = []
        self.verbose = verbose
        
    def get_file_checksums(self, file_path: Path) -> Tuple[str, str]:
        """Calculate both MD5 and SHA256 checksums of file content.
        
        Returns:
            Tuple[str, str]: (md5_hash, sha256_hash)
        """
        try:
            md5_hash = hashlib.md5()
            sha256_hash = hashlib.sha256()
            
            with open(str(file_path.resolve()), "rb") as f:
                # Read the file once but update both hashes
                while chunk := f.read(8192):  # 8KB chunks
                    md5_hash.update(chunk)
                    sha256_hash.update(chunk)
                    
            return (md5_hash.hexdigest(), sha256_hash.hexdigest())
        except (OSError, IOError) as e:
            if self.verbose:
                console.print(f"[red]Error reading file {file_path.relative_to(self.folder_path)}: {str(e)}[/red]")
            return ("", "")  # Return empty hashes for failed reads
    
    def get_file_size(self, file_path: Path) -> int:
        """Get file size in bytes."""
        try:
            return file_path.resolve().stat().st_size
        except (OSError, IOError) as e:
            if self.verbose:
                console.print(f"[red]Error getting size for {file_path.relative_to(self.folder_path)}: {str(e)}[/red]")
            return -1
    
    def are_files_identical(self, file1: Path, file2: Path) -> bool:
        """Compare two files using size and checksums."""
        try:
            # First compare sizes (fast)
            size1 = self.get_file_size(file1)
            size2 = self.get_file_size(file2)
            
            if size1 == -1 or size2 == -1:  # Error reading file size
                return False
                
            if size1 != size2:
                return False
                
            # Then compare both checksums (thorough)
            checksums1 = self.get_file_checksums(file1)
            checksums2 = self.get_file_checksums(file2)
            
            if checksums1[0] == "" or checksums2[0] == "":  # Error reading file
                return False
            
            return checksums1 == checksums2
        except Exception as e:
            if self.verbose:
                console.print(f"[red]Error comparing files: {str(e)}[/red]")
            return False
    
    def analyze_files(self):
        """Analyze files in the directory and categorize supported files."""
        total_files = 0
        supported_files = 0
        duplicates_dir = self.folder_path / "Duplicates"
        
        if self.verbose:
            console.print("\n[bold blue]Starting file analysis...[/bold blue]")
        
        try:
            # Only look at files in the root folder
            for item in self.folder_path.iterdir():
                try:
                    # Skip folders and the Duplicates folder
                    if item.is_dir() or item == duplicates_dir:
                        if self.verbose:
                            console.print(f"[yellow]Skipping folder[/yellow]: {item.relative_to(self.folder_path)}")
                        continue
                        
                    total_files += 1
                    ext = item.suffix.lower()[1:]  # Remove the dot from extension
                    if ext in self.SUPPORTED_EXTENSIONS:
                        supported_files += 1
                        category = self.SUPPORTED_EXTENSIONS[ext]
                        if self.verbose:
                            console.print(f"[green]Found[/green] {category} file: {item.relative_to(self.folder_path)}")
                        self.file_categories[category].append(item)
                except (OSError, IOError) as e:
                    if self.verbose:
                        console.print(f"[red]Error accessing file {item}: {str(e)}[/red]")
                    continue
            
            if self.verbose:
                console.print(f"\n[bold]Analysis Summary:[/bold]")
                console.print(f"Total files scanned: {total_files}")
                console.print(f"Supported files found: {supported_files}")
                for category, files in self.file_categories.items():
                    console.print(f"{category}: {len(files)} files")
        except Exception as e:
            console.print(f"[red]Error during analysis: {str(e)}[/red]")

    def create_category_folders(self):
        """Create folders for file categories."""
        for category in set(self.SUPPORTED_EXTENSIONS.values()):
            if any(files for cat, files in self.file_categories.items() if cat == category):
                category_folder = self.folder_path / category
                try:
                    if not category_folder.exists():
                        category_folder.mkdir(exist_ok=True)
                        self.actions_taken.append(f"Created category folder: {category}")
                except OSError as e:
                    console.print(f"[red]Error creating folder {category}: {str(e)}[/red]")
    
    def find_duplicates(self):
        """Find duplicate files based on content."""
        if self.verbose:
            console.print("\n[bold blue]Checking for duplicates...[/bold blue]")
        
        # Dictionary to store files by their combined checksum
        file_checksums = defaultdict(list)
        duplicates_dir = self.folder_path / "Duplicates"
        
        # First pass: collect all files and their checksums
        for category, files in self.file_categories.items():
            for file_path in files:
                # Skip files in Duplicates folder
                if duplicates_dir in file_path.parents or file_path == duplicates_dir:
                    continue
                    
                # Get both MD5 and SHA256 checksums
                checksums = self.get_file_checksums(file_path)
                # Use combined checksums as key
                checksum_key = f"{checksums[0]}_{checksums[1]}"
                
                # Store the file path along with its category
                file_checksums[checksum_key].append((file_path, category))
        
        # Second pass: identify duplicates
        self.duplicate_files.clear()  # Clear any existing duplicates
        for checksum_key, file_entries in file_checksums.items():
            if len(file_entries) > 1:
                # Verify files are actually identical
                base_file = file_entries[0][0]  # First file path
                identical_files = []
                
                for file_entry in file_entries:
                    file_path = file_entry[0]
                    if file_path == base_file or self.are_files_identical(base_file, file_path):
                        identical_files.append(file_entry)
                
                if len(identical_files) > 1:
                    # Sort by creation time, keep the oldest file
                    identical_files.sort(key=lambda x: x[0].stat().st_ctime)
                    self.duplicate_files[checksum_key] = identical_files
                    
                    if self.verbose:
                        kept_file = identical_files[0][0]
                        console.print(f"[yellow]Found duplicates[/yellow]: Keeping {kept_file.relative_to(self.folder_path)}")
                        for file_entry in identical_files[1:]:
                            console.print(f"  - Will move: {file_entry[0].relative_to(self.folder_path)}")

    def clean_filename(self, filepath: Path) -> Path:
        """Clean up filename by removing old style numbering (_1, _2, etc).
        
        Args:
            filepath: Path to the file
            
        Returns:
            Path: Cleaned file path
        """
        # Check if the filename ends with _number before the extension
        stem = filepath.stem
        import re
        
        # Match patterns like "_1", "_2", etc. at the end of the filename
        if re.search(r'_\d+$', stem):
            # Remove the _number suffix
            clean_stem = re.sub(r'_\d+$', '', stem)
            return filepath.parent / f"{clean_stem}{filepath.suffix}"
        return filepath

    def get_unique_path(self, target_path: Path) -> Path:
        """Get a unique path for the file, adding a suffix only if necessary.
        
        Args:
            target_path: The desired target path for the file
            
        Returns:
            Path: A unique path that doesn't exist
        """
        # First clean up the target path
        target_path = self.clean_filename(target_path)
        
        if not target_path.exists():
            return target_path
            
        counter = 1
        while True:
            # For a file like "document.pdf", this will create "document (1).pdf"
            new_path = target_path.parent / f"{target_path.stem} ({counter}){target_path.suffix}"
            if not new_path.exists():
                return new_path
            counter += 1

    def safe_move(self, source: Path, dest: Path) -> bool:
        """Safely move a file, handling special characters and spaces.
        
        Args:
            source: Source file path
            dest: Destination file path
            
        Returns:
            bool: True if move was successful, False otherwise
        """
        try:
            # Resolve any symlinks and get absolute paths
            source = source.resolve()
            dest = dest.resolve()
            
            # Create parent directory if it doesn't exist
            dest.parent.mkdir(parents=True, exist_ok=True)
            
            # Use shutil.move with resolved paths converted to strings
            shutil.move(str(source), str(dest))
            return True
        except (OSError, IOError) as e:
            if self.verbose:
                console.print(f"[red]Error moving file {source.relative_to(self.folder_path)} to {dest.relative_to(self.folder_path)}: {str(e)}[/red]")
            return False

    def organize_files(self, dry_run: bool = False):
        """Organize files into their respective folders.
        
        Args:
            dry_run (bool): If True, only simulate the organization without moving files.
        """
        # First, clean up existing files in category folders
        if not dry_run:
            for category in set(self.SUPPORTED_EXTENSIONS.values()):
                category_folder = self.folder_path / category
                if category_folder.exists():
                    for file_path in category_folder.glob("*"):
                        if file_path.is_file():
                            clean_path = self.clean_filename(file_path)
                            if clean_path != file_path:
                                new_path = self.get_unique_path(clean_path)
                                if self.safe_move(file_path, new_path):
                                    self.actions_taken.append(f"Renamed {file_path.relative_to(self.folder_path)} to {new_path.relative_to(self.folder_path)}")

        # Create a set of files to skip (duplicates that will be moved)
        files_to_skip = set()
        for duplicate_group in self.duplicate_files.values():
            # Skip the first file (we'll keep it)
            for file_entry in duplicate_group[1:]:
                files_to_skip.add(file_entry[0])

        # Move files to category folders
        for category, files in self.file_categories.items():
            category_folder = self.folder_path / category
            for file_path in files:
                # Skip if this file will be moved to duplicates
                if file_path in files_to_skip:
                    continue
                    
                if file_path.parent != category_folder:
                    new_path = self.get_unique_path(category_folder / file_path.name)
                    
                    if not dry_run:
                        if self.safe_move(file_path, new_path):
                            self.actions_taken.append(f"Moved {file_path.relative_to(self.folder_path)} to {category} folder")
                        else:
                            continue
                    else:
                        self.actions_taken.append(f"Would move {file_path.relative_to(self.folder_path)} to {category} folder")
        
        # Organize duplicates
        if self.duplicate_files:
            dup_folder = self.folder_path / "Duplicates"
            if not dry_run:
                try:
                    dup_folder.mkdir(exist_ok=True)
                except OSError as e:
                    console.print(f"[red]Error creating duplicates folder: {str(e)}[/red]")
                    return
            
            for hash_value, duplicate_files in self.duplicate_files.items():
                # Keep the first (oldest) file, move all others to duplicates
                for file_entry in duplicate_files[1:]:
                    file_path = file_entry[0]
                    if file_path.parent != dup_folder:
                        new_path = self.get_unique_path(dup_folder / file_path.name)
                        
                        if not dry_run:
                            if self.safe_move(file_path, new_path):
                                original = duplicate_files[0][0]
                                self.actions_taken.append(
                                    f"Moved duplicate file {file_path.relative_to(self.folder_path)} to Duplicates folder "
                                    f"(identical to {original.relative_to(self.folder_path)})"
                                )
                            else:
                                continue
                        else:
                            original = duplicate_files[0][0]
                            self.actions_taken.append(
                                f"Would move duplicate file {file_path.relative_to(self.folder_path)} to Duplicates folder "
                                f"(identical to {original.relative_to(self.folder_path)})"
                            )
    
    def get_category_color(self, category: str) -> str:
        """Get the color for a category in the console output."""
        CATEGORY_COLORS = {
            'Documents': 'blue',
            'PDFs': 'red',
            'Presentations': 'magenta',
            'Spreadsheets': 'green',
            'Images': 'cyan',
            'Videos': 'yellow',
            'Audio': 'red'
        }
        return CATEGORY_COLORS.get(category, 'white')

    def display_summary(self, dry_run: bool = False):
        """Display a summary of actions taken or to be taken."""
        if dry_run:
            console.print("\n[bold yellow]Analysis Mode (No files will be moved)[/bold yellow]")
        
        table = Table(title="File Organization Summary")
        table.add_column("Category", style="cyan")
        table.add_column("Count", style="magenta")
        table.add_column("Files", style="blue")
        
        # File categories summary - display in specified order
        for category in self.CATEGORY_ORDER:
            files = self.file_categories.get(category, [])
            if files:  # Only show categories that have files
                file_list = [str(f.relative_to(self.folder_path)) for f in files]
                color = self.get_category_color(category)
                table.add_row(
                    f"[{color}]{category}[/{color}]",
                    str(len(files)),
                    "\n".join(file_list) if file_list else "None"
                )
        
        console.print(table)
        
        # Duplicates summary
        if self.duplicate_files:
            console.print("\n[yellow]Duplicate Files Found:[/yellow]")
            for hash_value, files in self.duplicate_files.items():
                console.print(f"Group {hash_value[:8]}: {len(files)} files")
                for f in files:
                    console.print(f"  - {f[0].relative_to(self.folder_path)}")
        
        # Actions summary
        if self.actions_taken:
            console.print(f"\n[green]{'Proposed Actions' if dry_run else 'Actions Taken'}:[/green]")
            for action in self.actions_taken:
                console.print(f"✓ {action}")
        else:
            console.print("\n[yellow]No files found to organize.[/yellow]")
    
@click.command()
@click.argument('folder_path', type=click.Path(exists=True))
@click.option('--analyze', is_flag=True, help='Only analyze files without moving them')
@click.option('--verbose', '-v', is_flag=True, help='Show detailed progress during analysis')
def organize(folder_path, analyze, verbose):
    """Organize files (Office Documents, PDFs, Images, Videos, Audio) in the specified folder.
    
    Only processes files in the root folder, ignoring subfolders.
    """
    try:
        tidier = DeskTidy(folder_path, verbose=verbose)
        
        if analyze:
            with console.status("[bold green]Analyzing files..."):
                tidier.analyze_files()
        else:
            tidier.analyze_files()
            
        tidier.find_duplicates()
        
        if not analyze:
            tidier.create_category_folders()
            tidier.organize_files()
        
        tidier.display_summary(dry_run=analyze)
        
    except Exception as e:
        console.print(f"[red]Error: {str(e)}[/red]")
        return 1
    return 0

if __name__ == '__main__':
    organize()
