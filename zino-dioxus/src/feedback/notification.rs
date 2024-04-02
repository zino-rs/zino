use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A simple colored block meant to draw the attention to the user about something.
pub fn Notification(props: NotificationProps) -> Element {
    let class = format_class!(props, "notification");
    let close_class = format_class!(props, close_class, "delete");
    let hidden_class = Class::check("is-hidden", !props.visible);
    rsx! {
        div {
            class: "{class} {hidden_class}",
            button {
                class: "{close_class}",
                onclick: move |event| {
                    if let Some(handler) = props.on_close.as_ref() {
                        handler.call(event);
                    }
                }
            }
            { props.children }
        }
    }
}

/// The [`Notification`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct NotificationProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply to the `close` button element.
    #[props(into)]
    pub close_class: Option<Class>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<MouseEvent>>,
    /// A flag to determine whether the modal is visible or not.
    #[props(default = false)]
    pub visible: bool,
    /// The children to render within the component.
    children: Element,
}
