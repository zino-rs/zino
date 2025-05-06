use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// An interactive dropdown menu for discoverable content.
pub fn Dropdown(props: DropdownProps) -> Element {
    let title = props.title;
    rsx! {
        div {
            class: "{props.class}",
            class: if props.active { "is-active" },
            class: if props.hoverable { "is-hoverable" },
            class: if props.dropup { "is-up" },
            class: if props.align == "right" { "is-right" },
            onmouseenter: move |event| {
                if let Some(handler) = props.on_mouse_enter.as_ref() {
                    handler.call(event);
                }
            },
            onmouseleave: move |event| {
                if let Some(handler) = props.on_mouse_leave.as_ref() {
                    handler.call(event);
                }
            },
            if let Some(trigger) = props.trigger {
                div {
                    class: props.trigger_class,
                    title: if !title.is_empty() { "{title}" },
                    onclick: move |event| {
                        if props.hoverable {
                            event.stop_propagation();
                        }
                        if let Some(handler) = props.on_trigger.as_ref() {
                            handler.call(event);
                        }
                    },
                    { trigger }
                }
            }
            div {
                class: props.menu_class,
                div {
                    class: props.content_class,
                    onclick: move |event| {
                        if props.hoverable {
                            event.stop_propagation();
                        }
                        if let Some(handler) = props.on_action.as_ref() {
                            handler.call(event);
                        }
                    },
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
    #[props(into, default = "dropdown")]
    pub class: Class,
    /// A class to apply to the trigger button.
    #[props(into, default = "dropdown-trigger")]
    pub trigger_class: Class,
    /// A class to apply to the menu.
    #[props(into, default = "dropdown-menu")]
    pub menu_class: Class,
    /// A class to apply to the menu content.
    #[props(into, default = "dropdown-content")]
    pub content_class: Class,
    /// A flag to indicate whether the dropdown will show up all the time.
    #[props(default)]
    pub active: bool,
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
    pub trigger: Option<Element>,
    /// The menu items to be rendered.
    #[props(into)]
    pub items: Vec<Element>,
    /// An event handler to be called when the mouse enters the element.
    pub on_mouse_enter: Option<EventHandler<MouseEvent>>,
    /// An event handler to be called when the mouse leaves the element.
    pub on_mouse_leave: Option<EventHandler<MouseEvent>>,
    /// An event handler to be called when the trigger button is clicked.
    pub on_trigger: Option<EventHandler<MouseEvent>>,
    /// An event handler to be called when the menu action is clicked.
    pub on_action: Option<EventHandler<MouseEvent>>,
}
