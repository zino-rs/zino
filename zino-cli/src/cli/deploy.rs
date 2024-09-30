use clap::Parser;
use git2::{FetchOptions, Remote, Repository, ResetType};
use humantime_serde::re::humantime::format_duration;
use std::{
    net::Ipv4Addr,
    process::{Child, Command},
};
use rustls_acme::AcmeConfig;
use rustls_acme::caches::DirCache;
use tracing::{error, info, warn};

use tokio_stream::StreamExt;

use zino_core::error::Error;

use crate::structs::ZinoToml;

/// Deploy a zino project.
#[derive(Parser, Debug)]
#[clap(name = "deploy")]
pub struct Deploy {}

impl Deploy {
    /// Run the `deploy` command.
    pub async fn run(self) {
        tracing_subscriber::fmt::init();
        info!("deploying zino project");
        let repository_manager = RepositoryManager {
            zino_toml: ZinoToml::default(),
            active_project: None,
            last_checked_commit_oid: None,
        };
        let acme_manager = AcmeManager {
            zino_toml: ZinoToml::default(),
        };

        tokio::join!( async {
            if let Err(err) = acme_manager.do_acme_work_and_forward_connections().await {
                error!("failed to do acme work and forward connections: {}", err);
            }
        },repository_manager.main_loop());
    }
}

pub struct RepositoryManager {
    zino_toml: ZinoToml,
    active_project: Option<Child>,
    last_checked_commit_oid: Option<git2::Oid>,
}

impl RepositoryManager {
    async fn main_loop(mut self) {
        if let Err(err) = self.run_project(){
            error!("failed to run project: {}", err);
        }
        loop {
            match self.main_loop_body() {
                Ok(_) => match self.local_head_oid() {
                    Ok(oid) => info!("current commit_id: {}", oid),
                    Err(err) => error!("failed to get current commit_id: {}", err),
                },
                Err(err) => {
                    error!("deploy failed: {}", err);
                    match self.rollback_to_latest_checked_commit() {
                        Ok(_) => info!(
                            "rolled back to last commit: {}",
                            self.last_checked_commit_oid
                                .unwrap_or(git2::Oid::zero())
                                .to_string()
                        ),
                        Err(err) => error!("failed to rollback to last commit: {}", err),
                    }
                }
            }
            tokio::time::sleep(self.zino_toml.zli_config.refresh_interval).await;
        }
    }

    /// Initialize the zino.toml file.
    fn flush_zino_toml(&mut self) {
        self.zino_toml = ZinoToml::default();

        info!(
            "RepositoryManager will check for updates after {} ",
            format_duration(self.zino_toml.zli_config.refresh_interval)
        );
    }

    /// The main loop body of the deploy command.
    fn main_loop_body(&mut self) -> Result<(), Error> {
        self.flush_zino_toml();

        let local_oid = match self.local_head_oid() {
            Ok(local_oid) => {
                info!("local commit_id: {}", local_oid);
                local_oid
            }
            Err(err) => {
                error!("failed to get local commit_id: {}", err);
                error!("RepositoryManager now cannot update the project, but will keep running the application witch is alive now until the problem is fixed.");
                error!("RepositoryManager will be able to update the project again after the problem is fixed.");
                return Ok(());
            }
        };

        let remote_oid = match self.remote_head_oid() {
            Ok(remote_oid) => {
                info!("remote commit_id: {}", remote_oid);
                remote_oid
            }
            Err(err) => {
                warn!("failed to get remote commit_id: {}", err);
                warn!("Give up update this time. The running application is not affected.");
                return Ok(());
            }
        };

        if self.last_checked_commit_oid.unwrap_or(local_oid) != remote_oid
            || self.active_project.is_none()
        {
            info!("updating local repository");

            self.pull_remote()?;

            self.check_and_test()?;

            self.kill_active_project();
            self.run_project()?;

            info!("project updated to commit: {}", remote_oid);
            info!("project deployed");
        } else {
            info!("local repository is up-to-date");
        }

        Ok(())
    }

