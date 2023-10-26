[![github]](https://github.com/photino/zino)
[![crates-io]](https://crates.io/crates/zino-core)
[![docs-rs]](https://docs.rs/zino-core)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Core types and traits for [`zino`].

## Feature flags

The following optional features are available:

| Name                | Description                                            | Default? |
|---------------------|--------------------------------------------------------|----------|
| `accessor`          | Enables the data access layer built with [`opendal`].  | No       |
| `chatbot`           | Enables the chatbot services.                          | No       |
| `connector`         | Enables the data source connectors.                    | No       |
| `crypto-sm`         | Enables China's Standards of Encryption Algorithms.    | No       |
| `orm`               | Enables the ORM for MySQL, PostgreSQL or **SQLite**.   | No       |
| `runtime-async-std` | Enables the [`async-std`] runtime.                     | No       |
| `runtime-tokio`     | Enables the [`tokio`] runtime.                         | Yes      |
| `tls-native`        | Enables the [`native-tls`] TLS backend.                | No       |
| `tls-rustls`        | Enables the [`rustls`] TLS backend.                    | Yes      |
| `view`              | Enables the HTML template rendering.                   | No       |

[`zino`]: https://github.com/photino/zino
[`opendal`]: https://crates.io/crates/opendal
[`async-std`]: https://crates.io/crates/async-std
[`tokio`]: https://crates.io/crates/tokio
[`native-tls`]: https://crates.io/crates/native-tls
[`rustls`]: https://crates.io/crates/rustls
