use crate::{class::Class, extension::VNodeExt};
use dioxus::prelude::*;
use dioxus_router::components::{IntoRoutable, Link};

/// A responsive navigation header.
pub fn Navbar(props: NavbarProps) -> Element {
    let children = props.children.as_ref()?;
    let class = props.class;
    if children.has_component("NavbarBrand") {
        rsx! {
            nav {
                class: "{class}",
                { children }
            }
        }
    } else {
        rsx! {
            nav {
                class: "{class}",
                NavbarMenu {
                    { children }
                }
            }
        }
    }
}

/// The [`Navbar`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar is-link".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A container for the logo and optionally some links or icons.
pub fn NavbarBrand(props: NavbarBrandProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarBrand`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarBrandProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-brand".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A horizontal menu used in the navigation header.
pub fn NavbarMenu(props: NavbarMenuProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarMenu`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarMenuProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-menu is-active".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The left section of the navbar menu.
pub fn NavbarStart(props: NavbarStartProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarStart`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarStartProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-start".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The middle section of the navbar menu.
pub fn NavbarCenter(props: NavbarCenterProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarCenter`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarCenterProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-center".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The right section of the navbar menu.
pub fn NavbarEnd(props: NavbarEndProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarEnd`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarEndProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-end".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// An interactive dropdown menu in the navbar.
pub fn NavbarDropdown(props: NavbarDropdownProps) -> Element {
    let class = props.class;
    let button_class = props.button_class;
    let arrow_class = Class::check("is-arrowless", props.arrowless);
    rsx! {
        div {
            class: "navbar-item has-dropdown is-hoverable",
            a {
                class: "{button_class} {arrow_class}",
                { props.button }
            }
            div {
                class: "{class}",
                { props.children }
            }
        }
    }
}

/// The [`NavbarDropdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarDropdownProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-dropdown".into())]
    pub class: Class,
    /// A class to apply to the trigger button element.
    #[props(into, default = "navbar-link".into())]
    pub button_class: Class,
    /// A flag to indicate whether the trigger button has an arrow or not.
    #[props(default)]
    pub arrowless: bool,
    /// The trigger button for the dropdown menu.
    pub button: Element,
    /// The children to render within the component.
    children: Element,
}

/// A link to navigate to another route in the navigation header.
pub fn NavbarLink(props: NavbarLinkProps) -> Element {
    let class = props.class;
    let active_class = props.active_class;
    rsx! {
        Link {
            class: "{class}",
            active_class: "{active_class}",
            to: props.to.clone(),
            { props.children }
        }
    }
}

/// The [`NavbarLink`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarLinkProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-item".into())]
    pub class: Class,
    /// A class to apply to the generate HTML anchor tag if the `target` route is active.
    #[props(into, default = "is-active".into())]
    pub active_class: Class,
    /// The navigation target. Roughly equivalent to the href attribute of an HTML anchor tag.
    #[props(into)]
    pub to: IntoRoutable,
    /// The children to render within the component.
    children: Element,
}

/// A container for each single item of the navbar.
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    let class = props.class;
    rsx! {
        div {
            class: "{class}",
            { props.children }
        }
    }
}

/// The [`NavbarItem`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarItemProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-item".into())]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}
