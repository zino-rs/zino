# zino

`zino` is a full-featured web application framework for Rust with a focus on
productivity and performance.

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- ✨ Minimal design, modular architecture and high-level abstractions.
- ⚡ Embrace practical conventions to get the best performance.
- 🐘 Highly optimized ORM for PostgreSQL built on top of [`sqlx`].
- 🕗 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and chatbots.
- 📊 Supports for [`tracing`], [`metrics`] and logging.
- 🎨 Integrates with [`actix-web`] or [`axum`] in a consistent way.

## Getting started

You can start with the example [`actix-app`] or [`axum-app`].

## Feature flags

Currently, we provide the `actix` and `axum` features to enable an integration with
[`actix-web`] or [`axum`].

[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`actix-app`]: https://github.com/photino/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/photino/zino/tree/main/examples/axum-app
