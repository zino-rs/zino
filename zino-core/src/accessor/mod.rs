//! Unified data access to different storage services.
//!
//! ## Supported storage services
//!
//! | Scheme        | Description                              | Feature flag          |
//! |---------------|------------------------------------------|-----------------------|
//! | `azblob`      | Azure Storage Blob services.             | `accessor-azblob`     |
//! | `azdls`       | Azure Data Lake Storage Gen2 services.   | `accessor-azdls`      |
//! | `cacache`     | Cacache services.                        | `accessor-cacache`    |
//! | `cos`         | Tencent-Cloud COS services.              | `accessor-cos`        |
//! | `dashmap`     | Dashmap backend.                         | `accessor-dashmap`    |
//! | `dropbox`     | Dropbox backend.                         | `accessor-dropbox`    |
//! | `fs`          | POSIX alike file system.                 | `accessor-fs`         |
//! | `ftp`         | FTP and FTPS.                            | `accessor-ftp`        |
//! | `gcs`         | Google Cloud Storage services.           | `accessor-gcs`        |
//! | `gdrive`      | GoogleDrive backend.                     | `accessor-gdrive`     |
//! | `ghac`        | Github Action Cache services.            | `accessor-ghac`       |
//! | `http`        | HTTP Read-only services.                 | `accessor-http`       |
//! | `ipfs`        | InterPlanetary File System HTTP gateway. | `accessor-ipfs`       |
//! | `ipmfs`       | InterPlanetary File System MFS API.      | `accessor-ipmfs`      |
//! | `memcached`   | Memcached services.                      | `accessor-memcached`  |
//! | `memory`      | In memory backend.                       | `accessor-memory`     |
//! | `minio`       | MinIO services.                          | `accessor-s3`         |
//! | `minimoka`    | MiniMoka backend.                        | `accessor-mini-moka`  |
//! | `moka`        | Moka backend.                            | `accessor-moka`       |
//! | `mysql`       | MySQL services.                          | `accessor-mysql`      |
//! | `obs`         | Huawei Cloud Object Storage services.    | `accessor-obs`        |
//! | `onedrive`    | OneDrive backend.                        | `accessor-onedrive`   |
//! | `oss`         | Aliyun Object Storage Service.           | `accessor-oss`        |
//! | `persy`       | Persy services.                          | `accessor-persy`      |
//! | `postgresql`  | PostgreSQL services.                     | `accessor-postgresql` |
//! | `redb`        | Redb services.                           | `accessor-redb`       |
//! | `redis`       | Redis services.                          | `accessor-redis`      |
//! | `s3`          | AWS S3 alike services.                   | `accessor-s3`         |
//! | `sled`        | Sled services.                           | `accessor-sled`       |
//! | `sqlite`      | SQLite services.                         | `accessor-sqlite`     |
//! | `supabase`    | Supabase services.                       | `accessor-supabase`   |
//! | `wasabi`      | Wasabi services.                         | `accessor-wasabi`     |
//! | `webdav`      | WebDAV services.                         | `accessor-webdav`     |
//! | `webhdfs`     | WebHDFS services.                        | `accessor-webhdfs`    |
//!

use crate::{application::StaticRecord, extension::TomlTableExt, state::State};
use opendal::{
    layers::{MetricsLayer, RetryLayer, TracingLayer},
    services, Error,
    ErrorKind::Unsupported,
    Operator,
};
use std::sync::LazyLock;
use toml::Table;

/// Global storage accessor built on the top of [`opendal`](https://crates.io/crates/opendal).
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalAccessor;

