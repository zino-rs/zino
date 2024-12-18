[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-connector)
[![docs-rs]](https://docs.rs/zino-connector)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Unified connector to data sources for [`zino`].

## Supported data sources

| Data source type | Description            | Feature flag           |
|------------------|------------------------|------------------------|
| `arrow`          | Apache Arrow           | `connector-arrow`      |
| `ceresdb`        | CeresDB                | `connector-mysql`      |
| `citus`          | Citus                  | `connector-postgres`   |
| `databend`       | Databend               | `connector-mysql`      |
| `graphql`        | GraphQL API            | `connector-http`       |
| `greptimedb`     | GreptimeDB             | `connector-postgres`   |
| `highgo`         | HighGo Database        | `connector-postgres`   |
| `hologres`       | Aliyun Hologres        | `connector-postgres`   |
| `http`           | HTTP services          | `connector-http`       |
| `mariadb`        | MariaDB                | `connector-mysql`      |
| `mysql`          | MySQL                  | `connector-mysql`      |
| `opengauss`      | openGauss              | `connector-postgres`   |
| `postgis`        | PostGIS                | `connector-postgres`   |
| `postgres`       | PostgreSQL             | `connector-postgres`   |
| `rest`           | RESTful API            | `connector-http`       |
| `sqlite`         | SQLite                 | `connector-sqlite`     |
| `tidb`           | TiDB                   | `connector-mysql`      |
| `timescaledb`    | TimescaleDB            | `connector-postgres`   |

[`zino`]: https://github.com/zino-rs/zino
