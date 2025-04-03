[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-http)
[![docs-rs]](https://docs.rs/zino-http)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

HTTP requests and responses for [`zino`].

[`zino`]: https://github.com/zino-rs/zino

## Feature flags

The following optional features are available:

| Name                 | Description                                            | Default? |
|----------------------|--------------------------------------------------------|----------|
| `auth`               | Enables the authentication and authorization.          | No       |
| `cookie`             | Enables the support for cookies.                       | No       |
| `debug`              | Enables the features for ease of debugging.            | No       |
| `i18n`               | Enables the support for internationalization.          | No       |
| `inertia`            | Enables the support for the Inertia protocol.          | No       |
| `jwt`                | Enables the support for JSON Web Token.                | No       |
| `metrics`            | Enables the [`metrics`] exporter.                      | No       |
| `view`               | Enables the HTML template rendering.                   | No       |

[`metrics`]: https://crates.io/crates/metrics
