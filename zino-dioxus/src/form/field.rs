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
    let expanded_class = Class::check("is-expanded", cx.props.expanded);
    render! {
        div {
            class: "{class}",
            div {
                class: "{control_class} {expanded_class}",
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
    /// A flag to determine whether the control is expanded or not.
    #[props(default = false)]
    pub expanded: bool,
    /// The children to render within the component.
    children: Element<'a>,
}

/// Grouped fields with the form control.
pub fn FormGroup<'a>(cx: Scope<'a, FormGroupProps<'a>>) -> Element {
    let class = format_class!(cx, "field is-grouped");
    let container_class = if let Some(prop) = cx.props.float {
        match prop {
            "left" => format!("{class} is-pulled-left").into(),
            "right" => format!("{class} is-pulled-right").into(),
            _ => class,
        }
    } else {
        class
    };
    let control_class = format_class!(cx, control_class, "control");
    render! {
        div {
            class: "{container_class}",
            for item in cx.props.items.iter() {
                div {
                    class: "{control_class}",
                    item
                }
            }
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
    /// The `float` CSS property: `left` | `right`.
    pub float: Option<&'a str>,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element<'a>>,
}

/// Attaches inputs, buttons, and dropdowns together with the form control.
pub fn FormAddons<'a>(cx: Scope<'a, FormAddonsProps<'a>>) -> Element {
    let class = format_class!(cx, "field has-addons");
    let control_class = format_class!(cx, control_class, "control");
    let expand = cx.props.expand;
    let items = cx.props.items.iter().enumerate().map(|(index, item)| {
        let expand_class = Class::check("is-expanded", expand == index + 1);
        (expand_class, item)
    });
    render! {
        div {
            class: "{class}",
            for (expand_class, item) in items {
                div {
                    class: "{control_class} {expand_class}",
                    item
                }
            }
        }
    }
}

/// The [`FormAddons`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FormAddonsProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class<'a>>,
    /// A modifier to expand the `n`th element to fill up the remaining space.
    pub expand: usize,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element<'a>>,
}