    /// Kill the active project.
    fn kill_active_project(&mut self) {
        if let Some(mut active_project) = self.active_project.take() {
            match active_project.kill() {
                Ok(_) => self.active_project = None,
                Err(err) => error!("failed to kill active project: {}", err),
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
        let repo = Repository::open("./")?;

        let mut remote = repo.find_remote(&self.zino_toml.remote.name)?;

        let mut fo = FetchOptions::new();
        remote.fetch(&[&self.zino_toml.remote.branch], Some(&mut fo), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        let obj = repo.find_object(fetch_commit.id(), None)?;

        repo.reset(&obj, ResetType::Hard, None)?;

        Ok(())
    }

    /// Rollback to the latest checked commit.
    fn rollback_to_latest_checked_commit(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(
                self.last_checked_commit_oid
                    .ok_or(Error::new("no last active commit"))?
                    .to_string(),
            )
            .output()
            .map_err(|_| Error::new("failed to execute git reset"))?;

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

        info!("deploying new version of the project");

        Ok(())
    }

    /// Check and test the project.
    fn check_and_test(&mut self) -> Result<(), Error> {
        let oid = self.local_head_oid()?;

        self.run_cargo_command("check")?;
        self.run_cargo_command("test")?;

        self.last_checked_commit_oid = Some(oid);

        info!("project check and test passed: newest commit_id: {}", oid);
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
            info!("{} succeeded", command);
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

struct AcmeManager {
    zino_toml: ZinoToml,
}

/// about ACME
impl AcmeManager {
    async fn do_acme_work_and_forward_connections(&self) -> Result<(), Error> {
        println!("{:#?}", self.zino_toml);

        info!(
            "Starting to bind TCP listener on port {}",
            self.zino_toml.acme.listening_at
        );
        let tcp_listener = tokio::net::TcpListener::bind((
            Ipv4Addr::UNSPECIFIED,
            self.zino_toml.acme.listening_at,
        ))
        .await
        .map_err(|err| Error::new(format!("failed to bind TCP listener: {}", err)))?;
        info!("TCP listener bound successfully");

        // let mut tls_incoming = tokio_stream::wrappers::TcpListenerStream::new(tcp_listener);
        // info!("TCP incoming stream created");

        let tcp_incoming = tokio_stream::wrappers::TcpListenerStream::new(tcp_listener);

        let mut tls_incoming = AcmeConfig::new(self.zino_toml.acme.domain.clone())
            .contact(
                self.zino_toml
                    .acme
                    .email
                    .iter()
                    .map(|e| format!("mailto:{}", e)),
            )
            .cache_option(Some(self.zino_toml.acme.cache.clone()).map(DirCache::new))
            .directory_lets_encrypt(self.zino_toml.acme.product_mode)
            .tokio_incoming(tcp_incoming, Vec::new());
        info!("ACME configuration set up");

        while let Some(tls) = tls_incoming.next().await {
            let tls = match tls {
                Ok(tls) => tls,
                Err(err) => {
                    error!("Failed to accept TLS connection: {}", err);
                    continue;
                }
            };
            info!("Received a new TLS connection: {:?}", tls);

            let port = self.zino_toml.acme.forward_to;

            tokio::spawn(async move {
                if let Err(e) = Self::handle_tls_connection(tls, port).await {
                    error!("failed to handle tls connection: {}", e);
                } else {
                    info!("TLS connection handled successfully");
                }
            });
        }
        unreachable!()
    }

    async fn handle_tls_connection<T>(mut tls: T, forward_to: u16) -> Result<(), Error>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        // 连接到本机的 6080 端口
        let mut target_stream = tokio::net::TcpStream::connect(("127.0.0.1", forward_to))
            .await
            .map_err(|err| {
                error!("Failed to connect to target server: {}", err);
                Error::new(format!("failed to connect to target server: {}", err))
            })?;

        tokio::io::copy_bidirectional(&mut tls, &mut target_stream)
            .await
            .map_err(|err| {
                error!("Failed to forward connection: {}", err);
                Error::new(format!("failed to forward connection: {}", err))
            })?;

        Ok(())
    }
}
