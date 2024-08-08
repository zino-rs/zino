use crate::cli::{
    clean_template_dir, process_template, DEFAULT_TEMPLATE_URL, TEMPORARY_TEMPLATE_PATH,
};
use clap::Parser;
use std::{env, path::Path};
use zino_core::error::Error;

/// Initializes the project.
#[derive(Parser)]
#[clap(name = "init")]
pub struct Init {
    /// The template path.
    #[clap(long)]
    template: Option<String>,
    /// The project name (the directory name if not specified).
    #[clap(long)]
    project_name: Option<String>,
}

impl Init {
    /// Runs the `init` subcommand.
    pub fn run(self) -> Result<(), Error> {
        if Path::new("./Cargo.toml").is_file() {
            return Err(Error::new("current directory is already a Rust project"));
        }
        let init_res = self.init_with_template();
        // must clean the temporary template directory after the initialization
        clean_template_dir(TEMPORARY_TEMPLATE_PATH);
        match init_res {
            Ok(_) => {
                log::info!("project initialized successfully");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Initializes the project with the template.
    fn init_with_template(&self) -> Result<(), Error> {
        let current_dir = env::current_dir()?
            .file_name()
            .expect("fail to get the current directory name")
            .to_str()
            .expect("fail to convert the directory name to string")
            .to_string();
        let project_name = match &self.project_name {
            Some(project_name) => project_name,
            None => &current_dir,
        };
        let template_url = match self.template {
            Some(ref template) => template.as_ref(),
            None => DEFAULT_TEMPLATE_URL,
        };
        process_template(template_url, "", project_name)
    }
}
