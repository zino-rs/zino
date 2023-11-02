use clap::Parser;
use zino_core::error::Error;

/// Initialize the project for Zino.
#[derive(Parser)]
#[clap(name = "init")]
pub struct Init {
    /// Template path.
    #[clap(long)]
    template: String,
}

impl Init {
    /// Runs the `init` subcommand.
    pub fn run(self) -> Result<(), Error> {
        Ok(())
    }
}
