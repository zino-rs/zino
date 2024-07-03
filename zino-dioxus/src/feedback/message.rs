use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// Contextual feedback messages for typical user actions.
pub fn Message(props: MessageProps) -> Element {
    if props.hidden {
        return None;
    }
    rsx! {
        div {
            class: props.class,
            if !props.title.is_empty() {
                div {
                    class: "message-header",
                    span { { props.title } }
                    button {
                        r#type: "button",
                        class: props.close_class,
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
    #[props(into, default = "message".into())]
    pub class: Class,
    /// A class to apply to the `close` button element.
    #[props(into, default = "delete".into())]
    pub close_class: Class,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<bool>>,
    /// A flag to determine whether the message is hidden or not.
    #[props(default)]
    pub hidden: bool,
    /// The title in the message header.
    #[props(into, default)]
    pub title: SharedString,
    /// The children to render within the component.
    children: Element,
}
