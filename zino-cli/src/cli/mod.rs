//! CLI arguments and subcommands.

use clap::Parser;
use git2::Repository;
use std::fs;
use std::fs::remove_dir_all;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use zino_core::error::Error;

mod init;
mod new;

/// CLI tool for developing Zino applications.
#[derive(Parser)]
#[clap(name = "zino", version)]
pub struct Cli {
    /// Specify the bin target.
    #[clap(global = true, long)]
    bin: Option<String>,
    /// Subcommands.
    #[clap(subcommand)]
    action: Subcommands,
    /// Enable verbose logging.
    #[clap(long)]
    verbose: bool,
}

impl Cli {
    /// Returns the subcommand action.
    #[inline]
    pub fn action(self) -> Subcommands {
        self.action
    }
}

/// CLI subcommands.
#[derive(Parser)]
pub enum Subcommands {
    /// Initialize the project for Zino.
    Init(init::Init),
    /// Create a new project for Zino.
    New(new::New),
}

pub(crate) static TEMPORARY_TEMPLATE_PATH: &str = "./temporary_zino_template";
pub(crate) static DEFAULT_TEMPLATE_URL: &str =
    "https://github.com/zino-rs/zino-template-default.git";

pub(crate) fn process_template(
    template_url: &str,
    target_path_prefix: &str,
    project_name: &str,
) -> Result<(), Error> {
    // Clone the template repository.
    Repository::clone(template_url, TEMPORARY_TEMPLATE_PATH)?;

    // process the template
    for entry in WalkDir::new(TEMPORARY_TEMPLATE_PATH)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let template_file_path = entry.path();
            let target_path = format!(
                ".{}/{}",
                target_path_prefix,
                template_file_path
                    .strip_prefix(TEMPORARY_TEMPLATE_PATH)?
                    .to_str()
                    .unwrap()
            );

            fs::create_dir_all(Path::new(&target_path).parent().unwrap())?;

            let content =
                fs::read_to_string(template_file_path)?.replace("{project-name}", &project_name);
            fs::write(&target_path, content)?;
        }
    }

    Ok(())
}

// Helper function to determine ignored files
fn is_ignored(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map_or(false, |s| {
        s.starts_with(".") || s == "LICENSE" || s == "README.md"
    })
}

fn clean_template_dir(path: &str) {
    if let Err(e) = remove_dir_all(path) {
        println!("Failed to remove the temporary template directory: {}", e)
    }
}
