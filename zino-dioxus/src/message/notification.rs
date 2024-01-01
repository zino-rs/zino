use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// A simple colored block meant to draw the attention to the user about something.
pub fn Notification<'a>(cx: Scope<'a, NotificationProps<'a>>) -> Element {
    let close_notification = use_state(cx, || false);
    if *close_notification.get() {
        None
    } else {
        let class = format_class!(cx, "notification");
        let close_class = format_class!(cx, close_class, "delete");
        render! {
            div {
                class: "{class}",
                button {
                    class: "{close_class}",
                    onclick: |_event| {
                        close_notification.set(true);
                    }
                }
                &cx.props.children
            }
        }
    }
}

/// The [`Notification`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct NotificationProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply to the `close` button element.
    #[props(into)]
    pub close_class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}