impl GlobalAccessor {
    /// Constructs a new operator with the configuration for the specific storage service,
    /// returning an error if it fails.
    pub fn try_new_operator(scheme: &'static str, config: &Table) -> Result<Operator, Error> {
        let operator = match scheme {
            #[cfg(feature = "accessor-azblob")]
            "azblob" => {
                let mut builder = services::Azblob::default();
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
                if let Some(encryption_key) = config.get_str("encryption-key") {
                    builder.encryption_key(encryption_key);
                }
                if let Some(encryption_key_sha256) = config.get_str("encryption-key-sha256") {
                    builder.encryption_key_sha256(encryption_key_sha256);
                }
                if let Some(encryption_algorithm) = config.get_str("encryption-algorithm") {
                    builder.encryption_algorithm(encryption_algorithm);
                }
                if let Some(sas_token) = config.get_str("sas-token") {
                    builder.sas_token(sas_token);
                }
                if let Some(batch_max_operations) = config.get_usize("batch-max-operations") {
                    builder.batch_max_operations(batch_max_operations);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-azdls")]
            "azdls" => {
                let mut builder = services::Azdls::default();
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
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-cacache")]
            "cacache" => {
                let mut builder = services::Cacache::default();
                if let Some(dir) = config.get_str("data-dir") {
                    builder.datadir(dir);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-cos")]
            "cos" => {
                let mut builder = services::Cos::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(secret_id) = config.get_str("secret-id") {
                    builder.secret_id(secret_id);
                }
                if let Some(secret_key) = config.get_str("secret-key") {
                    builder.secret_key(secret_key);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-dashmap")]
            "dashmap" => {
                let mut builder = services::Dashmap::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-dropbox")]
            "dropbox" => {
                let mut builder = services::Dropbox::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(access_token) = config.get_str("access-token") {
                    builder.access_token(access_token);
                }
                if let Some(refresh_token) = config.get_str("refresh-token") {
                    builder.refresh_token(refresh_token);
                }
                if let Some(client_id) = config.get_str("client-id") {
                    builder.client_id(client_id);
                }
                if let Some(client_secret) = config.get_str("client-secret") {
                    builder.client_secret(client_secret);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-fs")]
            "fs" => {
                let mut builder = services::Fs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(atomic_write_dir) = config.get_str("atomic-write-dir") {
                    builder.atomic_write_dir(atomic_write_dir);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-ftp")]
            "ftp" => {
                let mut builder = services::Ftp::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(user) = config.get_str("user") {
                    builder.user(user);
                }
                if let Some(password) = State::decrypt_password(config) {
                    builder.password(password.as_ref());
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-gcs")]
            "gcs" => {
                let mut builder = services::Gcs::default();
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
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-gdrive")]
            "gdrive" => {
                let mut builder = services::Gdrive::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(access_token) = config.get_str("access-token") {
                    builder.access_token(access_token);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-ghac")]
            "ghac" => {
                let mut builder = services::Ghac::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(version) = config.get_str("version") {
                    builder.version(version);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-http")]
            "http" => {
                let mut builder = services::Http::default();
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
                if let Some(token) = config.get_str("token") {
                    builder.token(token);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-ipfs")]
            "ipfs" => {
                let mut builder = services::Ipfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-ipmfs")]
            "ipmfs" => {
                let mut builder = services::Ipmfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-memcached")]
            "memcached" => {
                let mut builder = services::Memcached::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(default_ttl) = config.get_duration("default-ttl") {
                    builder.default_ttl(default_ttl);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-memory")]
            "memory" => {
                let builder = services::Memory::default();
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-mini-moka")]
            "minimoka" => {
                let mut builder = services::MiniMoka::default();
                if let Some(max_capacity) = config.get_u64("max-capacity") {
                    builder.max_capacity(max_capacity);
                }
                if let Some(time_to_live) = config.get_duration("time-to-live") {
                    builder.time_to_live(time_to_live);
                }
                if let Some(time_to_idle) = config.get_duration("time-to-idle") {
                    builder.time_to_idle(time_to_idle);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-moka")]
            "moka" => {
                let mut builder = services::Moka::default();
                if let Some(name) = config.get_str("name") {
                    builder.name(name);
                }
                if let Some(max_capacity) = config.get_u64("max-capacity") {
                    builder.max_capacity(max_capacity);
                }
                if let Some(time_to_live) = config.get_duration("time-to-live") {
                    builder.time_to_live(time_to_live);
                }
                if let Some(time_to_idle) = config.get_duration("time-to-idle") {
                    builder.time_to_idle(time_to_idle);
                }
                if let Some(segments) = config.get_usize("segments") {
                    builder.segments(segments);
                }
                if let Some(thread_pool_enabled) = config.get_bool("thread-pool-enabled") {
                    builder.thread_pool_enabled(thread_pool_enabled);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-mysql")]
            "mysql" => {
                let mut builder = services::Mysql::default();
                if let Some(database) = config.get_str("database") {
                    let authority = State::format_authority(config, Some(3306));
                    let dsn = format!("mysql://{authority}/{database}");
                    builder.connection_string(dsn.as_str());
                }
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(table) = config.get_str("table") {
                    builder.table(table);
                }
                if let Some(key_field) = config.get_str("key-field") {
                    builder.key_field(key_field);
                }
                if let Some(value_field) = config.get_str("value-field") {
                    builder.value_field(value_field);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-obs")]
            "obs" => {
                let mut builder = services::Obs::default();
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
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-onedrive")]
            "onedrive" => {
                let mut builder = services::Onedrive::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(access_token) = config.get_str("access-token") {
                    builder.access_token(access_token);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-oss")]
            "oss" => {
                let mut builder = services::Oss::default();
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
                if let Some(server_side_encryption) = config.get_str("server-side-encryption") {
                    builder.server_side_encryption(server_side_encryption);
                }
                if let Some(encryption_key_id) = config.get_str("server-side-encryption-key-id") {
                    builder.server_side_encryption_key_id(encryption_key_id);
                }
                if let Some(write_min_size) = config.get_usize("write-min-size") {
                    builder.write_min_size(write_min_size);
                }
                if let Some(batch_max_operations) = config.get_usize("batch-max-operations") {
                    builder.batch_max_operations(batch_max_operations);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-persy")]
            "persy" => {
                let mut builder = services::Persy::default();
                if let Some(data_file) = config.get_str("data-file") {
                    builder.datafile(data_file);
                }
                if let Some(segment) = config.get_str("segment") {
                    builder.segment(segment);
                }
                if let Some(index) = config.get_str("index") {
                    builder.index(index);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-postgresql")]
            "postgresql" => {
                let mut builder = services::Postgresql::default();
                if let Some(database) = config.get_str("database") {
                    let authority = State::format_authority(config, Some(5432));
                    let dsn = format!("postgres://{authority}/{database}");
                    builder.connection_string(dsn.as_str());
                }
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(table) = config.get_str("table") {
                    builder.table(table);
                }
                if let Some(key_field) = config.get_str("key-field") {
                    builder.key_field(key_field);
                }
                if let Some(value_field) = config.get_str("value-field") {
                    builder.value_field(value_field);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-redis")]
            "redis" => {
                let mut builder = services::Redis::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(username) = config.get_str("username") {
                    builder.username(username);
                }
                if let Some(password) = State::decrypt_password(config) {
                    builder.password(password.as_ref());
                }
                if let Some(db) = config.get_i64("db") {
                    builder.db(db);
                }
                if let Some(default_ttl) = config.get_duration("default-ttl") {
                    builder.default_ttl(default_ttl);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-redb")]
            "redb" => {
                let mut builder = services::Redb::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(data_dir) = config.get_str("data-dir") {
                    builder.datadir(data_dir);
                }
                if let Some(table) = config.get_str("table") {
                    builder.table(table);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-s3")]
            "s3" | "minio" => {
                let mut builder = services::S3::default();
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
                if let Some(batch_max_operations) = config.get_usize("batch-max-operations") {
                    builder.batch_max_operations(batch_max_operations);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-sled")]
            "sled" => {
                let mut builder = services::Sled::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(dir) = config.get_str("data-dir") {
                    builder.datadir(dir);
                }
                if let Some(tree) = config.get_str("tree") {
                    builder.tree(tree);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-sqlite")]
            "sqlite" => {
                let mut builder = services::Sqlite::default();
                if let Some(database) = config.get_str("database") {
                    let dsn = format!("file://{database}.db");
                    builder.connection_string(dsn.as_str());
                }
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(table) = config.get_str("table") {
                    builder.table(table);
                }
                if let Some(key_field) = config.get_str("key-field") {
                    builder.key_field(key_field);
                }
                if let Some(value_field) = config.get_str("value-field") {
                    builder.value_field(value_field);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-supabase")]
            "supabase" => {
                let mut builder = services::Supabase::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(bucket) = config.get_str("bucket") {
                    builder.bucket(bucket);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(key) = config.get_str("key") {
                    builder.key(key);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-wasabi")]
            "wasabi" => {
                let mut builder = services::Wasabi::default();
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
                if let Some(default_storage_class) = config.get_str("default-storage-class") {
                    builder.default_storage_class(default_storage_class);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-webdav")]
            "webdav" => {
                let mut builder = services::Webdav::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(username) = config.get_str("username") {
                    builder.username(username);
                }
                if let Some(password) = State::decrypt_password(config) {
                    builder.password(password.as_ref());
                }
                if let Some(token) = config.get_str("token") {
                    builder.token(token);
                }
                Ok(Operator::new(builder)?.finish())
            }
            #[cfg(feature = "accessor-webhdfs")]
            "webhdfs" => {
                let mut builder = services::Webhdfs::default();
                if let Some(root) = config.get_str("root") {
                    builder.root(root);
                }
                if let Some(endpoint) = config.get_str("endpoint") {
                    builder.endpoint(endpoint);
                }
                if let Some(delegation) = config.get_str("delegation") {
                    builder.delegation(delegation);
                }
                Ok(Operator::new(builder)?.finish())
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
    pub fn get(name: &str) -> Option<&'static Operator> {
        GLOBAL_ACCESSOR.find(name)
    }
}

/// Global storage accessor.
static GLOBAL_ACCESSOR: LazyLock<StaticRecord<Operator>> = LazyLock::new(|| {
    let mut operators = StaticRecord::new();
    if let Some(accessors) = State::shared().config().get_array("accessor") {
        for accessor in accessors.iter().filter_map(|v| v.as_table()) {
            let scheme = accessor.get_str("scheme").unwrap_or("unkown");
            let name = accessor.get_str("name").unwrap_or(scheme);
            let operator = GlobalAccessor::try_new_operator(scheme, accessor)
                .unwrap_or_else(|err| panic!("fail to build `{scheme}` operator: {err}"));
            operators.push_entry(name, operator);
        }
    }
    operators
});
