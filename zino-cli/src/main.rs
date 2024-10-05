use clap::Parser;
use zino_cli::{Cli, Subcommands::*};

fn main() {
    let result = match Cli::parse().action() {
        Init(opts) => opts.run(),
        New(opts) => opts.run(),
        Serve(opts) => opts.run(),
        Deploy(opts) => {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(opts.run());
            unreachable!()
        }
    };
    if let Err(err) = result {
        log::error!("fail to run the command: {err}");
    }
}
