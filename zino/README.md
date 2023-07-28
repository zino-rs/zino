# zino

`zino` is a **next-generation** framework for **composable** applications in Rust
which emphasizes **simplicity**, **extensibility** and **productivity**.

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- 🎨 Minimal design, composable architecture and high-level abstractions.
- 🌐 Adopt an API-first approch to development with open standards.
- ⚡ Embrace practical conventions to get the best performance.
- 💎 Highly optimized ORM for MySQL and PostgreSQL based on [`sqlx`].
- ✨ Innovations on query population, field translation and model hooks.
- 📅 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and chatbots.
- 📊 Built-in support for [`tracing`], [`metrics`] and logging.
- 💖 Full integrations with [`actix-web`] and [`axum`].

## Getting started

You can start with the example [`actix-app`] or [`axum-app`].

## Feature flags

The following optional features are available:

| Name         | Description                                          | Default? |
|--------------|------------------------------------------------------|----------|
| `actix`      | Enables the integration with [`actix-web`].          | No       |
| `axum`       | Enables the integration with [`axum`].               | No       |
| `orm`        | Enables the ORM for MySQL or **PostgreSQL**.         | Yes      |
| `view`       | Enables the HTML template rendering.                 | Yes      |

[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`actix-app`]: https://github.com/photino/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/photino/zino/tree/main/examples/axum-app
