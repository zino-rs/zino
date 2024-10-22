use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// An interactive dropdown menu for discoverable content.
pub fn Dropdown(props: DropdownProps) -> Element {
    let title = props.title;
    rsx! {
        div {
            class: props.class,
            class: if props.hoverable { "is-hoverable" },
            class: if props.dropup { "is-up" },
            class: if props.align == "right" { "is-right" },
            div {
                class: props.trigger_class,
                title: if !title.is_empty() { "{title}" },
                { props.trigger }
            }
            div {
                class: props.menu_class,
                div {
                    class: props.content_class,
                    for item in props.items.iter() {
                        { item }
                    }
                }
            }
        }
    }
}

/// The [`Dropdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct DropdownProps {
    /// The class attribute for the component.
    #[props(into, default = "dropdown".into())]
    pub class: Class,
    /// A class to apply to the trigger button.
    #[props(into, default = "dropdown-trigger".into())]
    pub trigger_class: Class,
    /// A class to apply to the menu.
    #[props(into, default = "dropdown-menu".into())]
    pub menu_class: Class,
    /// A class to apply to the menu content.
    #[props(into, default = "dropdown-content".into())]
    pub content_class: Class,
    /// A flag to indicate whether the dropdown will show up when hovering.
    #[props(default)]
    pub hoverable: bool,
    /// A flag to indicate whether the dropdown menu will appear above the trigger button.
    #[props(default)]
    pub dropup: bool,
    /// The alignment of the dropdown: `left` | `right`.
    #[props(into, default)]
    pub align: SharedString,
    /// The title for the trigger button.
    #[props(into, default)]
    pub title: SharedString,
    /// The trigger button to be rendered.
    pub trigger: Element,
    /// The menu items to be rendered.
    #[props(into)]
    pub items: Vec<Element>,
}
