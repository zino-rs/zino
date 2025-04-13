use std::fmt::Display;
use zino_core::Uuid;

/// An interface for the primary key.
pub trait PrimaryKey: Default + Display + PartialEq {}

impl PrimaryKey for i32 {}

impl PrimaryKey for i64 {}

impl PrimaryKey for u32 {}

impl PrimaryKey for u64 {}

impl PrimaryKey for String {}

impl PrimaryKey for Uuid {}
