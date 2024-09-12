use clap::Parser;
use git2::{Remote, Repository};
use std::thread::sleep;
use std::{
    fs,
    process::{Child, Command},
};

use zino_core::error::Error;

use crate::structs::ZinoToml;

/// Deploy a zino project.
#[derive(Parser, Debug)]
#[clap(name = "deploy")]
pub struct Deploy {
    #[clap(skip)]
    zino_toml: ZinoToml,
    #[clap(skip)]
    active_project: Option<Child>,
    #[clap(skip)]
    last_checked_commit_oid: Option<git2::Oid>,
    #[clap(skip)]
    last_active_commit_oid: Option<git2::Oid>,
}

/// about Deploy
impl Deploy {
    /// Run the `deploy` command.
    pub fn run(mut self) -> Result<(), Error> {
        log::info!("deploying zino project");

        loop {
            match self.main_loop() {
                Ok(_) => match self.local_head_oid() {
                    Ok(oid) => log::info!("current commit_id: {}", oid),
                    Err(err) => log::error!("failed to get current commit_id: {}", err),
                },
                Err(err) => {
                    log::error!("deploy failed: {}", err);
                    match self.rollback_to_latest_checked_commit() {
                        Ok(_) => log::info!("rolled back to last commit"),
                        Err(err) => log::error!("failed to rollback to last commit: {}", err),
                    }
                }
            }
            sleep(std::time::Duration::from_secs(5));
        }
    }

    /// Initialize the zino.toml file.
    fn init_zino_toml(&mut self) -> Result<(), Error> {
        self.zino_toml = self.parse_zino_toml().unwrap_or_default();
        Ok(())
    }

    /// Parse the zino.toml file.
    fn parse_zino_toml(&self) -> Result<ZinoToml, Error> {
        let zino_toml = fs::read_to_string("zino.toml")
            .map_err(|err| Error::new(format!("failed to read zino.toml: {}", err)))?;
        let zino_toml: ZinoToml = toml::from_str(&zino_toml)
            .map_err(|err| Error::new(format!("failed to parse zino.toml: {}", err)))?;
        Ok(zino_toml)
    }

    /// The main loop of the deploy command.
    fn main_loop(&mut self) -> Result<(), Error> {
        self.init_zino_toml()?;

        let local_oid = self.local_head_oid()?;
        let remote_oid = self.remote_head_oid()?;

        log::info!("local commit_id: {}", local_oid);
        log::info!("remote commit_id: {}", remote_oid);

        if self
            .last_checked_commit_oid
            .unwrap_or(self.local_head_oid()?)
            != remote_oid
            || self.active_project.is_none()
        {
            log::info!("updating local repository");

            self.pull_remote()?;

            self.check_and_test()?;

            self.kill_active_project();
            self.run_project()?;

            log::info!("project updated to commit: {}", remote_oid);
            log::info!("project deployed");
        } else {
            log::info!("local repository is up-to-date");
        }

        Ok(())
    }

    /// Kill the active project.
    fn kill_active_project(&mut self) {
        if let Some(mut active_project) = self.active_project.take() {
            match active_project
                .kill()
                .map_err(|_| Error::new("failed to kill active project"))
            {
                Ok(_) => self.active_project = None,
                Err(err) => log::error!("failed to kill active project: {}", err),
            }
        }
    }

    /// Open the local repository.
    fn open_local_repo(&self) -> Result<Repository, Error> {
        let repo = Repository::open(".")
            .map_err(|err| Error::new(format!("failed to open local repository: {}", err)))?;
        Ok(repo)
    }

    /// Pull the remote repository.
    fn pull_remote(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("pull")
            .arg(&self.zino_toml.remote.name)
            .arg(&self.zino_toml.remote.branch)
            .output()
            .map_err(|_| Error::new("failed to execute git pull"))?;

        log::info!("local repository updated");
        Ok(())
    }

    /// Rollback to the latest checked commit.
    fn rollback_to_latest_checked_commit(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(
                self.last_active_commit_oid
                    .ok_or(Error::new("no last active commit"))?
                    .to_string(),
            )
            .output()
            .map_err(|_| Error::new("failed to execute git reset"))?;

        log::info!("rolled back to last commit");
        Ok(())
    }

    /// Get the local head OID.
    fn local_head_oid(&self) -> Result<git2::Oid, Error> {
        let repo = self.open_local_repo()?;

        let head_oid = repo
            .head()
            .map_err(|err| Error::new(format!("failed to get repository head: {}", err)))?
            .peel_to_commit()
            .map_err(|err| Error::new(format!("failed to peel to commit: {}", err)))?
            .id();

        Ok(head_oid)
    }

    /// Get the remote head OID.
    fn remote_head_oid(&self) -> Result<git2::Oid, Error> {
        let repo = self.open_local_repo()?;

        let mut remote = self.find_remote(&repo)?;
        remote
            .fetch(&[self.zino_toml.remote.branch.clone()], None, None)
            .map_err(|err| Error::new(format!("failed to fetch remote: {}", err)))?;

        let remote_branch = repo
            .find_branch(
                &format!(
                    "{}/{}",
                    self.zino_toml.remote.name, self.zino_toml.remote.branch
                ),
                git2::BranchType::Remote,
            )
            .map_err(|err| Error::new(format!("failed to find Remote branch: {}", err)))?;

        let remote_head_oid = remote_branch
            .get()
            .peel_to_commit()
            .map_err(|err| Error::new(format!("failed to peel to commit: {}", err)))?
            .id();

        Ok(remote_head_oid)
    }

    /// Find the remote.
    fn find_remote<'a>(&self, repo: &'a Repository) -> Result<Remote<'a>, Error> {
        let remote = repo
            .find_remote(&self.zino_toml.remote.name)
            .map_err(|err| Error::new(format!("failed to find remote: {}", err)))?;
        Ok(remote)
    }

    /// Run the project.
    fn run_project(&mut self) -> Result<(), Error> {
        if self.active_project.is_none() {
            self.active_project = Some(
                Command::new("cargo")
                    .arg("run")
                    .arg("--release")
                    .arg("-q")
                    .spawn()
                    .map_err(|_| Error::new("failed to run the project"))?,
            );
        }

        log::info!("deploying new version of the project");

        Ok(())
    }

    /// Check and test the project.
    fn check_and_test(&mut self) -> Result<(), Error> {
        let oid = self.local_head_oid()?;
        self.last_checked_commit_oid = Some(oid);

        self.run_cargo_command("check")?;
        self.run_cargo_command("test")?;
        self.last_active_commit_oid = Some(oid);
        log::info!("project check and test passed: newest commit_id: {}", oid);
        Ok(())
    }

    /// Run a cargo command in a subprocess.
    fn run_cargo_command(&self, command: &str) -> Result<(), Error> {
        let status = Command::new("cargo")
            .arg(command)
            .arg("-q")
            .status()
            .map_err(|_| Error::new("failed to execute cargo command"))?;

        if status.success() {
            log::info!("{} succeeded", command);
            Ok(())
        } else {
            Err(Error::new(format!(
                "{} failed, status code: {}",
                command,
                status.code().unwrap_or(-1)
            )))
        }
    }
}