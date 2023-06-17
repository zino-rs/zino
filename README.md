# zino

`zino` is a **full-featured application framework** for Rust which emphasizes
**simplicity**, **extensibility** and **productivity**.

[![Crates.io](https://img.shields.io/crates/v/zino)][zino]
[![Documentation](https://shields.io/docsrs/zino)][zino-docs]
[![Downloads](https://img.shields.io/crates/d/zino)][zino]
[![License](https://img.shields.io/crates/l/zino)][license]

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- ✨ Minimal design, modular architecture and high-level abstractions.
- ⚡ Embrace practical conventions to get the best performance.
- 💎 Highly optimized ORM for MySQL and PostgreSQL based on [`sqlx`].
- 📅 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and chatbots.
- 📊 Built-in support for [`tracing`], [`metrics`] and logging.
- 🎨 Full integrations with [`actix-web`] and [`axum`].

## Getting started

You can start with the example [`actix-app`] or [`axum-app`].
Currently, it requires rustc **nightly** to build the project.

```shell
cd examples/axum-app
cargo run -- --env=dev
```

## Crates

| Name            | Description            | Crates.io    | Documentation |
|-----------------|------------------------|--------------|---------------|
| [`zino-core`]   | Core types and traits. | [![Crates.io](https://img.shields.io/crates/v/zino-core)][zino-core] | [![Documentation](https://shields.io/docsrs/zino-core)][zino-core-docs] |
| [`zino-derive`] | Derived traits.        | [![Crates.io](https://img.shields.io/crates/v/zino-derive)][zino-derive] | [![Documentation](https://shields.io/docsrs/zino-derive)][zino-derive-docs] |
| [`zino-model`]  | Model types.           | [![Crates.io](https://img.shields.io/crates/v/zino-model)][zino-model] | [![Documentation](https://shields.io/docsrs/zino-model)][zino-model-docs] |

## License

This project is licensed under the [MIT license][license].

[`zino-core`]: https://github.com/photino/zino/tree/main/zino-core
[`zino-derive`]: https://github.com/photino/zino/tree/main/zino-derive
[`zino-model`]: https://github.com/photino/zino/tree/main/zino-model
[zino]: https://crates.io/crates/zino
[zino-docs]: https://docs.rs/zino
[zino-core]: https://crates.io/crates/zino-core
[zino-core-docs]: https://docs.rs/zino-core
[zino-derive]: https://crates.io/crates/zino-derive
[zino-derive-docs]: https://docs.rs/zino-derive
[zino-model]: https://crates.io/crates/zino-model
[zino-model-docs]: https://docs.rs/zino-model
[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`actix-app`]: https://github.com/photino/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/photino/zino/tree/main/examples/axum-app
[license]: https://github.com/photino/zino/blob/main/LICENSE
