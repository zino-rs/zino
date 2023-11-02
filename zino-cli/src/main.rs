use clap::Parser;
use zino_cli::{Cli, Subcommands::*};

fn main() {
    let result = match Cli::parse().action() {
        Init(opts) => opts.run(),
    };
    if let Err(err) = result {
        log::error!("Failed to run the command: {err}");
    }
}
