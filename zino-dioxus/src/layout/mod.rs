//! Block structures of a page.

mod columns;
mod container;

pub use columns::{Columns, ColumnsProps};
pub use container::{
    Container, ContainerProps, FluidContainer, FluidContainerProps, MainContainer,
    MainContainerProps,
};
