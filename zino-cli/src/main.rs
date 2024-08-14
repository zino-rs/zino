use clap::Parser;
use std::env;
use zino_cli::{Cli, Subcommands::*};

fn main() {
    env::set_var("RUST_LOG", "info");

    let result = match Cli::parse().action() {
        Init(opts) => {
            env_logger::init();
            opts.run()
        }
        New(opts) => {
            env_logger::init();
            opts.run()
        }
        Serve(opts) => opts.run(),
    };
    if let Err(err) = result {
        log::error!("fail to run the command: {err}");
    }
}
