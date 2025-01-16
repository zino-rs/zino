use simple_ldap::pool::{LdapConfig, LdapPool};
use std::sync::OnceLock;
use zino_core::{
    application::{Agent, Application, Plugin},
    bail,
    error::Error,
    extension::TomlTableExt,
    state::State,
};

/// The LDAP client.
#[derive(Debug, Clone, Copy)]
pub struct LdapClient;

impl LdapClient {
    /// Initializes the LDAP pool.
    pub fn init() -> Plugin {
        let loader = Box::pin(async {
            let Some(config) = Agent::config().get_table("ldap") else {
                bail!("`ldap` config should be specified");
            };
            let Some(url) = config.get_str("url") else {
                bail!("`ldap.url` should be specified");
            };
            let Some(account) = config.get_str("account") else {
                bail!("`ldap.account` should be specified");
            };
            let Some(password) = State::decrypt_password(config) else {
                bail!("`ldap.password` should be specified");
            };
            let ldap_config = LdapConfig {
                ldap_url: url.to_owned(),
                bind_dn: account.to_owned(),
                bind_pw: password.into_owned(),
                pool_size: config.get_usize("pool-size").unwrap_or(10),
                dn_attribute: config.get_str("attribute").map(|s| s.to_owned()),
            };
            let ldap_pool = simple_ldap::pool::build_connection_pool(&ldap_config).await;
            if LDAP_POOL.set(ldap_pool).is_err() {
                tracing::error!("fail to initialize the LDAP pool");
            }
            Ok(())
        });
        Plugin::with_loader("ldap-client", loader)
    }

    /// Returns an existing LDAP connection from the pool or creates a new one if required.
    #[inline]
    pub async fn get_connection() -> Result<simple_ldap::LdapClient, simple_ldap::Error> {
        if let Some(pool) = LDAP_POOL.get() {
            pool.get_connection().await
        } else {
            Err(simple_ldap::Error::NotFound(
                "LDAP pool is not initialized".to_owned(),
            ))
        }
    }
}

/// Shared LDAP pool.
static LDAP_POOL: OnceLock<LdapPool> = OnceLock::new();
