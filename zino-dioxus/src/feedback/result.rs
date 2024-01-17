use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;
use zino_core::error::Error;

/// Global messages as feedback in response to user operations.
pub fn OperationResult<'a>(cx: Scope<'a, OperationResultProps<'a>>) -> Element {
    if cx.props.hidden {
        return None;
    }

    let class = format_class!(cx, "message");
    let close_class = format_class!(cx, close_class, "delete");
    let title = cx.props.title.as_deref().unwrap_or_default();
    match cx.props.future.value() {
        Some(Ok(())) => {
            render! {
                div {
                    class: "{class} is-success",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
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
                    class: "{class} is-danger",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
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
            let loading = cx.props.loading.as_ref()?;
            render! {
                div {
                    class: "{class} is-warning",
                    position: "fixed",
                    top: "4rem",
                    right: "0.75rem",
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
        }
    }
}

/// The [`OperationResult`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct OperationResultProps<'a> {
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
    /// A flag to determine whether the message is hidden or not.
    #[props(default = false)]
    pub hidden: bool,
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
