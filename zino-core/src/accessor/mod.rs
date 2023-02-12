//! Unified data access to different storage services.
//!
//! Supported storage services:
//! - `azblob`: Azure Storage Blob services.
//! - `azdfs`: Azure Data Lake Storage Gen2 services.
//! - `fs`: POSIX alike file system.
//! - `ftp`: FTP and FTPS support.
//! - `gcs`: Google Cloud Storage Service.
//! - `ghac`: Github Action Cache Service.
//! - `ipfs`: InterPlanetary File System HTTP Gateway support.
//! - `ipmfs`: InterPlanetary File System MFS API support.
//! - `memcached`: Memcached service support.
//! - `memory`: In memory backend.
//! - `minio`: MinIO services support.
//! - `moka`: Moka backend support.
//! - `obs`: Huawei Cloud Object Storage Service (OBS).
//! - `oss`: Aliyun Object Storage Service (OSS).
//! - `redis`: Redis services support.
//! - `s3`: AWS S3 alike services.
//! - `webdav`: WebDAV Service Support.
//! - `webhdfs`: WebHDFS Service Support.
//!

use crate::{extend::TomlTableExt, state::State};
use opendal::{
    layers::{MetricsLayer, RetryLayer, TracingLayer},
    services::{
        Azblob, Azdfs, Fs, Ftp, Gcs, Ghac, Ipfs, Ipmfs, Memcached, Memory, Moka, Obs, Oss, Redis,
        Webdav, Webhdfs, S3,
    },
    Builder, Error,
    ErrorKind::Unsupported,
    Operator,
};
use std::{collections::HashMap, sync::LazyLock, time::Duration};
use toml::Table;

/// Global storage accessor built on the top of [`opendal`](https://crates.io/crates/opendal).
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalAccessor;

