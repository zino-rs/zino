use crate::{class::Class, format_class};
use dioxus::prelude::*;
use zino_core::SharedString;

/// Contextual feedback messages for typical user actions.
pub fn Message(props: MessageProps) -> Element {
    if props.hidden {
        return None;
    }

    let class = format_class!(props, "message");
    let close_class = format_class!(props, close_class, "delete");
    let title = props.title.as_deref().unwrap_or_default();
    rsx! {
        div {
            class: "{class}",
            if !title.is_empty() {
                div {
                    class: "message-header",
                    span { "{title}" }
                    button {
                        class: "{close_class}",
                        onclick: move |_event| {
                            if let Some(handler) = props.on_close.as_ref() {
                                handler.call(false);
                            }
                        }
                    }
                }
            }
            div {
                class: "message-body",
                { props.children }
            }
        }
    }
}

/// The [`Message`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct MessageProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply to the `close` button element.
    #[props(into)]
    pub close_class: Option<Class>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<bool>>,
    /// A flag to determine whether the message is hidden or not.
    #[props(default = false)]
    pub hidden: bool,
    /// The title in the message header.
    #[props(into)]
    pub title: Option<SharedString>,
    /// The children to render within the component.
    children: Element,
}
