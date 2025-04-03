use crate::class::Class;
use dioxus::prelude::*;
use std::mem;
use zino_core::SharedString;

/// The text input and its variations.
pub fn Input(props: InputProps) -> Element {
    let mut composing = use_signal(|| false);
    let mut cached_value = use_signal(String::new);
    rsx! {
       input {
            class: "{props.class}",
            class: if !props.color.is_empty() { "is-{props.color}" },
            class: if !props.size.is_empty() { "is-{props.size}" },
            class: if !props.state.is_empty() { "is-{props.state}" },
            value: if let Some(value) = props.initial_value { "{value}" },
            r#type: "text",
            onmounted: move |event| {
                if props.auto_focus {
                    spawn(async move {
                        if let Err(err) = event.data.set_focus(true).await {
                            tracing::error!("fail to focus on the input: {err}");
                        }
                    });
                }
            },
            onchange: move |event| {
                if let Some(handler) = props.on_change.as_ref() {
                    handler.call(event.value());
                }
            },
            oninput: move |event| {
                cached_value.set(event.value());
                if !composing() {
                    if let Some(handler) = props.on_input.as_ref() {
                        handler.call(mem::take(&mut cached_value.write()));
                    }
                }
            },
            onkeydown: move |event| {
                if let Some(handler) = props.on_keydown.as_ref() {
                    event.stop_propagation();
                    if !composing() {
                        handler.call(event);
                    }
                }
            },
            oncompositionstart: move |_event| {
                composing.set(true);
            },
            oncompositionend: move |_event| {
                composing.set(false);
                if let Some(handler) = props.on_input.as_ref() {
                    handler.call(mem::take(&mut cached_value.write()));
                }
            },
            ..props.attributes,
        }
    }
}

/// The [`Input`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct InputProps {
    /// The class attribute for the component.
    #[props(into, default = "input")]
    pub class: Class,
    /// The color of the input: `primary` | `link` | `info` | `success` | `warning` | `danger`.
    #[props(into, default)]
    pub color: SharedString,
    /// The size of the input: `small` | `normal` | `medium` | `large`.
    #[props(into, default)]
    pub size: SharedString,
    /// The state of the input: `hovered` | `focused` | `loading` | `static`.
    #[props(into, default)]
    pub state: SharedString,
    /// A flag to determine whether the textarea is focused automatically.
    #[props(default)]
    pub auto_focus: bool,
    /// The initial value of the textarea.
    #[props(into, default)]
    pub initial_value: Option<String>,
    /// An event handler to be called when the input state is changed.
    pub on_change: Option<EventHandler<String>>,
    /// An event handler to be called when inputing.
    pub on_input: Option<EventHandler<String>>,
    /// An event handler to be called when a key is pressed.
    pub on_keydown: Option<EventHandler<KeyboardEvent>>,
    /// Spreading the props of the `input` element.
    #[props(extends = input)]
    attributes: Vec<Attribute>,
}
