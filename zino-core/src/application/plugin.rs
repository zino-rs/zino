use crate::{
    error::Error,
    extension::TomlTableExt,
    state::{Env, State},
    BoxFuture,
};
use smallvec::SmallVec;
use toml::value::Table;

/// A custom plugin.
pub struct Plugin {
    /// Plugin name.
    name: &'static str,
    /// Plugin loader.
    loader: Option<BoxFuture<'static, Result<(), Error>>>,
    /// Running environments.
    environments: SmallVec<[Env; 2]>,
    /// Dependencies.
    dependencies: SmallVec<[&'static str; 2]>,
}

impl Plugin {
    /// Creates a new instance.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            loader: None,
            environments: SmallVec::new(),
            dependencies: SmallVec::new(),
        }
    }

    /// Sets an asynchronous loader for the plugin.
    #[inline]
    pub fn set_loader(&mut self, loader: BoxFuture<'static, Result<(), Error>>) {
        self.loader = Some(loader);
    }

    /// Enables the running environment [`Env::Dev`].
    #[inline]
    pub fn enable_dev(&mut self) {
        if !self.environments.contains(&Env::Dev) {
            self.environments.push(Env::Dev);
        }
    }

    /// Enables the running environment [`Env::Prod`].
    #[inline]
    pub fn enable_prod(&mut self) {
        if !self.environments.contains(&Env::Prod) {
            self.environments.push(Env::Prod);
        }
    }

    /// Enables a running environment [`Env::Custom`].
    #[inline]
    pub fn enable(&mut self, env: &'static str) {
        let custom_env = Env::Custom(env);
        if !self.environments.contains(&custom_env) {
            self.environments.push(custom_env);
        }
    }

    /// Adds a dependency for the plugin.
    #[inline]
    pub fn add_dependency(&mut self, dependency: &'static str) {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
        }
    }

    /// Returns a reference to the shared config corresponding to the plugin.
    #[inline]
    pub fn get_config(&self) -> Option<&'static Table> {
        State::shared()
            .config()
            .get_table("plugins")?
            .get_table(self.name())
    }

    /// Returns the plugin name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the plugin dependencies.
    #[inline]
    pub fn dependencies(&self) -> &[&'static str] {
        self.dependencies.as_slice()
    }

    /// Returns `ture` if the running environment is enabled.
    #[inline]
    pub fn enabled(&self, env: &Env) -> bool {
        let environments = &self.environments;
        if environments.is_empty() {
            true
        } else {
            environments.contains(env)
        }
    }

    /// Loads the plugin.
    #[inline]
    pub async fn load(self) -> Result<(), Error> {
        if let Some(loader) = self.loader {
            loader.await
        } else {
            Ok(())
        }
    }
}
