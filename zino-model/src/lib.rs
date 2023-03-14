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
#![feature(is_some_and)]
#![feature(let_chains)]
#![feature(once_cell)]
#![forbid(unsafe_code)]

use zino_core::{datetime::DateTime, extend::JsonObjectExt, model::Model, Map, Uuid};

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

/// Access model fields.
pub trait ModelAccessor: Model {
    /// Returns the `id` field.
    fn id(&self) -> Uuid;

    /// Returns the `name` field.
    fn name(&self) -> &str;

    /// Returns the `namespace` field.
    fn namespace(&self) -> &str;

    /// Returns the `visibility` field.
    fn visibility(&self) -> &str;

    /// Returns the `status` field.
    fn status(&self) -> &str;

    /// Returns the `description` field.
    fn description(&self) -> &str;

    /// Returns the `content` field.
    fn content(&self) -> &Map;

    /// Returns the `metrics` field.
    fn metrics(&self) -> &Map;

    /// Returns the `extras` field.
    fn extras(&self) -> &Map;

    /// Returns the `manager_id` field.
    fn manager_id(&self) -> Uuid;

    /// Returns the `maintainer_id` field.
    fn maintainer_id(&self) -> Uuid;

    /// Returns the `created_at` field.
    fn created_at(&self) -> DateTime;

    /// Returns the `version` field.
    fn updated_at(&self) -> DateTime;

    /// Returns the `version` field.
    fn version(&self) -> u64;

    /// Returns the `edition` field.
    fn edition(&self) -> u32;

    /// Returns `true` if `self` has the namespace prefix.
    #[inline]
    fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace()
            .strip_prefix(namespace)
            .is_some_and(|s| s == "" || s.starts_with(':'))
    }

    /// Returns `true` if the `visibility` is `public`.
    #[inline]
    fn is_public(&self) -> bool {
        self.visibility() == "public"
    }

    /// Returns `true` if the `visibility` is `internal`.
    #[inline]
    fn is_internal(&self) -> bool {
        self.visibility() == "internal"
    }

    /// Returns `true` if the `visibility` is `private`.
    #[inline]
    fn is_private(&self) -> bool {
        self.visibility() == "private"
    }

    /// Returns `true` if the `status` is `active`.
    #[inline]
    fn is_active(&self) -> bool {
        self.status() == "active"
    }

    /// Returns `true` if the `status` is `inactive`.
    #[inline]
    fn is_inactive(&self) -> bool {
        self.status() == "inactive"
    }

    /// Returns `true` if the `status` is `locked`.
    #[inline]
    fn is_locked(&self) -> bool {
        self.status() == "locked"
    }

    /// Returns `true` if the `status` is `deleted`.
    #[inline]
    fn is_deleted(&self) -> bool {
        self.status() == "deleted"
    }

    /// Returns the next version for the model.
    #[inline]
    fn next_version(&self) -> u64 {
        self.version() + 1
    }

    /// Constructs the query filters for the model of the current version.
    fn current_version_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("version", self.version());
        filters
    }

    /// Constructs the mutation updates for the model of the next version.
    fn next_version_updates(&self) -> Map {
        let mut updates = Map::with_capacity(2);
        updates.upsert("updated_at", DateTime::now().to_string());
        updates.upsert("version", self.next_version());
        updates
    }

    /// Constructs the query filters for the model of the next version.
    fn next_version_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("version", self.next_version());
        filters
    }

    /// Returns the next edition for the model.
    #[inline]
    fn next_edition(&self) -> u32 {
        self.edition() + 1
    }

    /// Constructs the query filters for the model of the current edition.
    fn current_edition_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("edition", self.edition());
        filters
    }

    /// Constructs the mutation updates for the model of the next edition.
    fn next_edition_updates(&self) -> Map {
        let mut updates = Map::with_capacity(2);
        updates.upsert("updated_at", DateTime::now().to_string());
        updates.upsert("version", self.next_version());
        updates.upsert("edition", self.next_edition());
        updates
    }

    /// Constructs the query filters for the model of the next edition.
    fn next_edition_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("edition", self.next_edition());
        filters
    }
}

macro impl_model_accessor($model:ty, $id:ident, $name:ident, $namespace:ident, $visibility:ident,
    $status:ident, $description:ident, $content:ident, $metrics:ident, $extras:ident,
    $manager_id:ident, $maintainer_id:ident, $created_at:ident, $updated_at:ident,
    $version:ident, $edition:ident) {
    impl ModelAccessor for $model {
        #[inline]
        fn id(&self) -> Uuid {
            self.$id
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
        fn content(&self) -> &Map {
            &self.$content
        }

        #[inline]
        fn metrics(&self) -> &Map {
            &self.$metrics
        }

        #[inline]
        fn extras(&self) -> &Map {
            &self.$extras
        }

        #[inline]
        fn manager_id(&self) -> Uuid {
            self.$manager_id
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
