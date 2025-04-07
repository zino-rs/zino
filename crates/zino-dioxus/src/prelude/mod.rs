//! Re-exports of components and common types.

pub use crate::{
    class::Class,
    extension::FormDataExt,
    feedback::{Message, ModalCard, ModalData, Notification, OperationResult},
    form::{
        Button, Buttons, Checkbox, DataEntry, DataSelect, FileTree, FileUpload, FormAddons,
        FormField, FormFieldContainer, FormGroup, Input, Progress, Radio, Textarea,
    },
    icon::{Icon, IconText, SvgIcon},
    layout::{Columns, Container, FluidContainer, MainContainer},
    navigation::{
        ContextMenu, Dropdown, Navbar, NavbarBrand, NavbarCenter, NavbarDropdown, NavbarEnd,
        NavbarItem, NavbarLink, NavbarMenu, NavbarStart, Pagination, Sidebar,
    },
    theme::Theme,
    typography::{Card, FixedWidthSpan, Tag, Tags, TuiEditor},
};

#[cfg(feature = "clipboard")]
pub use crate::form::CopyToClipboard;

#[cfg(feature = "desktop")]
pub use crate::application::Desktop;

#[cfg(feature = "markdown")]
pub use crate::typography::Markdown;

#[doc(no_inline)]
pub use dioxus_router::components::Router;
