use crate::class::Class;
use dioxus::prelude::*;
use zino_core::SharedString;

/// A container for the form field with a label.
pub fn FormFieldContainer(props: FormFieldContainerProps) -> Element {
    rsx! {
        div {
            class: props.class,
            div {
                class: props.field_label_class,
                label {
                    class: props.label_class,
                    { props.label }
                }
            }
            div {
                class: props.field_body_class,
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
    rsx! {
        div {
            class: props.class,
            div {
                class: props.control_class,
                class: if props.expanded { "is-expanded" },
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
    rsx! {
        div {
            class: props.class,
            class: if props.align == "center" { "is-grouped-centered" },
            class: if props.align == "right" { "is-grouped-right" },
            for item in props.items.iter() {
                div {
                    class: props.control_class.clone(),
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
    /// The alignment of the group: `left` | `center` | `right`.
    #[props(into, default)]
    pub align: SharedString,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}

/// Attaches inputs, buttons, and dropdowns together with the form control.
pub fn FormAddons(props: FormAddonsProps) -> Element {
    rsx! {
        div {
            class: props.class,
            for (index, item) in props.items.iter().enumerate() {
                div {
                    class: props.control_class.clone(),
                    class: if props.expand == index + 1 { "is-expanded" },
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
    #[props(default = 0)]
    pub expand: usize,
    /// The items to be grouped.
    #[props(into)]
    pub items: Vec<Element>,
}
