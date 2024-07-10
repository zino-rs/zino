use std::fs;
use std::path::Path;

use clap::Parser;

use zino_core::error::Error;

use crate::cli::{process_template, DEFAULT_TEMPLATE_URL, TEMPORARY_TEMPLATE_PATH};

//Creat a project for Zino.
#[derive(Parser)]
#[clap(name = "new")]
pub struct New {
    /// Project Name
    project_name: String,

    /// Template path
    #[clap(long)]
    template: Option<String>,
}

impl New {
    /// Runs the `new` subcommand.
    pub fn run(self) -> Result<(), Error> {
        // Check if the project directory already exists and is not empty.
        let project_dir_already_exists = self.check_project_dir_status()?;

        let new_res = self.new_with_template();

        // Remove the temporary template directory.
        if let Err(e) = fs::remove_dir_all(TEMPORARY_TEMPLATE_PATH) {
            println!("Failed to remove the temporary template directory: {}", e);
        }

        // Process result of the creation.
        match new_res {
            Ok(_) => {
                println!("Project `{}` created successfully.", self.project_name);
                Ok(())
            }
            // clean up the project directory if the project directory was created but the creation failed
            // will not be executed if the Project directory already existed and was not empty
            Err(e) if !project_dir_already_exists => {
                if let Err(e) = fs::remove_dir_all(&self.project_name) {
                    eprintln!("Warning: Failed to remove project directory: {e}");
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }

    fn check_project_dir_status(&self) -> Result<bool, Error> {
        let path = Path::new(self.project_name.as_str());
        let project_dir_already_exists = path.exists() && path.is_dir();

        if project_dir_already_exists && fs::read_dir(&self.project_name)?.next().is_some() {
            return Err(Error::new(format!(
                "The directory `{}` already exists and is not empty.",
                self.project_name
            )));
        }

        Ok(project_dir_already_exists)
    }

    fn new_with_template(&self) -> Result<(), Error> {
        let template_url = match self.template {
            Some(ref template) => template.as_ref(),
            None => DEFAULT_TEMPLATE_URL,
        };
        let project_root = &format!("/{}", self.project_name);

        process_template(template_url, project_root, &self.project_name)
    }
}
