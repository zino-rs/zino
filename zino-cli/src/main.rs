use std::env;
use clap::Parser;
use zino_cli::{Cli, Subcommands::*};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let result = match Cli::parse().action() {
        Init(opts) => opts.run(),
        New(opts) => opts.run(),
        Serve(opts) => opts.run(),
    };
    if let Err(err) = result {
        log::error!("fail to run the command: {err}");
    }
}
