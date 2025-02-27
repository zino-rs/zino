[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-auth)
[![docs-rs]](https://docs.rs/zino-auth)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Authentication and authorization for [`zino`].

[`zino`]: https://github.com/zino-rs/zino

## Feature flags

The following optional features are available:

| Name                 | Description                                            | Default? |
|----------------------|--------------------------------------------------------|----------|
| `crypto-sm`          | Enables China's Standards of Encryption Algorithms.    | No       |
| `jwt`                | Enables the support for JSON Web Token.                | No       |
| `oidc`               | Enables the support for OIDC via [`rauthy`].           | No       |
| `opa`                | Enables the support for OPA via [`regorus`].           | No       |
| `sqids`              | Enables the support for [`sqids`].                     | No       |

[`rauthy`]: https://crates.io/crates/rauthy-client
[`regorus`]: https://crates.io/crates/regorus
[`sqids`]: https://crates.io/crates/sqids

