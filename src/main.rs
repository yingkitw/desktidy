use clap::Parser;
use desktidy::{
    display::DisplayFormatter, duplicate_finder::DuplicateFinder, file_analyzer::FileAnalyzer,
    organizer::Organizer,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "desktidy",
    about = "Organize files (Office Documents, PDFs, Images, Videos, Audio) in a folder",
    long_about = "A command-line tool to organize files into categorized folders.\nOnly processes files in the root folder, ignoring subfolders."
)]
struct Args {
    /// Path to the folder to organize
    #[arg(value_name = "FOLDER_PATH")]
    folder_path: PathBuf,

    /// Only analyze files without moving them (dry run)
    #[arg(long)]
    analyze: bool,

    /// Show detailed progress during analysis
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Analyze files
    let analyzer = FileAnalyzer::new(args.folder_path.clone(), args.verbose);
    let analysis = analyzer.analyze()?;

    // Collect all entries
    let mut all_entries = Vec::new();
    for entries in analysis.categories.values() {
        all_entries.extend(entries.clone());
    }

    // Find duplicates
    let finder = DuplicateFinder::new(args.verbose);
    let duplicates = finder.find_duplicates(&all_entries)?;

    // Organize files
    let organizer = Organizer::new(args.folder_path.clone(), args.verbose);

    if !args.analyze {
        // Create category folders
        let categories: Vec<&str> = analysis
            .categories
            .keys()
            .map(|c| c.as_str())
            .collect();
        organizer.create_category_folders(&categories)?;
    }

    // Organize files
    let summary = organizer.organize_files(&all_entries, &duplicates, args.analyze)?;

    // Display summary
    DisplayFormatter::display_summary(
        &all_entries,
        &summary.duplicates_found,
        &summary.actions_taken,
        args.analyze,
        &args.folder_path,
    );

    Ok(())
}
