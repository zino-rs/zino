[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-core)
[![docs-rs]](https://docs.rs/zino-core)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Core types and traits for [`zino`].

## Feature flags

The following optional features are available:

| Name                 | Description                                            | Default? |
|----------------------|--------------------------------------------------------|----------|
| `accessor`           | Enables the data access layer built with [`opendal`].  | No       |
| `chatbot`            | Enables the chatbot services.                          | No       |
| `connector`          | Enables the data source connectors.                    | No       |
| `cookie`             | Enables the support for cookies.                       | No       |
| `crypto-sm`          | Enables China's Standards of Encryption Algorithms.    | No       |
| `dotenv`             | Enables the configuration loader for a `.env` file.    | No       |
| `env-filter`         | Enables the `env-filter` for [`tracing-subscriber`].   | No       |
| `flume`              | Enables the message channel via [`flume`].             | No       |
| `i18n`               | Enables the support for internationalization.          | No       |
| `jwt`                | Enables the support for JSON Web Token.                | No       |
| `locale`             | Enables the support for locale related utilities.      | No       |
| `metrics`            | Enables the [`metrics`] exporter.                      | No       |
| `oidc`               | Enables the support for OIDC via [`rauthy`].           | No       |
| `opa`                | Enables the support for OPA via [`regorus`].           | No       |
| `openapi`            | Enables the support for OpenAPI docs via [`utoipa`].   | No       |
| `orm`                | Enables the ORM for MySQL, PostgreSQL or **SQLite**.   | No       |
| `runtime-async-std`  | Enables the [`async-std`] runtime.                     | No       |
| `runtime-tokio`      | Enables the [`tokio`] runtime.                         | No       |
| `sentry`             | Enables the integration with [`sentry`].               | No       |
| `sqids`              | Enables the support for [`sqids`].                     | No       |
| `tls-native`         | Enables the [`native-tls`] TLS backend.                | No       |
| `tls-rustls`         | Enables the [`rustls`] TLS backend.                    | No       |
| `tracing-log`        | Enables the `tracing-log` for [`tracing-subscriber`].  | No       |
| `tracing-subscriber` | Enables the integration with [`tracing-subscriber`].   | No       |
| `validator`          | Enables the common validation rules.                   | No       |
| `view`               | Enables the HTML template rendering.                   | No       |

[`zino`]: https://github.com/zino-rs/zino
[`opendal`]: https://crates.io/crates/opendal
[`tracing-subscriber`]: https://crates.io/crates/tracing-subscriber
[`flume`]: https://crates.io/crates/flume
[`metrics`]: https://crates.io/crates/metrics
[`async-std`]: https://crates.io/crates/async-std
[`tokio`]: https://crates.io/crates/tokio
[`native-tls`]: https://crates.io/crates/native-tls
[`rauthy`]: https://crates.io/crates/rauthy-client
[`regorus`]: https://crates.io/crates/regorus
[`utoipa`]: https://crates.io/crates/utoipa
[`rustls`]: https://crates.io/crates/rustls
[`sentry`]: https://crates.io/crates/sentry
[`sqids`]: https://crates.io/crates/sqids
