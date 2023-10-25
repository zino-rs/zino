# zino-core

Core types and traits for zino.

## Feature flags

The following optional features are available:

| Name                | Description                                            | Default? |
|---------------------|--------------------------------------------------------|----------|
| `accessor`          | Enables the data access layer built with [`opendal`].  | No       |
| `cache`             | Enables the cache services.                            | No       |
| `chatbot`           | Enables the chatbot services.                          | No       |
| `connector`         | Enables the data source connectors.                    | No       |
| `crypto-sm`         | Enables China's Standards of Encryption Algorithms.    | No       |
| `format`            | Enables the support for common file formats.           | No       |
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
