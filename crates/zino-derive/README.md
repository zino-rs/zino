[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-derive)
[![docs-rs]](https://docs.rs/zino-derive)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Derived traits for [`zino`].

The following traits can be derived:

- [`Model`](zino_core::model::Model): General data model.
- [`ModelHooks`](zino_core::model::ModelHooks): Hooks for the model.
- [`Entity`](zino_orm::Entity): An interface for the model entity.
- [`DecodeRow`](zino_orm::DecodeRow): A collection of values that can be decoded from a single row.
- [`Schema`](zino_orm::Schema): Database schema.
- [`ModelAccessor`](zino_orm::ModelAccessor): Access model fields.

[`zino`]: https://github.com/zino-rs/zino
