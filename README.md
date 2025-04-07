# zino

`zino` is a **next-generation** framework for **composable** applications in Rust
which emphasizes **simplicity**, **extensibility** and **productivity**.

[![Crates.io](https://img.shields.io/crates/v/zino)][zino]
[![Documentation](https://shields.io/docsrs/zino)][zino-docs]
[![License](https://img.shields.io/crates/l/zino)][license]

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- 🎨 Minimal design, composable architecture and high-level abstractions.
- 🌐 Adopt an API-first approach to development with open standards.
- ⚡ Embrace practical conventions to get the best performance.
- 💎 Expressive ORM for MySQL, PostgreSQL and SQLite based on [`sqlx`].
- ✨ Innovations on query population, field translation and model hooks.
- 📅 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and LLMs.
- 📊 Built-in support for [`tracing`], [`metrics`] and logging.
- 💖 Full integrations with [`actix-web`], [`axum`], [`dioxus`] and more.

## Getting started

You can start with the example [`actix-app`], [`axum-app`], [`dioxus-desktop`] or [`ntex-app`].
It requires **Rust 1.85+** to build the project.

```shell
cd examples/axum-app
cargo run
```

Here is the simplest application to run a server:
```toml
[package]
name = "zino-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
zino = { version = "0.34", features = ["axum"] }
```

```rust
use zino::prelude::*;

fn main() {
    zino::Cluster::boot().run()
}
```

## Crates

| Name            | Description                       | Crates.io    | Documentation |
|-----------------|-----------------------------------|--------------|---------------|
| [`zino-core`]   | Core types and traits.            | [![Crates.io](https://img.shields.io/crates/v/zino-core)][zino-core] | [![Documentation](https://shields.io/docsrs/zino-core)][zino-core-docs] |
| [`zino-auth`]   | Authentication and authorization. | [![Crates.io](https://img.shields.io/crates/v/zino-auth)][zino-auth] | [![Documentation](https://shields.io/docsrs/zino-auth)][zino-auth-docs] |
| [`zino-channel`] | Cloud events and subscriptions.  | [![Crates.io](https://img.shields.io/crates/v/zino-channel)][zino-channel] | [![Documentation](https://shields.io/docsrs/zino-channel)][zino-channel-docs] |
| [`zino-storage`] | Files and storage services.      | [![Crates.io](https://img.shields.io/crates/v/zino-storage)][zino-storage] | [![Documentation](https://shields.io/docsrs/zino-storage)][zino-storage-docs] |
| [`zino-http`]   | Requests and responses.           | [![Crates.io](https://img.shields.io/crates/v/zino-http)][zino-http] | [![Documentation](https://shields.io/docsrs/zino-http)][zino-http-docs] |
| [`zino-openapi`] | OpenAPI docs generator.          | [![Crates.io](https://img.shields.io/crates/v/zino-openapi)][zino-openapi] | [![Documentation](https://shields.io/docsrs/zino-openapi)][zino-openapi-docs] |
| [`zino-orm`]    | Database schema and ORM.          | [![Crates.io](https://img.shields.io/crates/v/zino-orm)][zino-orm] | [![Documentation](https://shields.io/docsrs/zino-orm)][zino-orm-docs] |
| [`zino-derive`] | Derived traits.                   | [![Crates.io](https://img.shields.io/crates/v/zino-derive)][zino-derive] | [![Documentation](https://shields.io/docsrs/zino-derive)][zino-derive-docs] |
| [`zino-model`]  | Domain models.                    | [![Crates.io](https://img.shields.io/crates/v/zino-model)][zino-model] | [![Documentation](https://shields.io/docsrs/zino-model)][zino-model-docs] |
| [`zino-connector`] | Connector to data sources.     | [![Crates.io](https://img.shields.io/crates/v/zino-connector)][zino-connector] | [![Documentation](https://shields.io/docsrs/zino-connector)][zino-connector-docs] |
| [`zino-chatbot`] | Chatbot services.                | [![Crates.io](https://img.shields.io/crates/v/zino-chatbot)][zino-chatbot] | [![Documentation](https://shields.io/docsrs/zino-chatbot)][zino-chatbot-docs] |
| [`zino-extra`]  | Extra utilities.                  | [![Crates.io](https://img.shields.io/crates/v/zino-extra)][zino-extra] | [![Documentation](https://shields.io/docsrs/zino-extra)][zino-extra-docs] |
| [`zino-actix`]  | Integrations with actix-web.      | [![Crates.io](https://img.shields.io/crates/v/zino-actix)][zino-actix] | [![Documentation](https://shields.io/docsrs/zino-actix)][zino-actix-docs] |
| [`zino-axum`]   | Integrations with axum.           | [![Crates.io](https://img.shields.io/crates/v/zino-axum)][zino-axum] | [![Documentation](https://shields.io/docsrs/zino-axum)][zino-axum-docs] |
| [`zino-ntex`]   | Integrations with ntex.           | [![Crates.io](https://img.shields.io/crates/v/zino-ntex)][zino-ntex] | [![Documentation](https://shields.io/docsrs/zino-ntex)][zino-ntex-docs] |
| [`zino-dioxus`] | Dioxus components.                | [![Crates.io](https://img.shields.io/crates/v/zino-dioxus)][zino-dioxus] | [![Documentation](https://shields.io/docsrs/zino-dioxus)][zino-dioxus-docs] |
| [`zino-amis`]   | UI generator for amis.            | [![Crates.io](https://img.shields.io/crates/v/zino-amis)][zino-amis] | [![Documentation](https://shields.io/docsrs/zino-amis)][zino-amis-docs] |
| [`zino-cli`]    | CLI tools.                        | [![Crates.io](https://img.shields.io/crates/v/zino-cli)][zino-cli] | [![Documentation](https://shields.io/docsrs/zino-cli)][zino-cli-docs] |

## License

This project is licensed under the [MIT license][license].

## Community

If you have any problems or ideas, please don't hesitate to [open an issue][zino-issue].

[`zino-core`]: https://github.com/zino-rs/zino/tree/main/crates/zino-core
[`zino-auth`]: https://github.com/zino-rs/zino/tree/main/crates/zino-auth
[`zino-channel`]: https://github.com/zino-rs/zino/tree/main/crates/zino-channel
[`zino-storage`]: https://github.com/zino-rs/zino/tree/main/crates/zino-storage
[`zino-http`]: https://github.com/zino-rs/zino/tree/main/crates/zino-http
[`zino-openapi`]: https://github.com/zino-rs/zino/tree/main/crates/zino-openapi
[`zino-derive`]: https://github.com/zino-rs/zino/tree/main/crates/zino-derive
[`zino-orm`]: https://github.com/zino-rs/zino/tree/main/crates/zino-orm
[`zino-model`]: https://github.com/zino-rs/zino/tree/main/crates/zino-model
[`zino-connector`]: https://github.com/zino-rs/zino/tree/main/crates/zino-connector
[`zino-chatbot`]: https://github.com/zino-rs/zino/tree/main/crates/zino-chatbot
[`zino-extra`]: https://github.com/zino-rs/zino/tree/main/crates/zino-extra
[`zino-actix`]: https://github.com/zino-rs/zino/tree/main/crates/zino-actix
[`zino-axum`]: https://github.com/zino-rs/zino/tree/main/crates/zino-axum
[`zino-ntex`]: https://github.com/zino-rs/zino/tree/main/crates/zino-ntex
[`zino-dioxus`]: https://github.com/zino-rs/zino/tree/main/crates/zino-dioxus
[`zino-amis`]: https://github.com/zino-rs/zino/tree/main/crates/zino-amis
[`zino-cli`]: https://github.com/zino-rs/zino-cli
[zino]: https://crates.io/crates/zino
[zino-docs]: https://docs.rs/zino
[zino-core]: https://crates.io/crates/zino-core
[zino-core-docs]: https://docs.rs/zino-core
[zino-auth]: https://crates.io/crates/zino-auth
[zino-auth-docs]: https://docs.rs/zino-auth
[zino-channel]: https://crates.io/crates/zino-channel
[zino-channel-docs]: https://docs.rs/zino-channel
[zino-storage]: https://crates.io/crates/zino-storage
[zino-storage-docs]: https://docs.rs/zino-storage
[zino-http]: https://crates.io/crates/zino-http
[zino-http-docs]: https://docs.rs/zino-http
[zino-openapi]: https://crates.io/crates/zino-openapi
[zino-openapi-docs]: https://docs.rs/zino-openapi
[zino-orm]: https://crates.io/crates/zino-orm
[zino-orm-docs]: https://docs.rs/zino-orm
[zino-derive]: https://crates.io/crates/zino-derive
[zino-derive-docs]: https://docs.rs/zino-derive
[zino-model]: https://crates.io/crates/zino-model
[zino-model-docs]: https://docs.rs/zino-model
[zino-connector]: https://crates.io/crates/zino-connector
[zino-connector-docs]: https://docs.rs/zino-connector
[zino-chatbot]: https://crates.io/crates/zino-chatbot
[zino-chatbot-docs]: https://docs.rs/zino-chatbot
[zino-extra]: https://crates.io/crates/zino-extra
[zino-extra-docs]: https://docs.rs/zino-extra
[zino-actix]: https://crates.io/crates/zino-actix
[zino-actix-docs]: https://docs.rs/zino-actix
[zino-axum]: https://crates.io/crates/zino-axum
[zino-axum-docs]: https://docs.rs/zino-axum
[zino-ntex]: https://crates.io/crates/zino-ntex
[zino-ntex-docs]: https://docs.rs/zino-ntex
[zino-dioxus]: https://crates.io/crates/zino-dioxus
[zino-dioxus-docs]: https://docs.rs/zino-dioxus
[zino-amis]: https://crates.io/crates/zino-amis
[zino-amis-docs]: https://docs.rs/zino-amis
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
