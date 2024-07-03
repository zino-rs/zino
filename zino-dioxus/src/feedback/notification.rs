use crate::class::Class;
use dioxus::prelude::*;
use std::time::Duration;
use zino_core::SharedString;

/// A simple colored block meant to draw the attention to the user about something.
pub fn Notification(props: NotificationProps) -> Element {
    let mut hidden = use_signal(|| props.hidden);
    if props.duration > 0 {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(props.duration)).await;
            hidden.set(true);
        });
    }
    if hidden() {
        return None;
    }
    rsx! {
        div {
            class: props.class,
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.theme.is_empty() { "is-{props.theme}" },
            position: "fixed",
            top: "4rem",
            right: "0.75rem",
            z_index: 99,
            if props.on_close.is_some() {
                button {
                    r#type: "button",
                    class: props.close_class,
                    onclick: move |event| {
                        if let Some(handler) = props.on_close.as_ref() {
                            handler.call(event);
                        }
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
    /// The color of the notification: `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The theme of the notification: `light`.
    #[props(into, default)]
    pub theme: SharedString,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<MouseEvent>>,
    /// A flag to determine whether the notification is hidden or not.
    #[props(default)]
    pub hidden: bool,
    /// A duration in milliseconds.
    #[props(default = 3000)]
    pub duration: u64,
    /// The children to render within the component.
    children: Element,
}
