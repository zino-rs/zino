use crate::class::Class;
use dioxus::prelude::*;

/// Responsive columns powered by flexbox.
pub fn Columns(props: ColumnsProps) -> Element {
    rsx! {
        div {
            class: "{props.class}",
            class: if props.multiline { "is-multiline" },
            class: if let Some(gap) = props.gap { "is-variable is-{gap}" },
            for column in props.columns.iter() {
                div {
                    class: "{props.column_class}",
                    class: if let Some(size) = props.size { "is-{size}" },
                    class: if let Some(offset) = props.offset { "is-offset-{offset}" },
                    { column }
                }
            }
        }
    }
}

/// The [`Columns`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ColumnsProps {
    /// The class attribute for the component.
    #[props(into, default = "columns")]
    pub class: Class,
    /// A class to apply to a single column.
    #[props(into, default = "column")]
    pub column_class: Class,
    /// A custom size in the 12 columns system.
    #[props(into)]
    pub size: Option<u8>,
    /// A custom column offset.
    #[props(into)]
    pub offset: Option<u8>,
    /// A custom column gap.
    #[props(into)]
    pub gap: Option<u8>,
    /// A flag to add more column elements than would fit in a single row.
    #[props(default)]
    pub multiline: bool,
    /// The columns to be rendered.
    #[props(into)]
    pub columns: Vec<Element>,
}
