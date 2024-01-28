[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino)
[![docs-rs]](https://docs.rs/zino)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

[`zino`] is a **next-generation** framework for **composable** applications in Rust
which emphasizes **simplicity**, **extensibility** and **productivity**.

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
- 💖 Full integrations with [`actix-web`], [`axum`] and [`dioxus`].

## Getting started

You can start with the example [`actix-app`], [`axum-app`] or [`dioxus-desktop`].

## Feature flags

The following optional features are available:

| Name         | Description                                          | Default? |
|--------------|------------------------------------------------------|----------|
| `actix`      | Enables the integration with [`actix-web`].          | No       |
| `axum`       | Enables the integration with [`axum`].               | No       |
| `dioxus`     | Enables the integration with [`dioxus`].             | No       |
| `orm`        | Enables the ORM for MySQL, PostgreSQL or **SQLite**. | No       |
| `view`       | Enables the HTML template rendering.                 | No       |

[`zino`]: https://github.com/zino-rs/zino
[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`dioxus`]: https://crates.io/crates/dioxus
[`actix-app`]: https://github.com/zino-rs/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/zino-rs/zino/tree/main/examples/axum-app
[`dioxus-desktop`]: https://github.com/zino-rs/zino/tree/main/examples/dioxus-desktop
