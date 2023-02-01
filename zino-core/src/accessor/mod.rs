//! Unified data access to different storage services.

use crate::{extend::TomlTableExt, state::State};
use backon::ExponentialBackoff;
use opendal::{
    layers::{MetricsLayer, RetryLayer, TracingLayer},
    services::{
        azblob, azdfs, fs, ftp, gcs, ghac, ipfs, ipmfs, memcached, memory, moka, obs, oss, redis,
        s3, webdav,
    },
    Error,
    ErrorKind::{Unexpected, Unsupported},
    Operator,
};
use std::time::Duration;

/// Storage accessor built on the top of [`opendal`](https://crates.io/crates/opendal).
#[derive(Debug)]
pub struct StorageAccessor {}

impl StorageAccessor {
    /// Creates a new operator for the specific storage service.
    pub fn new_operator(
        scheme: &'static str,
        name: Option<&'static str>,
    ) -> Result<Operator, Error> {
        let config = State::shared().config();
        let operator = if scheme == "memory" {
            let mut builder = memory::Builder::default();
            Ok(Operator::new(builder.build()?))
        } else if let Some(accessors) = config.get_array("accessor") {
            if let Some(accessor) = accessors
                .iter()
                .filter_map(|v| v.as_table())
                .find(|t| t.get_str("scheme").contains(&scheme) && t.get_str("name") == name)
            {
                match scheme {
                    "azblob" => {
                        let mut builder = azblob::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(container) = accessor.get_str("container") {
                            builder.container(container);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(account_name) = accessor.get_str("account-name") {
                            builder.account_name(account_name);
                        }
                        if let Some(account_key) = accessor.get_str("account-key") {
                            builder.account_key(account_key);
                        }
                        if let Some(sas_token) = accessor.get_str("sas-token") {
                            builder.sas_token(sas_token);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "azdfs" => {
                        let mut builder = azdfs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(filesystem) = accessor.get_str("filesystem") {
                            builder.filesystem(filesystem);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(account_name) = accessor.get_str("account-name") {
                            builder.account_name(account_name);
                        }
                        if let Some(account_key) = accessor.get_str("account-key") {
                            builder.account_key(account_key);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "fs" => {
                        let mut builder = fs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(atomic_write_dir) = accessor.get_str("atomic-write-dir") {
                            builder.atomic_write_dir(atomic_write_dir);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ftp" => {
                        let mut builder = ftp::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(user) = accessor.get_str("user") {
                            builder.user(user);
                        }
                        if let Some(password) = accessor.get_str("password") {
                            builder.password(password);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "gcs" => {
                        let mut builder = gcs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get_str("bucket") {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(service_account) = accessor.get_str("service-account") {
                            builder.service_account(service_account);
                        }
                        if let Some(credential) = accessor.get_str("credential") {
                            builder.credential(credential);
                        }
                        if let Some(credential_path) = accessor.get_str("credential-path") {
                            builder.credential_path(credential_path);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ghac" => {
                        let mut builder = ghac::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(version) = accessor.get_str("version") {
                            builder.version(version);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ipfs" => {
                        let mut builder = ipfs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ipmfs" => {
                        let mut builder = ipmfs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "memcached" => {
                        let mut builder = memcached::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(default_ttl) = accessor.get_u64("default-ttl") {
                            builder.default_ttl(Duration::from_secs(default_ttl));
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "moka" => {
                        let mut builder = moka::Builder::default();
                        if let Some(name) = accessor.get_str("name") {
                            builder.name(name);
                        }
                        if let Some(max_capacity) = accessor.get_u64("max-capacity") {
                            builder.max_capacity(max_capacity);
                        }
                        if let Some(time_to_live) = accessor.get_u64("time-to-live") {
                            builder.time_to_live(Duration::from_secs(time_to_live));
                        }
                        if let Some(time_to_idle) = accessor.get_u64("time-to-idle") {
                            builder.time_to_idle(Duration::from_secs(time_to_idle));
                        }
                        if let Some(segments) = accessor.get_usize("segments") {
                            builder.segments(segments);
                        }
                        if let Some(thread_pool_enabled) = accessor.get_bool("thread-pool-enabled")
                        {
                            builder.thread_pool_enabled(thread_pool_enabled);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "obs" => {
                        let mut builder = obs::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get_str("bucket") {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(access_key_id) = accessor.get_str("access-key-id") {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(secret_access_key) = accessor.get_str("secret_access_key") {
                            builder.secret_access_key(secret_access_key);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "oss" => {
                        let mut builder = oss::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get_str("bucket") {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(presign_endpoint) = accessor.get_str("presign-endpoint") {
                            builder.presign_endpoint(presign_endpoint);
                        }
                        if let Some(access_key_id) = accessor.get_str("access-key-id") {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(access_key_secret) = accessor.get_str("access-key-secret") {
                            builder.access_key_secret(access_key_secret);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "redis" => {
                        let mut builder = redis::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(username) = accessor.get_str("username") {
                            builder.username(username);
                        }
                        if let Some(password) = accessor.get_str("password") {
                            builder.password(password);
                        }
                        if let Some(db) = accessor.get_i64("db") {
                            builder.db(db);
                        }
                        if let Some(default_ttl) = accessor.get_u64("default-ttl") {
                            builder.default_ttl(Duration::from_secs(default_ttl));
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "s3" => {
                        let mut builder = s3::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get_str("bucket") {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        if let Some(region) = accessor.get_str("region") {
                            builder.region(region);
                        }
                        if let Some(access_key_id) = accessor.get_str("access-key-id") {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(secret_access_key) = accessor.get_str("secret-access-key") {
                            builder.secret_access_key(secret_access_key);
                        }
                        if let Some(role_arn) = accessor.get_str("role-arn") {
                            builder.role_arn(role_arn);
                        }
                        if let Some(external_id) = accessor.get_str("external-id") {
                            builder.external_id(external_id);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "webdav" => {
                        let mut builder = webdav::Builder::default();
                        if let Some(root) = accessor.get_str("root") {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get_str("endpoint") {
                            builder.endpoint(endpoint);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    _ => Err(Error::new(Unsupported, "scheme is unsupported")),
                }
            } else {
                Err(Error::new(Unexpected, "failed to find the storage service"))
            }
        } else if name.is_none() {
            scheme.parse().and_then(Operator::from_env)
        } else {
            Err(Error::new(Unexpected, "failed to create the operator"))
        };
        operator.map(|op| {
            op.layer(TracingLayer)
                .layer(MetricsLayer)
                .layer(RetryLayer::new(ExponentialBackoff::default()))
        })
    }
}
