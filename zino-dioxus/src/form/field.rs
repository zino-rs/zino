use crate::{class::Class, format_class};
use dioxus::prelude::*;
use zino_core::SharedString;

/// A container for the form field with a label.
pub fn FormFieldContainer(props: FormFieldContainerProps) -> Element {
    let class = format_class!(props, "field is-horizontal");
    let field_label_class = format_class!(props, field_label_class, "field-label");
    let field_body_class = format_class!(props, field_body_class, "field-body");
    let label_class = format_class!(props, label_class, "label");
    rsx! {
        div {
            class: "{class}",
            div {
                class: "{field_label_class}",
                label {
                    class: "{label_class}",
                    "{props.label}"
                }
            }
            div {
                class: "{field_body_class}",
                { props.children }
            }
        }
    }
}

/// The [`FormFieldContainer`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FormFieldContainerProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply to the field label container.
    #[props(into)]
    pub field_label_class: Option<Class>,
    /// A class to apply to the field body container.
    #[props(into)]
    pub field_body_class: Option<Class>,
    /// A class to apply to the `label` element.
    #[props(into)]
    pub label_class: Option<Class>,
    /// The label content.
    #[props(into)]
    pub label: SharedString,
    /// The children to render within the component.
    children: Element,
}

/// A single field with the form control.
pub fn FormField(props: FormFieldProps) -> Element {
    let class = format_class!(props, "field");
    let control_class = format_class!(props, control_class, "control");
    let expanded_class = Class::check("is-expanded", props.expanded);
    rsx! {
        div {
            class: "{class}",
            div {
                class: "{control_class} {expanded_class}",
                { props.children }
            }
        }
    }
}

/// The [`FormField`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FormFieldProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class>,
    /// A flag to determine whether the control is expanded or not.
    #[props(default = false)]
    pub expanded: bool,
    /// The children to render within the component.
    children: Element,
}

/// Grouped fields with the form control.
pub fn FormGroup(props: FormGroupProps) -> Element {
    let class = format_class!(props, "field is-grouped");
    let container_class = if let Some(prop) = props.float {
        match prop.as_ref() {
            "left" => format!("{class} is-pulled-left").into(),
            "right" => format!("{class} is-pulled-right").into(),
            _ => class,
        }
    } else {
        class
    };
    let control_class = format_class!(props, control_class, "control");
    rsx! {
        div {
            class: "{container_class}",
            for item in props.items.iter() {
                div {
                    class: "{control_class}",
                    { item }
                }
            }
        }
    }
}

/// The [`FormGroup`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FormGroupProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class>,
    /// The `float` CSS property: `left` | `right`.
    pub float: Option<SharedString>,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}

/// Attaches inputs, buttons, and dropdowns together with the form control.
pub fn FormAddons(props: FormAddonsProps) -> Element {
    let class = format_class!(props, "field has-addons");
    let control_class = format_class!(props, control_class, "control");
    let expand = props.expand;
    let items = props.items.iter().enumerate().map(|(index, item)| {
        let expand_class = Class::check("is-expanded", expand == index + 1);
        (expand_class, item)
    });
    rsx! {
        div {
            class: "{class}",
            for (expand_class, item) in items {
                div {
                    class: "{control_class} {expand_class}",
                    { item }
                }
            }
        }
    }
}

/// The [`FormAddons`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FormAddonsProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply custom styles.
    #[props(into)]
    pub control_class: Option<Class>,
    /// A modifier to expand the `n`th element to fill up the remaining space.
    pub expand: usize,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}
