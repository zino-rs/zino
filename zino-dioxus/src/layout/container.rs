use crate::class::Class;
use dioxus::prelude::*;

/// A responsive, fixed-width container with the `max-width` changes at each breakpoint.
pub fn Container(props: ContainerProps) -> Element {
    rsx! {
        main {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ContainerProps {
    /// The class attribute for the component.
    #[props(into, default = "container".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A full width container spanning the entire width of the viewport.
pub fn FluidContainer(props: FluidContainerProps) -> Element {
    rsx! {
        main {
            class: props.class,
            width: "100%",
            { props.children }
        }
    }
}

/// The [`FluidContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FluidContainerProps {
    /// The class attribute for the component.
    #[props(into, default = "container is-fluid".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A container rendered as the `<main>` element.
pub fn MainContainer(props: MainContainerProps) -> Element {
    rsx! {
        main {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`MainContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct MainContainerProps {
    /// The class attribute for the component.
    #[props(into, default = "main-container px-3 py-3".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}
