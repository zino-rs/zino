[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-orm)
[![docs-rs]](https://docs.rs/zino-orm)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

Database schema and ORM for [`zino`].

# Supported database drivers

The following optional features are available:

| Feature flag   | Description                                          | Default? |
|----------------|------------------------------------------------------|----------|
| `orm-mariadb`  | Enables the MariaDB database driver.                 | No       |
| `orm-mysql`    | Enables the MySQL database driver.                   | No       |
| `orm-postgres` | Enables the PostgreSQL database driver.              | No       |
| `orm-sqlite`   | Enables the SQLite database driver.                  | No       |
| `orm-tidb`     | Enables the TiDB database driver.                    | No       |

# Mappings of Rust data types

| Rust type                     | MySQL datatype     | PostgreSQL datatype   | SQLite datatype |
|-------------------------------|--------------------|-----------------------| ----------------|
| `bool`                        | BOOLEAN            | BOOLEAN               | BOOLEAN         |
| `i8`                          | TINYINT            | SMALLINT              | INTEGER         |
| `u8`                          | TINYINT UNSIGNED   | SMALLINT              | INTEGER         |
| `i16`                         | SMALLINT           | SMALLINT              | INTEGER         |
| `u16`                         | SMALLINT UNSIGNED  | SMALLINT              | INTEGER         |
| `i32`, `Option<i32>`          | INT                | INT, SERIAL           | INTEGER         |
| `u32`, `Option<u32>`          | INT UNSIGNED       | INT, SERIAL           | INTEGER         |
| `i64`, `Option<i64>`, `isize` | BIGINT             | BIGINT, BIGSERIAL     | INTEGER         |
| `u64`, `Option<u64>`, `usize` | BIGINT UNSIGNED    | BIGINT, BIGSERIAL     | INTEGER         |
| `f32`                         | FLOAT              | REAL                  | REAL            |
| `f64`                         | DOUBLE             | DOUBLE PRECISION      | REAL            |
| `Decimal`                     | NUMERIC            | NUMERIC               | TEXT            |
| `String`, `Option<String>`    | TEXT, VARCHAR(255) | TEXT                  | TEXT            |
| `Date`, `NaiveDate`           | DATE               | DATE                  | DATE            |
| `Time`, `NaiveTime`           | TIME               | TIME                  | TIME            |
| `DateTime`                    | TIMESTAMP(6)       | TIMESTAMPTZ           | DATETIME        |
| `NaiveDateTime`               | DATETIME(6)        | TIMESTAMP             | DATETIME        |
| `Uuid`, `Option<Uuid>`        | CHAR(36), UUID     | UUID                  | TEXT            |
| `Vec<u8>`                     | BLOB               | BYTEA                 | BLOB            |
| `Vec<i32>`, `Vec<u32>`        | JSON               | INT[]                 | TEXT            |
| `Vec<i64>`, `Vec<u64>`        | JSON               | BIGINT[]              | TEXT            |
| `Vec<String>`                 | JSON               | TEXT[]                | TEXT            |
| `Vec<UUID>`                   | JSON               | UUID[]                | TEXT            |
| `Map`                         | JSON               | JSONB                 | TEXT            |

[`zino`]: https://github.com/zino-rs/zino
