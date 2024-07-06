use crate::cli::TEMPLATE_ROOT;
use clap::Parser;
use fs::File;
use include_dir::Dir;
use std::fs;
use std::io::Write;
use std::path::Path;
use zino_core::error::Error;

//Creat a project for Zino.
#[derive(Parser)]
#[clap(name = "new")]
pub struct New {
    /// Project Name
    project_name: String,
}

impl New {
    /// Runs the `new` subcommand.
    pub fn run(self) -> Result<(), Error> {
        // Check if the directory already exists
        if Path::new(&self.project_name).exists() {
            return Err(Error::new(format!(
                "Directory {} already exists\n\
                use a different name to create a new project\n\
                or cd into the directory and run `zli init` to initialize the project",
                self.project_name
            )));
        }

        // Create a new directory
        fs::create_dir_all(&self.project_name)?;

        // Iterate over all files in the template directory
        self.copy_template_files(&TEMPLATE_ROOT)
    }

    fn copy_template_files(&self, dir: &Dir) -> Result<(), Error> {
        for file in dir.files() {
            let content = file.contents_utf8().unwrap();
            let replaced_content =
                content.replace("{project-name}", &format!("\"{}\"", &self.project_name));

            let path =
                Path::new(&self.project_name).join(file.path().strip_prefix("default").unwrap());
            let mut file = File::create(path)?;
            file.write_all(replaced_content.as_bytes())?;
        }

        for subdir in dir.dirs() {
            let path =
                Path::new(&self.project_name).join(subdir.path().strip_prefix("default").unwrap());
            fs::create_dir_all(path)?;
            self.copy_template_files(subdir)?;
        }

        Ok(())
    }
}
