[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-storage)
[![docs-rs]](https://docs.rs/zino-storage)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Files and storage services for [`zino`].

## Feature flags

The following optional features are available:

| Name                 | Description                                            | Default? |
|----------------------|--------------------------------------------------------|----------|
| `accessor`           | Enables the data access layer built with [`opendal`].  | No       |
| `http-client`        | Enables the HTTP client via [`reqwest`].               | No       |

[`zino`]: https://github.com/zino-rs/zino
[`opendal`]: https://crates.io/crates/opendal
[`reqwest`]: https://crates.io/crates/reqwest
