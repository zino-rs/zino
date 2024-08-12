[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino-cli)
[![docs-rs]](https://docs.rs/zino-cli)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

CLI tools for [`zino`].

[`zino`]: https://github.com/zino-rs/zino

## Features
- **Project Initialization**: Quickly set up new `zino` projects.
- **Dependency Management**: Manage your project dependencies with ease.

## Installation
```sh
cargo install zino-cli
```

## Usage

### Create a new project
```sh
zli new <project_name>
```
options:
- `--template <template_url>`: Use a custom template for the project.

### Init project in current directory
```sh
zli init
```
options:
- `--template <template_url>`: Use a custom template for the project.
- `--project-name <project_name>`: Name of the project (current_dir by default).

### Manage dependencies
run `zli serve` and access http://localhost:6080/zino-config.html in your browser.