use crate::class::Class;
use dioxus::prelude::*;

/// Responsive columns powered by flexbox.
pub fn Columns(props: ColumnsProps) -> Element {
    let class = props.class;
    let column_class = props.column_class;
    let size_class = if let Some(size) = props.size {
        format!("is-{size}")
    } else {
        String::new()
    };
    let offset_class = if let Some(offset) = props.offset {
        format!("is-offset-{offset}")
    } else {
        String::new()
    };
    let gap_class = if let Some(gap) = props.gap {
        format!("is-variable is-{gap}")
    } else {
        String::new()
    };
    let multiline_class = Class::check("is-multiline", props.multiline);
    rsx! {
        div {
            class: "{class} {gap_class} {multiline_class}",
            for column in props.columns.iter() {
                div {
                    class: "{column_class} {size_class} {offset_class}",
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
    #[props(into, default = "columns".into())]
    pub class: Class,
    /// A class to apply to a single column.
    #[props(into, default = "column".into())]
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
