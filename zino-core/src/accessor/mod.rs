//! Unified data access to different storage services.

use crate::state::State;
use backon::ExponentialBackoff;
use opendal::{
    layers::{MetricsLayer, RetryLayer, TracingLayer},
    services::{
        azblob, azdfs, fs, ftp, gcs, ghac, ipfs, ipmfs, memcached, memory, moka, obs, oss, redis,
        s3,
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
        } else if let Some(accessors) = config.get("accessor").and_then(|v| v.as_array()) {
            if let Some(accessor) = accessors.iter().filter_map(|v| v.as_table()).find(|t| {
                t.get("scheme").and_then(|v| v.as_str()).contains(&scheme)
                    && t.get("name").and_then(|v| v.as_str()) == name
            }) {
                match scheme {
                    "azblob" => {
                        let mut builder = azblob::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(container) = accessor.get("container").and_then(|v| v.as_str())
                        {
                            builder.container(container);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(account_name) =
                            accessor.get("account-name").and_then(|v| v.as_str())
                        {
                            builder.account_name(account_name);
                        }
                        if let Some(account_key) =
                            accessor.get("account-key").and_then(|v| v.as_str())
                        {
                            builder.account_key(account_key);
                        }
                        if let Some(sas_token) = accessor.get("sas-token").and_then(|v| v.as_str())
                        {
                            builder.sas_token(sas_token);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "azdfs" => {
                        let mut builder = azdfs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(filesystem) =
                            accessor.get("filesystem").and_then(|v| v.as_str())
                        {
                            builder.filesystem(filesystem);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(account_name) =
                            accessor.get("account-name").and_then(|v| v.as_str())
                        {
                            builder.account_name(account_name);
                        }
                        if let Some(account_key) =
                            accessor.get("account-key").and_then(|v| v.as_str())
                        {
                            builder.account_key(account_key);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "fs" => {
                        let mut builder = fs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(atomic_write_dir) =
                            accessor.get("atomic-write-dir").and_then(|v| v.as_str())
                        {
                            builder.atomic_write_dir(atomic_write_dir);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ftp" => {
                        let mut builder = ftp::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(user) = accessor.get("user").and_then(|v| v.as_str()) {
                            builder.user(user);
                        }
                        if let Some(password) = accessor.get("password").and_then(|v| v.as_str()) {
                            builder.password(password);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "gcs" => {
                        let mut builder = gcs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get("bucket").and_then(|v| v.as_str()) {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(service_account) =
                            accessor.get("service-account").and_then(|v| v.as_str())
                        {
                            builder.service_account(service_account);
                        }
                        if let Some(credential) =
                            accessor.get("credential").and_then(|v| v.as_str())
                        {
                            builder.credential(credential);
                        }
                        if let Some(credential_path) =
                            accessor.get("credential-path").and_then(|v| v.as_str())
                        {
                            builder.credential_path(credential_path);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ghac" => {
                        let mut builder = ghac::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(version) = accessor.get("version").and_then(|v| v.as_str()) {
                            builder.version(version);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ipfs" => {
                        let mut builder = ipfs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "ipmfs" => {
                        let mut builder = ipmfs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "memcached" => {
                        let mut builder = memcached::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(default_ttl) = accessor
                            .get("default-ttl")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| u64::try_from(i).ok())
                        {
                            builder.default_ttl(Duration::from_secs(default_ttl));
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "moka" => {
                        let mut builder = moka::Builder::default();
                        if let Some(name) = accessor.get("name").and_then(|v| v.as_str()) {
                            builder.name(name);
                        }
                        if let Some(max_capacity) = accessor
                            .get("max-capacity")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| u64::try_from(i).ok())
                        {
                            builder.max_capacity(max_capacity);
                        }
                        if let Some(time_to_live) = accessor
                            .get("time-to-live")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| u64::try_from(i).ok())
                        {
                            builder.time_to_live(Duration::from_secs(time_to_live));
                        }
                        if let Some(time_to_idle) = accessor
                            .get("time-to-idle")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| u64::try_from(i).ok())
                        {
                            builder.time_to_idle(Duration::from_secs(time_to_idle));
                        }
                        if let Some(segments) = accessor
                            .get("segments")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| usize::try_from(i).ok())
                        {
                            builder.segments(segments);
                        }
                        if let Some(thread_pool_enabled) = accessor
                            .get("thread-pool-enabled")
                            .and_then(|v| v.as_bool())
                        {
                            builder.thread_pool_enabled(thread_pool_enabled);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "obs" => {
                        let mut builder = obs::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get("bucket").and_then(|v| v.as_str()) {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(access_key_id) =
                            accessor.get("access-key-id").and_then(|v| v.as_str())
                        {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(secret_access_key) =
                            accessor.get("secret_access_key").and_then(|v| v.as_str())
                        {
                            builder.secret_access_key(secret_access_key);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "oss" => {
                        let mut builder = oss::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get("bucket").and_then(|v| v.as_str()) {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(presign_endpoint) =
                            accessor.get("presign-endpoint").and_then(|v| v.as_str())
                        {
                            builder.presign_endpoint(presign_endpoint);
                        }
                        if let Some(access_key_id) =
                            accessor.get("access-key-id").and_then(|v| v.as_str())
                        {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(access_key_secret) =
                            accessor.get("access-key-secret").and_then(|v| v.as_str())
                        {
                            builder.access_key_secret(access_key_secret);
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "redis" => {
                        let mut builder = redis::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(username) = accessor.get("username").and_then(|v| v.as_str()) {
                            builder.username(username);
                        }
                        if let Some(password) = accessor.get("password").and_then(|v| v.as_str()) {
                            builder.password(password);
                        }
                        if let Some(db) = accessor.get("db").and_then(|v| v.as_integer()) {
                            builder.db(db);
                        }
                        if let Some(default_ttl) = accessor
                            .get("default-ttl")
                            .and_then(|v| v.as_integer())
                            .and_then(|i| u64::try_from(i).ok())
                        {
                            builder.default_ttl(Duration::from_secs(default_ttl));
                        }
                        Ok(Operator::new(builder.build()?))
                    }
                    "s3" => {
                        let mut builder = s3::Builder::default();
                        if let Some(root) = accessor.get("root").and_then(|v| v.as_str()) {
                            builder.root(root);
                        }
                        if let Some(bucket) = accessor.get("bucket").and_then(|v| v.as_str()) {
                            builder.bucket(bucket);
                        }
                        if let Some(endpoint) = accessor.get("endpoint").and_then(|v| v.as_str()) {
                            builder.endpoint(endpoint);
                        }
                        if let Some(region) = accessor.get("region").and_then(|v| v.as_str()) {
                            builder.region(region);
                        }
                        if let Some(access_key_id) =
                            accessor.get("access-key-id").and_then(|v| v.as_str())
                        {
                            builder.access_key_id(access_key_id);
                        }
                        if let Some(secret_access_key) =
                            accessor.get("secret-access-key").and_then(|v| v.as_str())
                        {
                            builder.secret_access_key(secret_access_key);
                        }
                        if let Some(role_arn) = accessor.get("role-arn").and_then(|v| v.as_str()) {
                            builder.role_arn(role_arn);
                        }
                        if let Some(external_id) =
                            accessor.get("external-id").and_then(|v| v.as_str())
                        {
                            builder.external_id(external_id);
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
