use crate::view::{
    dataset::{Dataset, DatasetList, DatasetView},
    home::Home,
    layout::Wrapper,
};
use dioxus::prelude::*;
use dioxus_router::prelude::*;
use zino::prelude::*;

#[derive(Clone, PartialEq, Eq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Wrapper)]
        #[route("/")]
        Home {},
        #[nest("/dataset")]
            #[layout(Dataset)]
                #[route("/list")]
                DatasetList {},
                #[route("/:id/view")]
                DatasetView { id: Uuid },
            #[end_layout]
        #[end_nest]
    #[end_layout]
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

impl Default for Route {
    fn default() -> Self {
        Self::Home {}
    }
}

#[inline_props]
fn PageNotFound(cx: Scope, segments: Vec<String>) -> Element {
    let path = segments.join("/");
    render! {
        h1 { "Page not found" }
        p { "The page `{path}` you requested doesn't exist." }
    }
}
