use crate::cli::{
    check_package_name_validation, clean_template_dir, clone_and_process_template,
    DEFAULT_TEMPLATE_URL, TEMPORARY_TEMPLATE_PATH,
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
        init_res.map(|_| {
            log::info!("project initialized successfully");
        })
    }

    /// Initializes the project with the template.
    fn init_with_template(&self) -> Result<(), Error> {
        let current_dir = env::current_dir()?
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::new("fail to get or convert the current directory name"))?;
        let project_name = match &self.project_name {
            Some(project_name) => {
                check_package_name_validation(project_name)?;
                project_name
            }
            None => {
                check_package_name_validation(&current_dir).map_err(|_| {
                    Error::new(format!(
                        "current directory's name:{} is not a valid Rust package name,\
                        try to specify the project name with `--project-name`",
                        &current_dir
                    ))
                })?;
                &current_dir
            }
        };
        let template_url = self.template.as_deref().unwrap_or(DEFAULT_TEMPLATE_URL);
        clone_and_process_template(template_url, "", project_name)
    }
}
