use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;

/// A container for the form field with a label.
pub fn FormFieldContainer<'a>(cx: Scope<'a, FormFieldContainerProps<'a>>) -> Element {
    let class = format_class!(cx, "field is-horizontal");
    let field_label_class = format_class!(cx, field_label_class, "field-label");
    let field_body_class = format_class!(cx, field_body_class, "field-body");
    let label_class = format_class!(cx, label_class, "label");
    render! {
        div {
            class: "{class}",
            div {
                class: "{field_label_class}",
                label {
                    class: "{label_class}",
                    "{cx.props.label}"
                }
            }
            div {
                class: "{field_body_class}",
                &cx.props.children
            }
        }
    }
}

/// The [`FormFieldContainer`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FormFieldContainerProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply to the field label container.
    #[props(into)]
    pub field_label_class: Option<Class<'a>>,
    /// A class to apply to the field body container.
    #[props(into)]
    pub field_body_class: Option<Class<'a>>,
    /// A class to apply to the `label` element.
    #[props(into)]
    pub label_class: Option<Class<'a>>,
    /// The label content.
    #[props(into)]
    pub label: Cow<'a, str>,
    /// The children to render within the component.
    children: Element<'a>,
}

/// A single field with the form control.
pub fn FormField<'a>(cx: Scope<'a, FormFieldProps<'a>>) -> Element {
    let class = format_class!(cx, "field");
    let control_class = format_class!(cx, control_class, "control");
    render! {
        div {
            class: "{class}",
            div {
                class: "{control_class}",
                &cx.props.children
            }
        }
    }
}

/// The [`FormField`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FormFieldProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}

/// Grouped fields with the form control.
pub fn FormGroup<'a>(cx: Scope<'a, FormGroupProps<'a>>) -> Element {
    let class = format_class!(cx, "field is-grouped");
    render! {
        div {
            class: "{class}",
            &cx.props.children
        }
    }
}

/// The [`FormGroup`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FormGroupProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class<'a>>,
    /// The children to render within the component.
    children: Element<'a>,
}
