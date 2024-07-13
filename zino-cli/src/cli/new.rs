use crate::cli::{
    clean_template_dir, process_template, DEFAULT_TEMPLATE_URL, TEMPORARY_TEMPLATE_PATH,
};
use clap::Parser;
use std::{fs, path::Path};
use zino_core::error::Error;

/// Creates a project.
#[derive(Parser)]
#[clap(name = "new")]
pub struct New {
    /// Project Name.
    project_name: String,
    /// Template path.
    #[clap(long)]
    template: Option<String>,
}

impl New {
    /// Runs the `new` subcommand.
    pub fn run(self) -> Result<(), Error> {
        let project_dir_already_exists = self.check_project_dir_status()?;
        clean_template_dir(TEMPORARY_TEMPLATE_PATH);
        match self.new_with_template() {
            Ok(_) => {
                log::info!("project `{}` created successfully", self.project_name);
                Ok(())
            }
            Err(err) => {
                if !project_dir_already_exists {
                    if let Err(err) = fs::remove_dir_all(&self.project_name) {
                        log::warn!("fail to remove project directory: {err}");
                    }
                }
                Err(err)
            }
        }
    }

    /// Needs documentation.
    fn check_project_dir_status(&self) -> Result<bool, Error> {
        let path = Path::new(self.project_name.as_str());
        let project_dir_already_exists = path.exists() && path.is_dir();
        if project_dir_already_exists && fs::read_dir(&self.project_name)?.next().is_some() {
            return Err(Error::new(format!(
                "the directory `{}` already exists and is not empty",
                self.project_name
            )));
        }
        Ok(project_dir_already_exists)
    }

    /// Needs documentation.
    fn new_with_template(&self) -> Result<(), Error> {
        let template_url = match self.template {
            Some(ref template) => template.as_ref(),
            None => DEFAULT_TEMPLATE_URL,
        };
        let project_root = &format!("/{}", self.project_name);
        process_template(template_url, project_root, &self.project_name)
    }
}
