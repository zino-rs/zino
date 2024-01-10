use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;
use zino_core::error::Error;

/// Global messages as feedback in response to user operations.
pub fn Message<'a>(cx: Scope<'a, MessageProps<'a>>) -> Element {
    let class = format_class!(cx, "message");
    let close_class = format_class!(cx, close_class, "delete");
    let hidden_class = Class::check("is-hidden", !cx.props.visible);
    let title = cx.props.title.as_deref().unwrap_or_default();
    match cx.props.future.value() {
        Some(Ok(())) => {
            render! {
                div {
                    class: "{class} {hidden_class} is-success",
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
                        "{cx.props.success}"
                    }
                }
            }
        }
        Some(Err(err)) => {
            render! {
                div {
                    class: "{class} {hidden_class} is-danger",
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
                        span { "{cx.props.error}" }
                        span { "{err}" }
                    }
                }
            }
        }
        None => {
            if let Some(loading) = cx.props.loading.as_ref() {
                render! {
                    div {
                        class: "{class} {hidden_class} is-warning",
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
                            span { "{loading}" }
                        }
                    }
                }
            } else {
                render! {
                    div {
                        class: "{class} is-hidden",
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
    pub on_close: Option<EventHandler<'a, bool>>,
    /// A flag to determine whether the message is visible or not.
    #[props(default = false)]
    pub visible: bool,
    /// The title in the message header.
    #[props(into)]
    pub title: Option<Cow<'a, str>>,
    /// A message to render when the future value is not ready.
    #[props(into)]
    pub loading: Option<Cow<'a, str>>,
    /// A message to render when the future value is resolved.
    #[props(into)]
    pub success: Cow<'a, str>,
    /// A message to render when the future value is rejected.
    #[props(into)]
    pub error: Cow<'a, str>,
}
