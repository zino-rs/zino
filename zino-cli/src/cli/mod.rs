//! CLI arguments and subcommands.

use clap::Parser;

mod init;

/// CLI tool for developing Zino applications.
#[derive(Parser)]
#[clap(name = "zino", version)]
pub struct Cli {
    /// Specify the bin target.
    #[clap(global = true, long)]
    bin: Option<String>,
    /// Subcomands.
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
}
