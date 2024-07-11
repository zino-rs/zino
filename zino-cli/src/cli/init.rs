use std::env;
use std::path::Path;

use clap::Parser;

use zino_core::error::Error;

use crate::cli::{clean_template_dir, DEFAULT_TEMPLATE_URL, process_template, TEMPORARY_TEMPLATE_PATH};

/// Initialize the project for Zino.
#[derive(Parser)]
#[clap(name = "init")]
pub struct Init {
    /// Template path.
    #[clap(long)]
    template: Option<String>,

    /// Project Name(directory name if not specified).
    #[clap(long)]
    project_name: Option<String>,
}

impl Init {
    /// Runs the `init` subcommand.
    pub fn run(self) -> Result<(), Error> {
        if Path::new("./Cargo.toml").is_file() {
            return Err(Error::new(
                "The current directory is already a Rust project.",
            ));
        }

        let init_res = self.init_with_template();

        clean_template_dir(TEMPORARY_TEMPLATE_PATH);

        match init_res {
            Ok(_) => {
                println!("Project initialized successfully.", );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn init_with_template(&self) -> Result<(), Error> {
        let current_dir = env::current_dir()?
            .file_name()
            .expect("Failed to get the current directory name")
            .to_str()
            .expect("Failed to convert the directory name to string")
            .to_string();

        let project_name = match &self.project_name {
            Some(project_name) => project_name,
            None => &current_dir
        };

        let template_url = match self.template {
            Some(ref template) => template.as_ref(),
            None => DEFAULT_TEMPLATE_URL,
        };

        process_template(template_url, "", project_name)
    }
}
