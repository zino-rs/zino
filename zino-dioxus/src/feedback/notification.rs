use crate::class::Class;
use dioxus::prelude::*;

/// A simple colored block meant to draw the attention to the user about something.
pub fn Notification(props: NotificationProps) -> Element {
    let class = props.class;
    let close_class = props.close_class;
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
    #[props(into, default = "notification".into())]
    pub class: Class,
    /// A class to apply to the `close` button element.
    #[props(into, default = "delete".into())]
    pub close_class: Class,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<MouseEvent>>,
    /// A flag to determine whether the modal is visible or not.
    #[props(default)]
    pub visible: bool,
    /// The children to render within the component.
    children: Element,
}
