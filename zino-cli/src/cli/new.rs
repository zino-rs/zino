use crate::cli::{
    check_package_name_validation, clean_template_dir, clone_and_process_template,
    DEFAULT_TEMPLATE_URL, TEMPORARY_TEMPLATE_PATH,
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
        let new_res = self.new_with_template();
        // must clean the temporary template directory after the initialization
        clean_template_dir(TEMPORARY_TEMPLATE_PATH);
        new_res
            .map(|_| {
                log::info!("project `{}` created successfully", self.project_name);
            })
            .map_err(|err| {
                if !project_dir_already_exists && Path::new("./Cargo.toml").is_dir() {
                    if let Err(err) = fs::remove_dir_all(&self.project_name) {
                        log::warn!(
                            "fail to remove project directory:{}, {err}",
                            self.project_name
                        );
                    }
                }
                err
            })
    }

    /// Checks if the project directory already exists and if it's empty.
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

    /// Creates a new project with the template.
    fn new_with_template(&self) -> Result<(), Error> {
        let template_url = self.template.as_deref().unwrap_or(DEFAULT_TEMPLATE_URL);
        check_package_name_validation(&self.project_name)?;
        let project_root = &format!("/{}", &self.project_name);
        clone_and_process_template(template_url, project_root, &self.project_name)
    }
}
