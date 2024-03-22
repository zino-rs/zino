use casbin::prelude::*;
use std::sync::OnceLock;
use zino::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Casbin;

impl Casbin {
    pub fn init() -> Plugin {
        let loader = Box::pin(async {
            let model_file = "./config/casbin/model.conf";
            let policy_file = "./config/casbin/policy.csv";
            let enforcer = Enforcer::new(model_file, policy_file).await?;
            if CASBIN_ENFORCER.set(enforcer).is_err() {
                tracing::error!("fail to initialize the Casbin enforcer");
            }
            Ok(())
        });
        let mut plugin = Plugin::new("casbin");
        plugin.add_dependency("foo");
        plugin.set_loader(loader);
        plugin
    }
}

static CASBIN_ENFORCER: OnceLock<Enforcer> = OnceLock::new();
