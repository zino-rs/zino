use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;

/// Contextual feedback messages for typical user actions.
pub fn Message<'a>(cx: Scope<'a, MessageProps<'a>>) -> Element {
    if cx.props.hidden {
        return None;
    }

    let class = format_class!(cx, "message");
    let close_class = format_class!(cx, close_class, "delete");
    let title = cx.props.title.as_deref().unwrap_or_default();
    render! {
        div {
            class: "{class}",
            if !title.is_empty() {
                render!(div {
                    class: "message-header",
                    span { "{title}" }
                    button {
                        class: "{close_class}",
                        onclick: move |_event| {
                            if let Some(handler) = cx.props.on_close.as_ref() {
                                handler.call(false);
                            }
                        }
                    }
                })
            }
            div {
                class: "message-body",
                &cx.props.children
            }
        }
    }
}

/// The [`Message`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct MessageProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply to the `close` button element.
    #[props(into)]
    pub close_class: Option<Class<'a>>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<'a, bool>>,
    /// A flag to determine whether the message is hidden or not.
    #[props(default = false)]
    pub hidden: bool,
    /// The title in the message header.
    #[props(into)]
    pub title: Option<Cow<'a, str>>,
    /// The children to render within the component.
    children: Element<'a>,
}
