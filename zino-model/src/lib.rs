//! [![github]](https://github.com/photino/zino)
//! [![crates-io]](https://crates.io/crates/zino-model)
//! [![docs-rs]](https://docs.rs/zino-model)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs
//!
//! Model types for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![feature(async_fn_in_trait)]
#![feature(decl_macro)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![forbid(unsafe_code)]

use zino_core::{database::ModelAccessor, datetime::DateTime, Map, Uuid};

mod group;
mod policy;
mod resource;
mod tag;
mod user;

mod message;
mod order;

mod collection;
mod dataset;
mod source;
mod task;

mod log;
mod record;

pub use group::Group;
pub use policy::Policy;
pub use resource::Resource;
pub use tag::Tag;
pub use user::User;

pub use message::Message;
pub use order::Order;

pub use collection::Collection;
pub use dataset::Dataset;
pub use source::Source;
pub use task::Task;

pub use log::Log;
pub use record::Record;

macro impl_model_accessor($model:ty, $id:ident, $name:ident, $namespace:ident,
    $visibility:ident, $status:ident, $description:ident, $content:ident, $extra:ident,
    $owner_id:ident, $maintainer_id:ident, $created_at:ident, $updated_at:ident,
    $version:ident, $edition:ident) {
    impl ModelAccessor<Uuid> for $model {
        #[inline]
        fn id(&self) -> &Uuid {
            &self.$id
        }

        #[inline]
        fn name(&self) -> &str {
            &self.$name
        }

        #[inline]
        fn namespace(&self) -> &str {
            &self.$namespace
        }

        #[inline]
        fn visibility(&self) -> &str {
            &self.$visibility
        }

        #[inline]
        fn status(&self) -> &str {
            &self.$status
        }

        #[inline]
        fn description(&self) -> &str {
            &self.$description
        }

        #[inline]
        fn content(&self) -> Option<&Map> {
            let content = &self.$content;
            (!content.is_empty()).then_some(content)
        }

        #[inline]
        fn extra(&self) -> Option<&Map> {
            let extra = &self.$extra;
            (!extra.is_empty()).then_some(extra)
        }

        #[inline]
        fn owner_id(&self) -> Uuid {
            self.$owner_id
        }

        #[inline]
        fn maintainer_id(&self) -> Uuid {
            self.$maintainer_id
        }

        #[inline]
        fn created_at(&self) -> DateTime {
            self.$created_at
        }

        #[inline]
        fn updated_at(&self) -> DateTime {
            self.$updated_at
        }

        #[inline]
        fn version(&self) -> u64 {
            self.$version
        }

        #[inline]
        fn edition(&self) -> u32 {
            self.$edition
        }
    }
}
