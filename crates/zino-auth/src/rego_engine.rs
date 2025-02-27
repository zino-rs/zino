use parking_lot::Mutex;
use regorus::{Engine, Value};
use std::{fs, io::ErrorKind};
use zino_core::{
    LazyLock,
    application::{Agent, Application},
    error::Error,
};

/// Rego evaluation engine.
pub struct RegoEngine {
    /// The engine.
    engine: Mutex<Engine>,
}

impl RegoEngine {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(Engine::default()),
        }
    }

    /// Adds a policy.
    #[inline]
    pub fn add_policy(
        &self,
        path: impl Into<String>,
        rego: impl Into<String>,
    ) -> Result<String, Error> {
        self.engine
            .lock()
            .add_policy(path.into(), rego.into())
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Adds the data document.
    #[inline]
    pub fn add_data(&self, value: impl Into<Value>) -> Result<(), Error> {
        self.engine
            .lock()
            .add_data(value.into())
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Adds the data document in the JSON format.
    #[inline]
    pub fn add_data_json(&self, data_json: &str) -> Result<(), Error> {
        self.engine
            .lock()
            .add_data_json(data_json)
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Clears the data document.
    #[inline]
    pub fn clear_data(&self) {
        self.engine.lock().clear_data()
    }

    /// Sets the input document.
    #[inline]
    pub fn set_input(&self, input: impl Into<Value>) {
        self.engine.lock().set_input(input.into())
    }

    /// Sets the input document in the JSON format.
    #[inline]
    pub fn set_input_json(&self, input_json: &str) -> Result<(), Error> {
        self.engine
            .lock()
            .set_input_json(input_json)
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Evaluates a rule at the given path.
    #[inline]
    pub fn eval_rule(&self, path: impl Into<String>) -> Result<Value, Error> {
        self.engine
            .lock()
            .eval_rule(path.into())
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Evaluates a Rego query that produces a boolean value.
    #[inline]
    pub fn eval_bool_query(&self, query: impl Into<String>) -> Result<bool, Error> {
        self.engine
            .lock()
            .eval_bool_query(query.into(), false)
            .map_err(|err| Error::new(err.to_string()))
    }

    /// Evaluates an `allow` query.
    #[inline]
    pub fn eval_allow_query(&self, query: impl Into<String>) -> bool {
        self.engine.lock().eval_allow_query(query.into(), false)
    }

    /// Evaluates a `deny` query.
    #[inline]
    pub fn eval_deny_query(&self, query: impl Into<String>) -> bool {
        self.engine.lock().eval_deny_query(query.into(), false)
    }

    /// Returns a reference to the shared Rego engine.
    #[inline]
    pub fn shared() -> &'static Self {
        &SHARED_REGO_ENGINE
    }
}

/// Shared Rego evaluation engine.
static SHARED_REGO_ENGINE: LazyLock<RegoEngine> = LazyLock::new(|| {
    let engine = RegoEngine::new();
    let opa_dir = Agent::config_dir().join("opa");
    match fs::read_dir(opa_dir) {
        Ok(entries) => {
            let files = entries.filter_map(|entry| entry.ok());
            for file in files {
                let opa_file = file.path();
                if opa_file.extension().is_some_and(|ext| ext == "rego") {
                    let opa_policy = fs::read_to_string(&opa_file).unwrap_or_else(|err| {
                        let opa_file = opa_file.display();
                        panic!("fail to read the policy file `{opa_file}`: {err}");
                    });
                    let file_name = opa_file
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    engine
                        .add_policy(file_name, opa_policy)
                        .unwrap_or_else(|err| {
                            let opa_file = opa_file.display();
                            panic!("fail to read the policy file `{opa_file}`: {err}");
                        });
                } else {
                    let opa_data = fs::read_to_string(&opa_file).unwrap_or_else(|err| {
                        let opa_file = opa_file.display();
                        panic!("fail to read the data file `{opa_file}`: {err}");
                    });
                    engine
                        .add_data_json(&opa_data)
                        .expect("fail to add the data document for the OPA");
                }
            }
        }
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                tracing::error!("{err}");
            }
        }
    }
    engine
});
