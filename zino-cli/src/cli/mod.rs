//! CLI arguments and subcommands.

use clap::Parser;
use git2::Repository;
use regex::Regex;
use std::fs;
use std::fs::remove_dir_all;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use zino_core::error::Error;

mod init;
mod new;

mod serve;

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
    /// Initialize the project.
    Init(init::Init),
    /// Create a new project.
    New(new::New),
    /// Start the server at localhost:6080/zino-config.html.
    Serve(serve::Serve),
}

/// Default path for temporary template.
pub(crate) static TEMPORARY_TEMPLATE_PATH: &str = "./temporary_zino_template";

/// Default template URL.
pub(crate) static DEFAULT_TEMPLATE_URL: &str =
    "https://github.com/zino-rs/zino-template-default.git";

/// Clones the template repository, do replacements, and create the project.
pub(crate) fn clone_and_process_template(
    template_url: &str,
    target_path_prefix: &str,
    project_name: &str,
) -> Result<(), Error> {
    Repository::clone(template_url, TEMPORARY_TEMPLATE_PATH)?;

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
                    .ok_or_else(|| Error::new(
                        "fail to convert the template file path to string"
                    ))?
            );
            fs::create_dir_all(Path::new(&target_path).parent().unwrap())?;

            let content =
                fs::read_to_string(template_file_path)?.replace("{project-name}", project_name);
            fs::write(&target_path, content)?;
        }
    }

    Ok(())
}

/// Helper function to determine ignored files.
fn is_ignored(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map_or(false, |s| {
        s.starts_with('.') || s == "LICENSE" || s == "README.md"
    })
}

/// Clean the temporary template directory.
fn clean_template_dir(path: &str) {
    let _ = remove_dir_all(path);
}

/// Check name validity.
pub(crate) fn check_package_name_validation(name: &str) -> Result<(), Error> {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$")
        .map_err(|e| Error::new(e.to_string()))?
        .is_match(name)
        .then_some(())
        .ok_or_else(|| Error::new(format!("invalid package name: `{}`", name)))
}
