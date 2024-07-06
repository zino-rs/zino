use std::fs::File;
use std::io::Write;
use std::{env, fs};

use clap::Parser;
use include_dir::Dir;

use zino_core::error::Error;

use crate::cli::TEMPLATE_ROOT;

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
        match self.template {
            Some(template) => Self::init_with_template(template),
            None => self.init_default(),
        }
    }

    fn init_with_template(_template: String) -> Result<(), Error> {
        todo!("Implement the `init` subcommand with a template");
    }

    fn init_default(self) -> Result<(), Error> {
        // Determine the project name
        let project_name = match self.project_name {
            Some(ref name) => name,
            None => &env::current_dir()?
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        };

        // Iterate over all files in the template directory
        self.copy_template_files(&TEMPLATE_ROOT, &project_name)?;

        Ok(())
    }

    fn copy_template_files(&self, dir: &Dir, project_name: &str) -> Result<(), Error> {
        for file in dir.files() {
            let content = file.contents_utf8().unwrap();
            let replaced_content =
                content.replace("{project-name}", &format!("\"{}\"", project_name));

            let path = file.path().strip_prefix("default").unwrap();
            let mut file = File::create(path)?;
            file.write_all(replaced_content.as_bytes())?;
        }

        for subdir in dir.dirs() {
            let path = subdir.path().strip_prefix("default").unwrap();
            fs::create_dir_all(path)?;
            self.copy_template_files(subdir, project_name)?;
        }

        Ok(())
    }
}
