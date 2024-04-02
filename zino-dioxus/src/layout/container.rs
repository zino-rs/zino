use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A responsive, fixed-width container with the `max-width` changes at each breakpoint.
pub fn Container(props: ContainerProps) -> Element {
    let class = format_class!(props, "container");
    rsx! {
        main {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ContainerProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A full width container spanning the entire width of the viewport.
pub fn FluidContainer(props: FluidContainerProps) -> Element {
    let class = format_class!(props, "container-fluid");
    rsx! {
        main {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FluidContainerProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}

/// A container rendered as the `<main>` element.
pub fn MainContainer(props: MainContainerProps) -> Element {
    let class = format_class!(props, "container-fluid px-3 my-3");
    rsx! {
        main {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`MainContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct MainContainerProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The children to render within the component.
    children: Element,
}
