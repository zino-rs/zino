//! Building HTML views using templates.
//!
//! # Supported template engines
//!
//! The following optional features are available:
//!
//! | Feature flag     | Description                                          | Default? |
//! |------------------|------------------------------------------------------|----------|
//! | `view-minijinja` | Enables the `minijinja` template engine.             | No       |
//! | `view-tera`      | Enables the `tera` template engine.                  | No       |

cfg_if::cfg_if! {
    if #[cfg(feature = "view-tera")] {
        mod tera;

        pub use self::tera::render;
    } else {
        mod minijinja;

        pub use self::minijinja::render;
    }
}
