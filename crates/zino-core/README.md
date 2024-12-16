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
| `cookie`             | Enables the support for cookies.                       | No       |
| `crypto-sm`          | Enables China's Standards of Encryption Algorithms.    | No       |
| `debug`              | Enables the features for ease of debugging.            | No       |
| `dotenv`             | Enables the configuration loader for a `.env` file.    | No       |
| `env-filter`         | Enables the `env-filter` for [`tracing-subscriber`].   | No       |
| `http-client`        | Enables the HTTP client via [`reqwest`].               | No       |
| `locale`             | Enables the support for locale related utilities.      | No       |
| `metrics`            | Enables the [`metrics`] exporter.                      | No       |
| `oidc`               | Enables the support for OIDC via [`rauthy`].           | No       |
| `runtime-async-std`  | Enables the [`async-std`] runtime.                     | No       |
| `runtime-tokio`      | Enables the [`tokio`] runtime.                         | No       |
| `sentry`             | Enables the integration with [`sentry`].               | No       |
| `tls-native`         | Enables the [`native-tls`] TLS backend.                | No       |
| `tls-rustls`         | Enables the [`rustls`] TLS backend.                    | No       |
| `tracing-log`        | Enables the `tracing-log` for [`tracing-subscriber`].  | No       |
| `tracing-subscriber` | Enables the integration with [`tracing-subscriber`].   | No       |
| `validator`          | Enables the common validation rules.                   | No       |

[`zino`]: https://github.com/zino-rs/zino
[`tracing-subscriber`]: https://crates.io/crates/tracing-subscriber
[`reqwest`]: https://crates.io/crates/reqwest
[`metrics`]: https://crates.io/crates/metrics
[`async-std`]: https://crates.io/crates/async-std
[`tokio`]: https://crates.io/crates/tokio
[`native-tls`]: https://crates.io/crates/native-tls
[`rauthy`]: https://crates.io/crates/rauthy-client
[`rustls`]: https://crates.io/crates/rustls
[`sentry`]: https://crates.io/crates/sentry
