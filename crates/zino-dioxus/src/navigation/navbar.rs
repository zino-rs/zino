use crate::{class::Class, extension::VNodeExt};
use dioxus::prelude::*;
use dioxus_router::{components::Link, navigation::NavigationTarget};

/// A responsive navigation header.
pub fn Navbar(props: NavbarProps) -> Element {
    let children = props.children?;
    if children.has_component("NavbarBrand") {
        rsx! {
            nav {
                class: props.class,
                { children }
            }
        }
    } else {
        rsx! {
            nav {
                class: props.class,
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
    #[props(into, default = "navbar is-link")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A container for the logo and optionally some links or icons.
pub fn NavbarBrand(props: NavbarBrandProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarBrand`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarBrandProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-brand")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// A horizontal menu used in the navigation header.
pub fn NavbarMenu(props: NavbarMenuProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarMenu`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarMenuProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-menu is-active")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The left section of the navbar menu.
pub fn NavbarStart(props: NavbarStartProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarStart`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarStartProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-start")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The middle section of the navbar menu.
pub fn NavbarCenter(props: NavbarCenterProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarCenter`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarCenterProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-center")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// The right section of the navbar menu.
pub fn NavbarEnd(props: NavbarEndProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarEnd`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarEndProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-end")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}

/// An interactive dropdown menu in the navbar.
pub fn NavbarDropdown(props: NavbarDropdownProps) -> Element {
    rsx! {
        div {
            class: "navbar-item has-dropdown is-hoverable",
            a {
                class: "{props.button_class}",
                class: if props.arrowless { "is-arrowless" },
                { props.button }
            }
            div {
                class: props.class,
                { props.children }
            }
        }
    }
}

/// The [`NavbarDropdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarDropdownProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-dropdown")]
    pub class: Class,
    /// A class to apply to the trigger button element.
    #[props(into, default = "navbar-link")]
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
    rsx! {
        Link {
            class: props.class.to_string(),
            active_class: props.active_class.to_string(),
            to: props.to,
            { props.children }
        }
    }
}

/// The [`NavbarLink`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarLinkProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-item")]
    pub class: Class,
    /// A class to apply to the generate HTML anchor tag if the `target` route is active.
    #[props(into, default = "is-active")]
    pub active_class: Class,
    /// The navigation target. Roughly equivalent to the href attribute of an HTML anchor tag.
    #[props(into)]
    pub to: NavigationTarget,
    /// The children to render within the component.
    children: Element,
}

/// A container for each single item of the navbar.
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    rsx! {
        div {
            class: props.class,
            { props.children }
        }
    }
}

/// The [`NavbarItem`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NavbarItemProps {
    /// The class attribute for the component.
    #[props(into, default = "navbar-item")]
    pub class: Class,
    /// The children to render within the component.
    children: Element,
}
