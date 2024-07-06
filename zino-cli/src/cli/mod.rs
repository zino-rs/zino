//! CLI arguments and subcommands.

use clap::Parser;
use include_dir::{include_dir, Dir};


mod init;
mod new;


static TEMPLATE_ROOT: Dir<'_> = include_dir!("zino-cli/template/");

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
