use clap::Parser;
use zino_cli::{Cli, Subcommands::*};

fn main() {
    let result = match Cli::parse().action() {
        Init(opts) => opts.run(),
        New(opts) => opts.run(),
        Serve(opts) => opts.run(),
    };
    if let Err(err) = result {
        log::error!("fail to run the command: {err}");
    }
}
