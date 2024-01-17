use super::DataEntry;
use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;
use zino_core::error::Error;

/// A control that provides a menu of data entries.
pub fn DataSelect<'a, T: DataEntry>(cx: Scope<'a, DataSelectProps<'a, T>>) -> Element {
    let class = format_class!(cx, "select");
    let fullwidth_class = Class::check("is-fullwidth", cx.props.fullwidth);
    if let Some(Ok(entries)) = cx.props.future.value() {
        render! {
            div {
                class: "{class} {fullwidth_class}",
                select {
                    name: "{cx.props.name}",
                    required: cx.props.required,
                    onmounted: move |_event| {
                        if let Some(handler) = cx.props.on_select.as_ref() {
                            if let Some(entry) = entries.first() {
                                handler.call(entry);
                            }
                        }
                    },
                    onchange: move |event| {
                        if let Some(handler) = cx.props.on_select.as_ref() {
                           let value = event.inner().value.as_str();
                           if let Some(entry) = entries.iter().find(|d| d.value() == value) {
                               handler.call(entry);
                           }
                        }
                    },
                    for entry in entries {
                        option {
                            key: "{entry.key()}",
                            value: "{entry.value()}",
                            "{entry.label()}"
                        }
                    }
                }
            }
        }
    } else {
        render! {
            div {
                class: "{class} {fullwidth_class}",
                select {
                    name: "{cx.props.name}",
                    required: cx.props.required,
                }
            }
        }
    }
}

/// The [`DataSelect`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct DataSelectProps<'a, T: 'static> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A future value which represents the data entries.
    pub future: &'a UseFuture<Result<Vec<T>, Error>>,
    /// The name of the control.
    #[props(into)]
    pub name: Cow<'a, str>,
    /// A flag to determine whether the control is fullwidth or not.
    #[props(default = false)]
    pub fullwidth: bool,
    /// A flag to determine whether the control is required or not.
    #[props(default = false)]
    pub required: bool,
    /// An event handler to be called when the choice is selected.
    pub on_select: Option<EventHandler<'a, &'a T>>,
}
