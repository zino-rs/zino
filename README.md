# zino

`zino` is a **next-generation** framework for **composable** applications in Rust
which emphasizes **simplicity**, **extensibility** and **productivity**.

[![Crates.io](https://img.shields.io/crates/v/zino)][zino]
[![Documentation](https://shields.io/docsrs/zino)][zino-docs]
[![License](https://img.shields.io/crates/l/zino)][license]

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- 🎨 Minimal design, composable architecture and high-level abstractions.
- 🌐 Adopt an API-first approch to development with open standards.
- ⚡ Embrace practical conventions to get the best performance.
- 💎 Highly optimized ORM for MySQL, PostgreSQL and SQLite based on [`sqlx`].
- ✨ Innovations on query population, field translation and model hooks.
- 📅 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and chatbots.
- 📊 Built-in support for [`tracing`], [`metrics`] and logging.
- 💖 Full integrations with [`actix-web`], [`axum`], [`dioxus`] and [`ntex`].

## Getting started

You can start with the example [`actix-app`], [`axum-app`], [`dioxus-desktop`] or [`ntex-app`].
It requires **Rust 1.75+** to build the project.

```shell
cd examples/axum-app
cargo run
```

Here is the simplest application to run a server:
```toml
[package]
name = "zino-app"
version = "0.1.0"
edition = "2021"

[dependencies]
zino = { version = "0.24", features = ["axum"] }
```

```rust
use zino::prelude::*;

fn main() {
    zino::Cluster::boot().run()
}
```

## Crates

| Name            | Description            | Crates.io    | Documentation |
|-----------------|------------------------|--------------|---------------|
| [`zino-core`]   | Core types and traits. | [![Crates.io](https://img.shields.io/crates/v/zino-core)][zino-core] | [![Documentation](https://shields.io/docsrs/zino-core)][zino-core-docs] |
| [`zino-derive`] | Derived traits.        | [![Crates.io](https://img.shields.io/crates/v/zino-derive)][zino-derive] | [![Documentation](https://shields.io/docsrs/zino-derive)][zino-derive-docs] |
| [`zino-model`]  | Domain models.         | [![Crates.io](https://img.shields.io/crates/v/zino-model)][zino-model] | [![Documentation](https://shields.io/docsrs/zino-model)][zino-model-docs] |
| [`zino-extra`]  | Extra utilities.       | [![Crates.io](https://img.shields.io/crates/v/zino-extra)][zino-extra] | [![Documentation](https://shields.io/docsrs/zino-extra)][zino-extra-docs] |
| [`zino-dioxus`] | Dioxus components.     | [![Crates.io](https://img.shields.io/crates/v/zino-dioxus)][zino-dioxus] | [![Documentation](https://shields.io/docsrs/zino-dioxus)][zino-dioxus-docs] |
| [`zino-cli`]    | CLI tools.             | [![Crates.io](https://img.shields.io/crates/v/zino-cli)][zino-cli] | [![Documentation](https://shields.io/docsrs/zino-cli)][zino-cli-docs] |

## License

This project is licensed under the [MIT license][license].

## Community

If you have any problems or ideas, please don't hesitate to [open an issue][zino-issue].

[`zino-core`]: https://github.com/zino-rs/zino/tree/main/zino-core
[`zino-derive`]: https://github.com/zino-rs/zino/tree/main/zino-derive
[`zino-model`]: https://github.com/zino-rs/zino/tree/main/zino-model
[`zino-extra`]: https://github.com/zino-rs/zino/tree/main/zino-extra
[`zino-dioxus`]: https://github.com/zino-rs/zino/tree/main/zino-dioxus
[`zino-cli`]: https://github.com/zino-rs/zino/tree/main/zino-cli
[zino]: https://crates.io/crates/zino
[zino-docs]: https://docs.rs/zino
[zino-core]: https://crates.io/crates/zino-core
[zino-core-docs]: https://docs.rs/zino-core
[zino-derive]: https://crates.io/crates/zino-derive
[zino-derive-docs]: https://docs.rs/zino-derive
[zino-model]: https://crates.io/crates/zino-model
[zino-model-docs]: https://docs.rs/zino-model
[zino-extra]: https://crates.io/crates/zino-extra
[zino-extra-docs]: https://docs.rs/zino-extra
[zino-dioxus]: https://crates.io/crates/zino-dioxus
[zino-dioxus-docs]: https://docs.rs/zino-dioxus
[zino-cli]: https://crates.io/crates/zino-cli
[zino-cli-docs]: https://docs.rs/zino-cli
[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`dioxus`]: https://crates.io/crates/dioxus
[`ntex`]: https://crates.io/crates/ntex
[`actix-app`]: https://github.com/zino-rs/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/zino-rs/zino/tree/main/examples/axum-app
[`dioxus-desktop`]: https://github.com/zino-rs/zino/tree/main/examples/dioxus-desktop
[`ntex-app`]: https://github.com/zino-rs/zino/tree/main/examples/ntex-app
[license]: https://github.com/zino-rs/zino/blob/main/LICENSE
[zino-issue]: https://github.com/zino-rs/zino/issues/new
