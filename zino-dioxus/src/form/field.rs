use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// A container for the form field with a label.
pub fn FormFieldContainer(props: FormFieldContainerProps) -> Element {
    let class = props.class;
    let field_label_class = props.field_label_class;
    let field_body_class = props.field_body_class;
    let label_class = props.label_class;
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
    #[props(into, default = "field is-horizontal".into())]
    pub class: Class,
    /// A class to apply to the field label container.
    #[props(into, default = "field-label".into())]
    pub field_label_class: Class,
    /// A class to apply to the field body container.
    #[props(into, default = "field-body".into())]
    pub field_body_class: Class,
    /// A class to apply to the `label` element.
    #[props(into, default = "label".into())]
    pub label_class: Class,
    /// The label content.
    #[props(into)]
    pub label: SharedString,
    /// The children to render within the component.
    children: Element,
}

/// A single field with the form control.
pub fn FormField(props: FormFieldProps) -> Element {
    let class = props.class;
    let control_class = props.control_class;
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
    #[props(into, default = "field".into())]
    pub class: Class,
    /// A class to apply custom styles.
    #[props(into, default = "control".into())]
    pub control_class: Class,
    /// A flag to determine whether the control is expanded or not.
    #[props(default = false)]
    pub expanded: bool,
    /// The children to render within the component.
    children: Element,
}

/// Grouped fields with the form control.
pub fn FormGroup(props: FormGroupProps) -> Element {
    let class = props.class;
    let container_class = match props.align.as_ref() {
        "center" => format!("{class} is-grouped-centered"),
        "right" => format!("{class} is-grouped-right"),
        _ => class.to_string(),
    };
    let control_class = props.control_class;
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
    #[props(into, default = "field is-grouped".into())]
    pub class: Class,
    /// A class to apply custom styles.
    #[props(into, default = "control".into())]
    pub control_class: Class,
    /// The alignment of the group: `left` | "center" | `right`.
    #[props(into, default = "left".into())]
    pub align: SharedString,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}

/// Attaches inputs, buttons, and dropdowns together with the form control.
pub fn FormAddons(props: FormAddonsProps) -> Element {
    let class = props.class;
    let control_class = props.control_class;
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
    #[props(into, default = "field has-addons".into())]
    pub class: Class,
    /// A class to apply custom styles.
    #[props(into, default = "control".into())]
    pub control_class: Class,
    /// A modifier to expand the `n`th element to fill up the remaining space.
    pub expand: usize,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}
