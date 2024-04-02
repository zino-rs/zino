use crate::view::{
    dependency::DependencyList, layout::Wrapper, overview::Overview, stargazer::StargazerList,
};
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Clone, PartialEq, Eq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Wrapper)]
        #[route("/")]
        Overview {},
        #[route("/stargazers")]
        StargazerList {},
        #[route("/dependencies")]
        DependencyList {},
    #[end_layout]
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

impl Default for Route {
    fn default() -> Self {
        Self::Overview {}
    }
}

#[component]
fn PageNotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div {
            class: "notification is-danger is-light",
            h3 { "Page not found" }
            p { "The page `{path}` you requested doesn't exist." }
        }
    }
}
