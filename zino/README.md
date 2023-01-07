# zino

`zino` is a full featured web application framework that focuses on productivity and performance.

[![Crates.io](https://img.shields.io/crates/v/zino)][zino]
[![Documentation](https://docs.rs/zino/badge.svg)][zino-docs]

## High level features

- ✨ Full features for web development out of the box.
- 💖 Minimal design, modular architecture and ease to use.
- 🚀 Embrace practical conventions to get high performance.
- 🐘 Built-in ORM for PostgreSQL based on [`sqlx`][sqlx].
- 📈 Support for `tracing`, `logging` and `metrics`.

## Getting started

You can start with the example [`axum-app`][axum-app].

## Documentation

`zino` consists of 4 crates:

- [zino][zino-docs]: Named features.
- [zino-core][zino-core-docs]: Core types and traits.
- [zino-derive][zino-derive-docs]: Derived traits.
- [zino-model][zino-model-docs]: Model types.

## License

This project is licensed under the [MIT license][license].

[zino]: https://crates.io/crates/zino
[sqlx]: https://crates.io/crates/sqlx
[axum-app]: https://github.com/photino/zino/tree/main/examples/axum-app
[zino-docs]: https://docs.rs/zino
[zino-core-docs]: https://docs.rs/zino-core
[zino-derive-docs]: https://docs.rs/zino-derive
[zino-model-docs]: https://docs.rs/zino-model
[license]: https://github.com/photino/zino/blob/main/LICENSE