impl GlobalAccessor {
    /// Creates a new operator with the configuration for the specific storage service.
    pub fn new_operator(scheme: &'static str, config: &Table) -> Result<Operator, Error> {
        let operator = match scheme {
            "azblob" => {
                let mut builder = Azblob::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(container) = config.get_str("container") {
                    builder.container(container);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(account_name) = config.get_str("account-name") {
                    builder.account_name(account_name);
                }
                if let Some(account_key) = config.get_str("account-key") {
                    builder.account_key(account_key);
                }
                if let Some(sas_token) = config.get_str("sas-token") {
                    builder.sas_token(sas_token);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "azdfs" => {
                let mut builder = Azdfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(filesystem) = config.get_str("filesystem") {
                    builder.filesystem(filesystem);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(account_name) = config.get_str("account-name") {
                    builder.account_name(account_name);
                }
                if let Some(account_key) = config.get_str("account-key") {
                    builder.account_key(account_key);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "fs" => {
                let mut builder = Fs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(atomic_write_dir) = config.get_str("atomic-write-dir") {
                    builder.atomic_write_dir(atomic_write_dir);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "ftp" => {
                let mut builder = Ftp::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(user) = config.get_str("user") {
                    builder.user(user);
                }
                if let Some(password) = config.get_str("password") {
                    builder.password(password);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "gcs" => {
                let mut builder = Gcs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(service_account) = config.get_str("service-account") {
                    builder.service_account(service_account);
                }
                if let Some(credential) = config.get_str("credential") {
                    builder.credential(credential);
                }
                if let Some(credential_path) = config.get_str("credential-path") {
                    builder.credential_path(credential_path);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "ghac" => {
                let mut builder = Ghac::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(version) = config.get_str("version") {
                    builder.version(version);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "ipfs" => {
                let mut builder = Ipfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "ipmfs" => {
                let mut builder = Ipmfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "memcached" => {
                let mut builder = Memcached::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(default_ttl) = config.get_u64("default-ttl") {
                    builder.default_ttl(Duration::from_secs(default_ttl));
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "memory" => {
                let mut builder = Memory::default();
                Ok(Operator::new(builder.build()?).finish())
            }
            "moka" => {
                let mut builder = Moka::default();
                if let Some(name) = config.get_str("name") {
                    builder.name(name);
                }
                if let Some(max_capacity) = config.get_u64("max-capacity") {
                    builder.max_capacity(max_capacity);
                }
                if let Some(time_to_live) = config.get_u64("time-to-live") {
                    builder.time_to_live(Duration::from_secs(time_to_live));
                }
                if let Some(time_to_idle) = config.get_u64("time-to-idle") {
                    builder.time_to_idle(Duration::from_secs(time_to_idle));
                }
                if let Some(segments) = config.get_usize("segments") {
                    builder.segments(segments);
                }
                if let Some(thread_pool_enabled) = config.get_bool("thread-pool-enabled") {
                    builder.thread_pool_enabled(thread_pool_enabled);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "obs" => {
                let mut builder = Obs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(access_key_id) = config.get_str("access-key-id") {
                    builder.access_key_id(access_key_id);
                }
                if let Some(secret_access_key) = config.get_str("secret_access_key") {
                    builder.secret_access_key(secret_access_key);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "oss" => {
                let mut builder = Oss::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(presign_endpoint) = config.get_str("presign-endpoint") {
                    builder.presign_endpoint(presign_endpoint);
                }
                if let Some(access_key_id) = config.get_str("access-key-id") {
                    builder.access_key_id(access_key_id);
                }
                if let Some(access_key_secret) = config.get_str("access-key-secret") {
                    builder.access_key_secret(access_key_secret);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "redis" => {
                let mut builder = Redis::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(username) = config.get_str("username") {
                    builder.username(username);
                }
                if let Some(password) = config.get_str("password") {
                    builder.password(password);
                }
                if let Some(db) = config.get_i64("db") {
                    builder.db(db);
                }
                if let Some(default_ttl) = config.get_u64("default-ttl") {
                    builder.default_ttl(Duration::from_secs(default_ttl));
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "s3" | "minio" => {
                let mut builder = S3::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(region) = config.get_str("region") {
                    builder.region(region);
                }
                if let Some(access_key_id) = config.get_str("access-key-id") {
                    builder.access_key_id(access_key_id);
                }
                if let Some(secret_access_key) = config.get_str("secret-access-key") {
                    builder.secret_access_key(secret_access_key);
                }
                if let Some(role_arn) = config.get_str("role-arn") {
                    builder.role_arn(role_arn);
                }
                if let Some(external_id) = config.get_str("external-id") {
                    builder.external_id(external_id);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "webdav" => {
                let mut builder = Webdav::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            "webhdfs" => {
                let mut builder = Webhdfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(delegation) = config.get_str("delegation") {
                    builder.delegation(delegation);
                }
                Ok(Operator::new(builder.build()?).finish())
            }
            _ => Err(Error::new(Unsupported, "scheme is unsupported")),
        };
        operator.map(|op| {
            op.layer(TracingLayer)
                .layer(MetricsLayer)
                .layer(RetryLayer::new())
        })
    }

    /// Gets the operator for the specific storage service.
    #[inline]
    pub fn get(name: &'static str) -> Option<&'static Operator> {
        GLOBAL_ACCESSOR.get(name)
    }
}

/// Global storage accessor.
static GLOBAL_ACCESSOR: LazyLock<HashMap<&'static str, Operator>> = LazyLock::new(|| {
    let mut operators = HashMap::new();
    let memory_accessor = Memory::default()
        .build()
        .expect("failed to build memory accessor");
    let memory_operator = Operator::new(memory_accessor)
        .layer(TracingLayer)
        .layer(MetricsLayer)
        .layer(RetryLayer::new())
        .finish();
    operators.insert("memory", memory_operator);

    if let Some(accessors) = State::shared().config().get_array("accessor") {
        for accessor in accessors.iter().filter_map(|v| v.as_table()) {
            let scheme = accessor.get_str("scheme").unwrap_or("unkown");
            let name = accessor.get_str("name").unwrap_or(scheme);
            let operator = GlobalAccessor::new_operator(scheme, accessor)
                .unwrap_or_else(|err| panic!("failed to build `{scheme}` operator: {err}"));
            operators.insert(name, operator);
        }
    }
    operators
});
