use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::{borrow::Cow, time::Duration};
use zino_core::error::Error;

/// Global messages as feedback in response to user operations.
pub fn Message<'a>(cx: Scope<'a, MessageProps<'a>>) -> Element {
    let Some(value) = cx.props.future.value() else {
        return None;
    };
    let close_message = use_state(cx, || false);
    if *close_message.get() {
        return None;
    }

    let duration = cx.props.duration;
    cx.spawn({
        to_owned![close_message];
        async move {
            tokio::time::sleep(Duration::from_millis(duration)).await;
            close_message.set(true);
        }
    });

    let class = format_class!(cx, "message");
    let close_class = format_class!(cx, close_class, "delete");
    let title = cx.props.title.as_deref().unwrap_or_default();
    match value {
        Ok(()) => {
            render! {
                div {
                    class: "{class} is-success",
                    if !title.is_empty() {
                        rsx!(div {
                            class: "message-header",
                            span { "{title}" }
                            button {
                                class: "{close_class}",
                                onclick: move |event| {
                                    if let Some(handler) = cx.props.on_close.as_ref() {
                                        handler.call(event);
                                    }
                                    close_message.set(true);
                                }
                            }
                        })
                    }
                    div {
                        class: "message-body",
                        "{cx.props.success}"
                    }
                }
            }
        }
        Err(err) => {
            render! {
                div {
                    class: "{class} is-danger",
                    if !title.is_empty() {
                        rsx!(div {
                            class: "message-header",
                            span { "{title}" }
                            button {
                                class: "{close_class}",
                                onclick: move |event| {
                                    if let Some(handler) = cx.props.on_close.as_ref() {
                                        handler.call(event);
                                    }
                                    close_message.set(true);
                                }
                            }
                        })
                    }
                    div {
                        class: "message-body",
                        span { "{cx.props.error}" }
                        span { "{err}" }
                    }
                }
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
    /// A future value which represents the user operations.
    pub future: &'a UseFuture<Result<(), Error>>,
    /// An event handler to be called when the `close` button is clicked.
    pub on_close: Option<EventHandler<'a, MouseEvent>>,
    /// The title in the message header.
    #[props(into)]
    pub title: Option<Cow<'a, str>>,
    /// The success message to render when the future value is resolved.
    #[props(into)]
    pub success: Cow<'a, str>,
    /// An error message to render when the future value is rejected.
    #[props(into)]
    pub error: Cow<'a, str>,
    /// Time before auto-close in milliseconds.
    #[props(default = 3000)]
    pub duration: u64,
}
